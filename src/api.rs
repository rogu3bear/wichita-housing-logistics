use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// --- Shared wire types ---------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Household {
    pub id: i64,
    pub head_name: String,
    pub household_size: i64,
    pub phone: Option<String>,
    pub email: Option<String>,
    /// One of: intake | assessment | placement | follow_up | exited
    pub stage: String,
    pub intake_notes: Option<String>,
    /// 24-char hex, unguessable. Pair with `/case/{share_token}` to share a
    /// customer-facing case page. `None` on legacy rows pre-migration.
    pub share_token: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Customer-facing view of a single household. Stripped of audit fields,
/// ids, and cross-household context — only what the household themselves
/// should see on `/case/{share_token}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseView {
    pub head_name: String,
    pub household_size: i64,
    /// Same vocabulary as `Household::stage`; UI translates to plain language.
    pub stage: String,
    pub intake_notes: Option<String>,
    pub current_placement: Option<CasePlacement>,
    pub recent_updates: Vec<CaseUpdate>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CasePlacement {
    pub resource_label: String,
    /// proposed | confirmed | moved_in | exited | cancelled
    pub status: String,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaseUpdate {
    pub body: String,
    /// true when the household posted this themselves via the /case page.
    pub author_is_household: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageCounts {
    pub intake: u32,
    pub assessment: u32,
    pub placement: u32,
    pub follow_up: u32,
    pub exited: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HouseholdsResponse {
    pub items: Vec<Household>,
    pub counts: StageCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HousingResource {
    pub id: i64,
    pub label: String,
    /// One of: shelter_bed | transitional | permanent_supportive | rental_unit | other
    pub kind: String,
    pub address: Option<String>,
    pub capacity: i64,
    /// One of: available | held | occupied | offline
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceStatusCounts {
    pub available: u32,
    pub held: u32,
    pub occupied: u32,
    pub offline: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourcesResponse {
    pub items: Vec<HousingResource>,
    pub counts: ResourceStatusCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Placement {
    pub id: i64,
    pub household_id: i64,
    pub resource_id: i64,
    pub head_name: String,
    pub resource_label: String,
    /// One of: proposed | confirmed | moved_in | exited | cancelled
    pub status: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlacementStatusCounts {
    pub proposed: u32,
    pub confirmed: u32,
    pub moved_in: u32,
    pub exited: u32,
    pub cancelled: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlacementsResponse {
    pub items: Vec<Placement>,
    pub counts: PlacementStatusCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivityNote {
    pub id: i64,
    /// One of: household | resource | placement | system
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub author: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardSnapshot {
    pub households: StageCounts,
    pub household_total: u32,
    pub resources: ResourceStatusCounts,
    pub resource_total: u32,
    pub placements: PlacementStatusCounts,
    pub placement_total: u32,
    pub recent_activity: Vec<ActivityNote>,
}

// --- Server functions ----------------------------------------------------
//
// Each #[server] wraps the SSR-only query layer inside a SendWrapper because
// Leptos server fns require Send but worker::Env is !Send on Workers.

#[allow(unused_macros)]
macro_rules! ssr_call {
    ($body:expr) => {{
        #[cfg(feature = "ssr")]
        {
            send_wrapper::SendWrapper::new(async move { $body.map_err(crate::server::server_error) })
                .await
        }

        #[cfg(not(feature = "ssr"))]
        {
            unreachable!("server functions only execute on the server")
        }
    }};
}

#[server(DashboardSnapshotFn)]
pub async fn dashboard_snapshot() -> Result<DashboardSnapshot, ServerFnError> {
    ssr_call!(crate::server::dashboard::snapshot().await)
}

#[server(ListHouseholds)]
pub async fn list_households() -> Result<HouseholdsResponse, ServerFnError> {
    ssr_call!(crate::server::households::list_households().await)
}

#[server(CreateHousehold)]
pub async fn create_household(
    head_name: String,
    household_size: i64,
    phone: String,
    email: String,
    intake_notes: String,
) -> Result<Household, ServerFnError> {
    let phone = empty_to_none(phone);
    let email = empty_to_none(email);
    let notes = empty_to_none(intake_notes);
    ssr_call!(
        crate::server::households::create_household(head_name, household_size, phone, email, notes)
            .await
    )
}

#[server(SetHouseholdStage)]
pub async fn set_household_stage(id: i64, stage: String) -> Result<Household, ServerFnError> {
    ssr_call!(crate::server::households::set_stage(id, stage).await)
}

#[server(CaseViewFn)]
pub async fn case_view(token: String) -> Result<CaseView, ServerFnError> {
    ssr_call!(crate::server::households::case_view(token).await)
}

#[server(SubmitHouseholdUpdate)]
pub async fn submit_household_update(
    token: String,
    body: String,
) -> Result<(), ServerFnError> {
    ssr_call!(crate::server::households::submit_household_update(token, body).await)
}

#[server(ListResources)]
pub async fn list_resources() -> Result<ResourcesResponse, ServerFnError> {
    ssr_call!(crate::server::resources::list_resources().await)
}

#[server(CreateResource)]
pub async fn create_resource(
    label: String,
    kind: String,
    address: String,
    capacity: i64,
    notes: String,
) -> Result<HousingResource, ServerFnError> {
    let address = empty_to_none(address);
    let notes = empty_to_none(notes);
    ssr_call!(
        crate::server::resources::create_resource(label, kind, address, capacity, notes).await
    )
}

#[server(SetResourceStatus)]
pub async fn set_resource_status(id: i64, status: String) -> Result<HousingResource, ServerFnError> {
    ssr_call!(crate::server::resources::set_status(id, status).await)
}

#[server(ListPlacements)]
pub async fn list_placements() -> Result<PlacementsResponse, ServerFnError> {
    ssr_call!(crate::server::placements::list_placements().await)
}

#[server(CreatePlacement)]
pub async fn create_placement(
    household_id: i64,
    resource_id: i64,
    notes: String,
) -> Result<Placement, ServerFnError> {
    let notes = empty_to_none(notes);
    ssr_call!(crate::server::placements::create_placement(household_id, resource_id, notes).await)
}

#[server(SetPlacementStatus)]
pub async fn set_placement_status(id: i64, status: String) -> Result<Placement, ServerFnError> {
    ssr_call!(crate::server::placements::set_status(id, status).await)
}

#[server(ListActivity)]
pub async fn list_activity(limit: i64) -> Result<Vec<ActivityNote>, ServerFnError> {
    ssr_call!(crate::server::activity::list_activity(limit).await)
}

#[server(CreateNote)]
pub async fn create_note(
    entity_type: String,
    entity_id: Option<i64>,
    author: String,
    body: String,
) -> Result<ActivityNote, ServerFnError> {
    ssr_call!(crate::server::activity::create_note(entity_type, entity_id, author, body).await)
}

#[allow(dead_code)]
fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
