use crate::api::DashboardSnapshot;

use super::{activity, households, placements, resources, AppResult};

pub async fn snapshot() -> AppResult<DashboardSnapshot> {
    let households = households::list_households().await?;
    let resources = resources::list_resources().await?;
    let placements = placements::list_placements().await?;
    let recent_activity = activity::list_activity(10).await?;

    Ok(DashboardSnapshot {
        households: households.counts,
        household_total: households.items.len(),
        resources: resources.counts,
        resource_total: resources.items.len(),
        placements: placements.counts,
        placement_total: placements.items.len(),
        recent_activity,
    })
}
