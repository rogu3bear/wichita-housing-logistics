use serde::Deserialize;
use worker::D1Type;

use crate::api::ActivityNote;

use super::{
    d1_error, database, normalize_text, row_id_arg, validate_allowed, AppError, AppResult,
};

pub const ALLOWED_ENTITIES: &[&str] = &["household", "resource", "placement", "system"];

#[derive(Debug, Deserialize)]
struct NoteRow {
    id: i64,
    entity_type: String,
    entity_id: Option<i64>,
    author: String,
    body: String,
    created_at: String,
}

fn map(row: NoteRow) -> ActivityNote {
    ActivityNote {
        id: row.id,
        entity_type: row.entity_type,
        entity_id: row.entity_id,
        author: row.author,
        body: row.body,
        created_at: row.created_at,
    }
}

pub async fn list_activity(limit: i64) -> AppResult<Vec<ActivityNote>> {
    let db = database()?;
    let capped = limit.clamp(1, 500);
    let limit_i32 = i32::try_from(capped).unwrap_or(100);
    let limit_arg = D1Type::Integer(limit_i32);

    let result = db
        .prepare(
            "SELECT id, entity_type, entity_id, author, body,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at
             FROM activity_notes
             ORDER BY id DESC
             LIMIT ?1",
        )
        .bind_refs(&limit_arg)
        .map_err(|e| d1_error("Failed to bind activity list.", e))?
        .all()
        .await
        .map_err(|e| d1_error("Failed to list activity notes.", e))?;

    Ok(result
        .results::<NoteRow>()
        .map_err(|e| d1_error("Failed to deserialize activity rows.", e))?
        .into_iter()
        .map(map)
        .collect())
}

pub async fn create_note(
    entity_type: String,
    entity_id: Option<i64>,
    author: String,
    body: String,
) -> AppResult<ActivityNote> {
    let db = database()?;
    let entity_type = validate_allowed("Entity type", entity_type.trim(), ALLOWED_ENTITIES)?
        .to_string();
    let author = normalize_text("Author", &author, 80, true)?
        .ok_or_else(|| AppError::client("Author is required."))?;
    let body = normalize_text("Body", &body, 2_000, true)?
        .ok_or_else(|| AppError::client("Note body is required."))?;

    if entity_type != "system" && entity_id.is_none() {
        return Err(AppError::client(
            "Entity id is required unless entity type is 'system'.",
        ));
    }

    let entity_id_arg = match entity_id {
        Some(id) => row_id_arg(id)?,
        None => D1Type::Null,
    };

    let args = [
        D1Type::Text(entity_type.as_str()),
        entity_id_arg,
        D1Type::Text(author.as_str()),
        D1Type::Text(body.as_str()),
    ];

    let result = db
        .prepare(
            "INSERT INTO activity_notes (entity_type, entity_id, author, body)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind activity insert.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to insert activity note.", e))?;

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

    let id_arg = row_id_arg(inserted_id)?;
    let row = db
        .prepare(
            "SELECT id, entity_type, entity_id, author, body,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at
             FROM activity_notes
             WHERE id = ?1",
        )
        .bind_refs(&id_arg)
        .map_err(|e| d1_error("Failed to bind activity lookup.", e))?
        .first::<NoteRow>(None)
        .await
        .map_err(|e| d1_error("Failed to fetch activity note.", e))?;
    row.map(map)
        .ok_or_else(|| AppError::client("Activity note was not found after insert."))
}
