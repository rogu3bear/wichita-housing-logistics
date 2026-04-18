use serde::Deserialize;
use worker::D1Type;

use crate::api::{HousingResource, ResourceStatusCounts, ResourcesResponse};

use super::{
    d1_error, database, normalize_text, row_id_arg, validate_allowed, AppError, AppResult,
    GroupCountRow,
};

pub const ALLOWED_KINDS: &[&str] = &[
    "shelter_bed",
    "transitional",
    "permanent_supportive",
    "rental_unit",
    "other",
];

pub const ALLOWED_STATUSES: &[&str] = &["available", "held", "occupied", "offline"];

#[derive(Debug, Deserialize)]
struct ResourceRow {
    id: i64,
    label: String,
    kind: String,
    address: Option<String>,
    capacity: i64,
    status: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

fn map(row: ResourceRow) -> HousingResource {
    HousingResource {
        id: row.id,
        label: row.label,
        kind: row.kind,
        address: row.address,
        capacity: row.capacity,
        status: row.status,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_resources() -> AppResult<ResourcesResponse> {
    let db = database()?;

    let statements = vec![
        db.prepare(
            "SELECT id, label, kind, address, capacity, status, notes,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM housing_resources
             ORDER BY id DESC
             LIMIT 500",
        ),
        db.prepare(
            "SELECT status AS key, COUNT(*) AS n FROM housing_resources GROUP BY status",
        ),
    ];

    let results = db
        .batch(statements)
        .await
        .map_err(|e| d1_error("Failed to batch list_resources.", e))?;

    if results.len() != 2 {
        return Err(AppError::internal(
            "resources batch returned unexpected number of results.",
            format!("expected 2, got {}", results.len()),
        ));
    }

    let items = results[0]
        .results::<ResourceRow>()
        .map_err(|e| d1_error("Failed to deserialize resource rows.", e))?
        .into_iter()
        .map(map)
        .collect::<Vec<_>>();

    let count_rows: Vec<GroupCountRow> = results[1]
        .results()
        .map_err(|e| d1_error("Failed to deserialize resource status counts.", e))?;

    let mut counts = ResourceStatusCounts::default();
    for row in count_rows {
        match row.key.as_str() {
            "available" => counts.available = row.n,
            "held" => counts.held = row.n,
            "occupied" => counts.occupied = row.n,
            "offline" => counts.offline = row.n,
            _ => {}
        }
    }

    Ok(ResourcesResponse { items, counts })
}

pub async fn create_resource(
    label: String,
    kind: String,
    address: Option<String>,
    capacity: i64,
    notes: Option<String>,
) -> AppResult<HousingResource> {
    let db = database()?;
    let label = normalize_text("Resource label", &label, 120, true)?
        .ok_or_else(|| AppError::client("Resource label is required."))?;
    let kind = validate_allowed("Kind", kind.trim(), ALLOWED_KINDS)?.to_string();
    let address = address
        .as_deref()
        .map(|a| normalize_text("Address", a, 240, false))
        .transpose()?
        .flatten();
    if capacity < 1 || capacity > 1_000 {
        return Err(AppError::client("Capacity must be between 1 and 1000."));
    }
    let capacity_i32 =
        i32::try_from(capacity).map_err(|_| AppError::client("Capacity overflow."))?;
    let notes = notes
        .as_deref()
        .map(|n| normalize_text("Notes", n, 1_000, false))
        .transpose()?
        .flatten();

    let args = [
        D1Type::Text(label.as_str()),
        D1Type::Text(kind.as_str()),
        match address.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
        D1Type::Integer(capacity_i32),
        match notes.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
    ];

    let result = db
        .prepare(
            "INSERT INTO housing_resources (label, kind, address, capacity, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind resource insert.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to insert housing resource.", e))?;

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

pub async fn set_status(id: i64, status: String) -> AppResult<HousingResource> {
    let db = database()?;
    let status = validate_allowed("Status", status.trim(), ALLOWED_STATUSES)?.to_string();

    let id_arg = row_id_arg(id)?;
    let args = [D1Type::Text(status.as_str()), id_arg];

    let result = db
        .prepare(
            "UPDATE housing_resources
             SET status = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind resource status update.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to update resource status.", e))?;

    let changed = result
        .meta()
        .map_err(|e| d1_error("Failed to read update metadata.", e))?
        .and_then(|meta| meta.changes)
        .unwrap_or_default();
    if changed == 0 {
        return Err(AppError::client(format!("Resource {id} was not found.")));
    }
    get_by_id(&db, id).await
}

async fn get_by_id(db: &worker::D1Database, id: i64) -> AppResult<HousingResource> {
    let id_arg = row_id_arg(id)?;
    let row = db
        .prepare(
            "SELECT id, label, kind, address, capacity, status, notes,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at
             FROM housing_resources
             WHERE id = ?1",
        )
        .bind_refs(&id_arg)
        .map_err(|e| d1_error("Failed to bind resource lookup.", e))?
        .first::<ResourceRow>(None)
        .await
        .map_err(|e| d1_error("Failed to fetch resource.", e))?;
    row.map(map)
        .ok_or_else(|| AppError::client(format!("Resource {id} was not found.")))
}
