use leptos::prelude::*;

use crate::api::{dashboard_snapshot, DashboardSnapshot};
use crate::components::layout::{humanize, ErrorBanner, PageHeader, TopNav};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let snapshot = Resource::new(|| (), |_| async move { dashboard_snapshot().await });

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Operations dashboard"
                subtitle="Pipeline overview and the latest activity across Wichita housing logistics."
            />
            {move || match snapshot.get() {
                None => view! { <p class="loading">"Loading dashboard…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(data)) => view! { <DashboardBoard data=data/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn DashboardBoard(data: DashboardSnapshot) -> impl IntoView {
    let DashboardSnapshot {
        households,
        household_total,
        resources,
        resource_total,
        placements,
        placement_total,
        recent_activity,
    } = data;

    view! {
        <section class="metric-group">
            <h2 class="section-title">"Households"</h2>
            <div class="metric-row">
                <MetricCard label="Total" value=household_total/>
                <MetricCard label="Intake" value=households.intake/>
                <MetricCard label="Assessment" value=households.assessment/>
                <MetricCard label="Placement" value=households.placement/>
                <MetricCard label="Follow-up" value=households.follow_up/>
                <MetricCard label="Exited" value=households.exited/>
            </div>
        </section>

        <section class="metric-group">
            <h2 class="section-title">"Inventory"</h2>
            <div class="metric-row">
                <MetricCard label="Total" value=resource_total/>
                <MetricCard label="Available" value=resources.available/>
                <MetricCard label="Held" value=resources.held/>
                <MetricCard label="Occupied" value=resources.occupied/>
                <MetricCard label="Offline" value=resources.offline/>
            </div>
        </section>

        <section class="metric-group">
            <h2 class="section-title">"Placements"</h2>
            <div class="metric-row">
                <MetricCard label="Total" value=placement_total/>
                <MetricCard label="Proposed" value=placements.proposed/>
                <MetricCard label="Confirmed" value=placements.confirmed/>
                <MetricCard label="Moved in" value=placements.moved_in/>
                <MetricCard label="Exited" value=placements.exited/>
                <MetricCard label="Cancelled" value=placements.cancelled/>
            </div>
        </section>

        <section class="panel">
            <div class="panel-head">
                <h2>"Recent activity"</h2>
                <p>"Latest 10 entries from the audit trail."</p>
            </div>
            {if recent_activity.is_empty() {
                view! { <p class="muted">"No activity yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="activity-list">
                        {recent_activity.into_iter().map(|note| view! {
                            <li class="activity-row">
                                <div class="activity-meta">
                                    <span class="pill">{humanize(&note.entity_type)}</span>
                                    {note.entity_id.map(|id| view! { <span class="muted">"#"{id}</span> })}
                                    <time>{note.created_at.clone()}</time>
                                </div>
                                <p class="activity-body">{note.body.clone()}</p>
                                <p class="activity-author muted">{"— "}{note.author.clone()}</p>
                            </li>
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn MetricCard(#[prop(into)] label: String, value: usize) -> impl IntoView {
    view! {
        <article class="metric-card">
            <span class="metric-label">{label}</span>
            <strong class="metric-value">{value}</strong>
        </article>
    }
}
