use serde::Deserialize;
use worker::D1Type;

use crate::api::Sitrep;

use super::{d1_error, database, normalize_text, validate_allowed, AppError, AppResult};

pub const ALLOWED_LEVELS: &[&str] = &["warn", "red"];

#[derive(Debug, Deserialize)]
struct SitrepRow {
    active: i64,
    summary: String,
    level: String,
    started_at: Option<String>,
    updated_at: String,
    updated_by: Option<String>,
}

fn map(row: SitrepRow) -> Sitrep {
    Sitrep {
        active: row.active != 0,
        summary: row.summary,
        level: row.level,
        started_at: row.started_at,
        updated_at: row.updated_at,
        updated_by: row.updated_by,
    }
}

pub async fn get_sitrep() -> AppResult<Sitrep> {
    let db = database()?;
    let row = db
        .prepare(
            "SELECT active, summary, level,
                    strftime('%Y-%m-%d %H:%M UTC', started_at) AS started_at,
                    strftime('%Y-%m-%d %H:%M UTC', updated_at) AS updated_at,
                    updated_by
             FROM sitrep
             WHERE id = 1",
        )
        .first::<SitrepRow>(None)
        .await
        .map_err(|e| d1_error("Failed to fetch sitrep.", e))?;
    // Row 1 is seeded at migration time; its absence is an internal error.
    row.map(map)
        .ok_or_else(|| AppError::internal("sitrep row 1 missing", "table not seeded"))
}

pub async fn set_sitrep(
    active: bool,
    summary: String,
    level: String,
    updated_by: String,
) -> AppResult<Sitrep> {
    let db = database()?;
    let level = validate_allowed("Level", level.trim(), ALLOWED_LEVELS)?.to_string();
    // Summary can be empty when deactivating; when active, require at
    // least a short message so the banner isn't a red smear with nothing.
    let summary = summary.trim().to_string();
    if active && summary.is_empty() {
        return Err(AppError::client(
            "An active sitrep needs a summary — narrate what's happening.",
        ));
    }
    if summary.chars().count() > 240 {
        return Err(AppError::client(
            "Summary is capped at 240 characters — the banner is a tease, not a memo.",
        ));
    }
    let updated_by = normalize_text("Updated by", &updated_by, 80, false)?;

    let active_i32: i32 = if active { 1 } else { 0 };
    let started_sql = if active {
        "COALESCE(started_at, CURRENT_TIMESTAMP)"
    } else {
        "NULL"
    };

    let args = [
        D1Type::Integer(active_i32),
        D1Type::Text(summary.as_str()),
        D1Type::Text(level.as_str()),
        match updated_by.as_deref() {
            Some(s) => D1Type::Text(s),
            None => D1Type::Null,
        },
    ];

    let sql = format!(
        "UPDATE sitrep
         SET active = ?1,
             summary = ?2,
             level = ?3,
             started_at = {started_sql},
             updated_at = CURRENT_TIMESTAMP,
             updated_by = ?4
         WHERE id = 1"
    );

    db.prepare(&sql)
        .bind_refs(&args)
        .map_err(|e| d1_error("Failed to bind sitrep update.", e))?
        .run()
        .await
        .map_err(|e| d1_error("Failed to update sitrep.", e))?;

    get_sitrep().await
}
