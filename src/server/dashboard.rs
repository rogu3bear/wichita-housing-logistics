use serde::Deserialize;

use crate::api::{
    ActivityNote, DashboardSnapshot, PlacementStatusCounts, ResourceStatusCounts, StageCounts,
};

use super::{d1_error, database, AppError, AppResult};

#[derive(Debug, Deserialize)]
struct GroupCountRow {
    key: String,
    n: u32,
}

#[derive(Debug, Deserialize)]
struct ActivityRow {
    id: i64,
    entity_type: String,
    entity_id: Option<i64>,
    author: String,
    body: String,
    created_at: String,
}

/// Composite snapshot the dashboard renders on every GET /.
///
/// Executes four queries in a single D1 `batch` round-trip:
///   1. household stage counts (GROUP BY stage)
///   2. resource status counts (GROUP BY status)
///   3. placement status counts (GROUP BY status)
///   4. ten most-recent activity notes
///
/// Counts are aggregated server-side so the Worker never ships full row sets
/// just to compute numbers.
pub async fn snapshot() -> AppResult<DashboardSnapshot> {
    let db = database()?;

    let statements = vec![
        db.prepare(
            "SELECT stage AS key, COUNT(*) AS n FROM households GROUP BY stage",
        ),
        db.prepare(
            "SELECT status AS key, COUNT(*) AS n FROM housing_resources GROUP BY status",
        ),
        db.prepare(
            "SELECT status AS key, COUNT(*) AS n FROM placements GROUP BY status",
        ),
        db.prepare(
            "SELECT id, entity_type, entity_id, author, body,
                    strftime('%Y-%m-%d %H:%M UTC', created_at) AS created_at
             FROM activity_notes
             ORDER BY id DESC
             LIMIT 10",
        ),
    ];

    let results = db
        .batch(statements)
        .await
        .map_err(|e| d1_error("Failed to batch dashboard queries.", e))?;

    if results.len() != 4 {
        return Err(AppError::internal(
            "Dashboard batch returned unexpected number of results.",
            format!("expected 4, got {}", results.len()),
        ));
    }

    let household_rows: Vec<GroupCountRow> = results[0]
        .results()
        .map_err(|e| d1_error("Failed to deserialize household stage counts.", e))?;
    let resource_rows: Vec<GroupCountRow> = results[1]
        .results()
        .map_err(|e| d1_error("Failed to deserialize resource status counts.", e))?;
    let placement_rows: Vec<GroupCountRow> = results[2]
        .results()
        .map_err(|e| d1_error("Failed to deserialize placement status counts.", e))?;
    let activity_rows: Vec<ActivityRow> = results[3]
        .results()
        .map_err(|e| d1_error("Failed to deserialize recent activity.", e))?;

    let mut households = StageCounts::default();
    let mut household_total = 0_u32;
    for row in household_rows {
        household_total = household_total.saturating_add(row.n);
        match row.key.as_str() {
            "intake" => households.intake = row.n,
            "assessment" => households.assessment = row.n,
            "placement" => households.placement = row.n,
            "follow_up" => households.follow_up = row.n,
            "exited" => households.exited = row.n,
            _ => {}
        }
    }

    let mut resources = ResourceStatusCounts::default();
    let mut resource_total = 0_u32;
    for row in resource_rows {
        resource_total = resource_total.saturating_add(row.n);
        match row.key.as_str() {
            "available" => resources.available = row.n,
            "held" => resources.held = row.n,
            "occupied" => resources.occupied = row.n,
            "offline" => resources.offline = row.n,
            _ => {}
        }
    }

    let mut placements = PlacementStatusCounts::default();
    let mut placement_total = 0_u32;
    for row in placement_rows {
        placement_total = placement_total.saturating_add(row.n);
        match row.key.as_str() {
            "proposed" => placements.proposed = row.n,
            "confirmed" => placements.confirmed = row.n,
            "moved_in" => placements.moved_in = row.n,
            "exited" => placements.exited = row.n,
            "cancelled" => placements.cancelled = row.n,
            _ => {}
        }
    }

    let recent_activity = activity_rows
        .into_iter()
        .map(|r| ActivityNote {
            id: r.id,
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            author: r.author,
            body: r.body,
            created_at: r.created_at,
        })
        .collect();

    Ok(DashboardSnapshot {
        households,
        household_total,
        resources,
        resource_total,
        placements,
        placement_total,
        recent_activity,
    })
}
