use serde::Deserialize;
use worker::D1Type;

use crate::api::{Household, HouseholdsResponse, StageCounts};

use super::{
    d1_error, database, normalize_text, row_id_arg, validate_allowed, AppError, AppResult,
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
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_households() -> AppResult<HouseholdsResponse> {
    let db = database()?;

    let result = db
        .prepare(
            "SELECT id, head_name, household_size, phone, email, stage, intake_notes,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM households
             ORDER BY id DESC
             LIMIT 500",
        )
        .all()
        .await
        .map_err(|e| d1_error("Failed to list households.", e))?;

    let items = result
        .results::<HouseholdRow>()
        .map_err(|e| d1_error("Failed to deserialize household rows.", e))?
        .into_iter()
        .map(map)
        .collect::<Vec<_>>();

    let mut counts = StageCounts::default();
    for item in &items {
        match item.stage.as_str() {
            "intake" => counts.intake += 1,
            "assessment" => counts.assessment += 1,
            "placement" => counts.placement += 1,
            "follow_up" => counts.follow_up += 1,
            "exited" => counts.exited += 1,
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

    let result = db
        .prepare(
            "INSERT INTO households (head_name, household_size, phone, email, intake_notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
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
            "SELECT id, head_name, household_size, phone, email, stage, intake_notes,
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
