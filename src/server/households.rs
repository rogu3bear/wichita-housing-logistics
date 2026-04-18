use serde::Deserialize;
use worker::D1Type;

use crate::api::{CasePlacement, CaseUpdate, CaseView, Household, HouseholdsResponse, StageCounts};

use super::{
    d1_error, database, normalize_text, row_id_arg, validate_allowed, AppError, AppResult,
    GroupCountRow,
};

pub const ALLOWED_STAGES: &[&str] = &[
    "intake",
    "assessment",
    "placement",
    "follow_up",
    "exited",
];

#[derive(Debug, Deserialize)]
struct HouseholdRow {
    id: i64,
    head_name: String,
    household_size: i64,
    phone: Option<String>,
    email: Option<String>,
    stage: String,
    intake_notes: Option<String>,
    share_token: Option<String>,
    created_at: String,
    updated_at: String,
}

fn map(row: HouseholdRow) -> Household {
    Household {
        id: row.id,
        head_name: row.head_name,
        household_size: row.household_size,
        phone: row.phone,
        email: row.email,
        stage: row.stage,
        intake_notes: row.intake_notes,
        share_token: row.share_token,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_households() -> AppResult<HouseholdsResponse> {
    let db = database()?;

    // Two statements per round-trip: the capped list + an un-capped GROUP BY
    // so stage counts don't lie once the table grows past LIMIT 500.
    let statements = vec![
        db.prepare(
            "SELECT id, head_name, household_size, phone, email, stage, intake_notes, share_token,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM households
             ORDER BY id DESC
             LIMIT 500",
        ),
        db.prepare("SELECT stage AS key, COUNT(*) AS n FROM households GROUP BY stage"),
    ];

    let results = db
        .batch(statements)
        .await
        .map_err(|e| d1_error("Failed to batch list_households.", e))?;

    if results.len() != 2 {
        return Err(AppError::internal(
            "households batch returned unexpected number of results.",
            format!("expected 2, got {}", results.len()),
        ));
    }

    let items = results[0]
        .results::<HouseholdRow>()
        .map_err(|e| d1_error("Failed to deserialize household rows.", e))?
        .into_iter()
        .map(map)
        .collect::<Vec<_>>();

    let count_rows: Vec<GroupCountRow> = results[1]
        .results()
        .map_err(|e| d1_error("Failed to deserialize household stage counts.", e))?;

    let mut counts = StageCounts::default();
    for row in count_rows {
        match row.key.as_str() {
            "intake" => counts.intake = row.n,
            "assessment" => counts.assessment = row.n,
            "placement" => counts.placement = row.n,
            "follow_up" => counts.follow_up = row.n,
            "exited" => counts.exited = row.n,
            _ => {}
        }
    }

    Ok(HouseholdsResponse { items, counts })
}

pub async fn create_household(
    head_name: String,
    household_size: i64,
    phone: Option<String>,
    email: Option<String>,
    intake_notes: Option<String>,
) -> AppResult<Household> {
    let db = database()?;

    let head_name = normalize_text("Head of household name", &head_name, 120, true)?
        .ok_or_else(|| AppError::client("Head of household name is required."))?;
    if household_size < 1 || household_size > 32 {
        return Err(AppError::client(
            "Household size must be between 1 and 32.",
        ));
    }
    let phone = phone
        .as_deref()
        .map(|p| normalize_text("Phone", p, 40, false))
        .transpose()?
        .flatten();
    let email = email
        .as_deref()
        .map(|e| normalize_text("Email", e, 120, false))
        .transpose()?
        .flatten();
    let intake_notes = intake_notes
        .as_deref()
        .map(|n| normalize_text("Intake notes", n, 1_000, false))
        .transpose()?
        .flatten();

    let size_i32 =
        i32::try_from(household_size).map_err(|_| AppError::client("Household size overflow."))?;

    let args = [
        D1Type::Text(head_name.as_str()),
        D1Type::Integer(size_i32),
        match phone.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
        match email.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
        match intake_notes.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
    ];

    // share_token generated SQL-side via lower(hex(randomblob(12))) so we
    // don't have to carry a CSPRNG through the Rust layer just for 12 bytes.
    let result = db
        .prepare(
            "INSERT INTO households (head_name, household_size, phone, email, intake_notes, share_token)
             VALUES (?1, ?2, ?3, ?4, ?5, lower(hex(randomblob(12))))",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind household insert.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to insert household.", e))?;

    let inserted_id = result
        .meta()
        .map_err(|e| d1_error("Failed to read insert metadata.", e))?
        .and_then(|meta| meta.last_row_id)
        .ok_or_else(|| {
            AppError::internal(
                "INSERT completed without returning last_row_id.",
                "missing last_row_id",
            )
        })?;

    get_by_id(&db, inserted_id).await
}

pub async fn set_stage(id: i64, stage: String) -> AppResult<Household> {
    let db = database()?;
    let stage = validate_allowed("Stage", stage.trim(), ALLOWED_STAGES)?.to_string();

    let id_arg = row_id_arg(id)?;
    let args = [D1Type::Text(stage.as_str()), id_arg];

    let result = db
        .prepare(
            "UPDATE households
             SET stage = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind stage update.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to update household stage.", e))?;

    let changed = result
        .meta()
        .map_err(|e| d1_error("Failed to read update metadata.", e))?
        .and_then(|meta| meta.changes)
        .unwrap_or_default();
    if changed == 0 {
        return Err(AppError::client(format!("Household {id} was not found.")));
    }
    get_by_id(&db, id).await
}

async fn get_by_id(db: &worker::D1Database, id: i64) -> AppResult<Household> {
    let id_arg = row_id_arg(id)?;
    let row = db
        .prepare(
            "SELECT id, head_name, household_size, phone, email, stage, intake_notes, share_token,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM households
             WHERE id = ?1",
        )
        .bind_refs(&id_arg)
        .map_err(|e| d1_error("Failed to bind household lookup.", e))?
        .first::<HouseholdRow>(None)
        .await
        .map_err(|e| d1_error("Failed to fetch household.", e))?;
    row.map(map)
        .ok_or_else(|| AppError::client(format!("Household {id} was not found.")))
}

// --- Customer case page (/case/:token) -----------------------------------
//
// The share token is authentication: whoever holds the URL can read the
// case page and post one-way updates back to the activity feed. Tokens are
// 24-char lowercase hex (12 random bytes) generated SQL-side on insert.

#[derive(Debug, Deserialize)]
struct CaseHouseholdRow {
    head_name: String,
    household_size: i64,
    stage: String,
    intake_notes: Option<String>,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CasePlacementRow {
    resource_label: String,
    status: String,
    started_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CaseUpdateRow {
    body: String,
    author: String,
    created_at: String,
}

fn normalize_share_token(token: &str) -> AppResult<String> {
    let trimmed = token.trim().to_ascii_lowercase();
    if trimmed.len() < 8 || trimmed.len() > 64 {
        return Err(AppError::client("Invalid case link."));
    }
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::client("Invalid case link."));
    }
    Ok(trimmed)
}

pub async fn case_view(token: String) -> AppResult<CaseView> {
    let db = database()?;
    let token = normalize_share_token(&token)?;
    let token_arg = D1Type::Text(token.as_str());

    // Three reads in one batch. Statements 2 & 3 subquery households by
    // token so we only bind one argument per statement.
    let statements = vec![
        db.prepare(
            "SELECT head_name, household_size, stage, intake_notes,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM households
             WHERE share_token = ?1",
        )
        .bind_refs(&token_arg)
        .map_err(|e| d1_error("Failed to bind case household lookup.", e))?,
        db.prepare(
            "SELECT r.label AS resource_label, p.status,
                    strftime('%Y-%m-%d %H:%M UTC', p.started_at) AS started_at
             FROM placements p
             JOIN housing_resources r ON r.id = p.resource_id
             WHERE p.household_id = (SELECT id FROM households WHERE share_token = ?1)
             ORDER BY p.id DESC
             LIMIT 1",
        )
        .bind_refs(&token_arg)
        .map_err(|e| d1_error("Failed to bind case placement lookup.", e))?,
        db.prepare(
            "SELECT body, author,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at
             FROM activity_notes
             WHERE entity_type = 'household'
               AND entity_id = (SELECT id FROM households WHERE share_token = ?1)
             ORDER BY id DESC
             LIMIT 10",
        )
        .bind_refs(&token_arg)
        .map_err(|e| d1_error("Failed to bind case activity lookup.", e))?,
    ];

    let results = db
        .batch(statements)
        .await
        .map_err(|e| d1_error("Failed to batch case_view queries.", e))?;

    if results.len() != 3 {
        return Err(AppError::internal(
            "case_view batch returned unexpected number of results.",
            format!("expected 3, got {}", results.len()),
        ));
    }

    let household_rows: Vec<CaseHouseholdRow> = results[0]
        .results()
        .map_err(|e| d1_error("Failed to deserialize case household row.", e))?;
    let Some(household) = household_rows.into_iter().next() else {
        // Don't leak whether the token shape is valid-but-wrong vs revoked.
        return Err(AppError::client("This case link isn't active."));
    };

    let placement_rows: Vec<CasePlacementRow> = results[1]
        .results()
        .map_err(|e| d1_error("Failed to deserialize case placement row.", e))?;
    let current_placement = placement_rows.into_iter().next().map(|r| CasePlacement {
        resource_label: r.resource_label,
        status: r.status,
        started_at: r.started_at,
    });

    let update_rows: Vec<CaseUpdateRow> = results[2]
        .results()
        .map_err(|e| d1_error("Failed to deserialize case activity rows.", e))?;
    let recent_updates = update_rows
        .into_iter()
        .map(|r| CaseUpdate {
            body: r.body,
            author_is_household: r.author == "household",
            created_at: r.created_at,
        })
        .collect();

    Ok(CaseView {
        head_name: household.head_name,
        household_size: household.household_size,
        stage: household.stage,
        intake_notes: household.intake_notes,
        current_placement,
        recent_updates,
        updated_at: household.updated_at,
    })
}

pub async fn submit_household_update(token: String, body: String) -> AppResult<()> {
    let db = database()?;
    let token = normalize_share_token(&token)?;
    let body = normalize_text("Your update", &body, 1_000, true)?
        .ok_or_else(|| AppError::client("Your update can't be empty."))?;

    let args = [D1Type::Text(token.as_str()), D1Type::Text(body.as_str())];

    let result = db
        .prepare(
            "INSERT INTO activity_notes (entity_type, entity_id, author, body)
             SELECT 'household', id, 'household', ?2
               FROM households
              WHERE share_token = ?1",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind household update insert.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to insert household update.", e))?;

    let changed = result
        .meta()
        .map_err(|e| d1_error("Failed to read household update metadata.", e))?
        .and_then(|meta| meta.changes)
        .unwrap_or_default();

    if changed == 0 {
        // Either the token was invalid or nobody owns it.
        return Err(AppError::client("This case link isn't active."));
    }

    Ok(())
}
