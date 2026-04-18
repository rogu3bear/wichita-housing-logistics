use serde::Deserialize;
use worker::D1Type;

use crate::api::{Placement, PlacementStatusCounts, PlacementsResponse};

use super::{
    d1_error, database, normalize_text, row_id_arg, validate_allowed, AppError, AppResult,
};

pub const ALLOWED_STATUSES: &[&str] = &["proposed", "confirmed", "moved_in", "exited", "cancelled"];

#[derive(Debug, Deserialize)]
struct PlacementRow {
    id: i64,
    household_id: i64,
    resource_id: i64,
    head_name: String,
    resource_label: String,
    status: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

fn map(row: PlacementRow) -> Placement {
    Placement {
        id: row.id,
        household_id: row.household_id,
        resource_id: row.resource_id,
        head_name: row.head_name,
        resource_label: row.resource_label,
        status: row.status,
        started_at: row.started_at,
        ended_at: row.ended_at,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

const LIST_SQL: &str = "SELECT p.id, p.household_id, p.resource_id,
        h.head_name AS head_name,
        r.label AS resource_label,
        p.status, p.started_at, p.ended_at, p.notes,
        strftime('%Y-%m-%d %H:%M UTC', p.created_at) AS created_at,
        strftime('%Y-%m-%d %H:%M UTC', p.updated_at) AS updated_at
    FROM placements p
    JOIN households h ON h.id = p.household_id
    JOIN housing_resources r ON r.id = p.resource_id";

pub async fn list_placements() -> AppResult<PlacementsResponse> {
    let db = database()?;
    let result = db
        .prepare(&format!("{LIST_SQL} ORDER BY p.id DESC"))
        .all()
        .await
        .map_err(|e| d1_error("Failed to list placements.", e))?;

    let items = result
        .results::<PlacementRow>()
        .map_err(|e| d1_error("Failed to deserialize placement rows.", e))?
        .into_iter()
        .map(map)
        .collect::<Vec<_>>();

    let mut counts = PlacementStatusCounts::default();
    for item in &items {
        match item.status.as_str() {
            "proposed" => counts.proposed += 1,
            "confirmed" => counts.confirmed += 1,
            "moved_in" => counts.moved_in += 1,
            "exited" => counts.exited += 1,
            "cancelled" => counts.cancelled += 1,
            _ => {}
        }
    }

    Ok(PlacementsResponse { items, counts })
}

pub async fn create_placement(
    household_id: i64,
    resource_id: i64,
    notes: Option<String>,
) -> AppResult<Placement> {
    let db = database()?;
    let household_arg = row_id_arg(household_id)?;
    let resource_arg = row_id_arg(resource_id)?;
    let notes = notes
        .as_deref()
        .map(|n| normalize_text("Notes", n, 1_000, false))
        .transpose()?
        .flatten();

    let args = [
        household_arg,
        resource_arg,
        match notes.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
    ];

    let result = db
        .prepare(
            "INSERT INTO placements (household_id, resource_id, notes)
             VALUES (?1, ?2, ?3)",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind placement insert.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to insert placement.", e))?;

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

pub async fn set_status(id: i64, status: String) -> AppResult<Placement> {
    let db = database()?;
    let status = validate_allowed("Status", status.trim(), ALLOWED_STATUSES)?.to_string();

    // Auto-stamp lifecycle timestamps on known transitions.
    let started_clause = match status.as_str() {
        "moved_in" => "COALESCE(started_at, CURRENT_TIMESTAMP)",
        _ => "started_at",
    };
    let ended_clause = match status.as_str() {
        "exited" | "cancelled" => "COALESCE(ended_at, CURRENT_TIMESTAMP)",
        _ => "ended_at",
    };
    let sql = format!(
        "UPDATE placements
         SET status = ?1,
             started_at = {started_clause},
             ended_at = {ended_clause},
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?2"
    );

    let id_arg = row_id_arg(id)?;
    let args = [D1Type::Text(status.as_str()), id_arg];

    let result = db
        .prepare(&sql)
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind placement status update.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to update placement status.", e))?;

    let changed = result
        .meta()
        .map_err(|e| d1_error("Failed to read update metadata.", e))?
        .and_then(|meta| meta.changes)
        .unwrap_or_default();
    if changed == 0 {
        return Err(AppError::client(format!("Placement {id} was not found.")));
    }
    get_by_id(&db, id).await
}

async fn get_by_id(db: &worker::D1Database, id: i64) -> AppResult<Placement> {
    let id_arg = row_id_arg(id)?;
    let row = db
        .prepare(&format!("{LIST_SQL} WHERE p.id = ?1"))
        .bind_refs(&id_arg)
        .map_err(|e| d1_error("Failed to bind placement lookup.", e))?
        .first::<PlacementRow>(None)
        .await
        .map_err(|e| d1_error("Failed to fetch placement.", e))?;
    row.map(map)
        .ok_or_else(|| AppError::client(format!("Placement {id} was not found.")))
}
