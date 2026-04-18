use leptos::prelude::*;

use crate::api::{dashboard_snapshot, DashboardSnapshot};
use crate::components::layout::{
    entity_pill_class, humanize, EmptyState, ErrorBanner, PageHeader, TopNav,
};

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
            <Suspense fallback=|| view! { <p class="loading">"Loading dashboard…"</p> }>
                {move || snapshot.get().map(|res| match res {
                    Err(error) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                    Ok(data) => view! { <DashboardBoard data=data/> }.into_any(),
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn DashboardBoard(data: DashboardSnapshot) -> impl IntoView {
    let DashboardSnapshot {
        households,
        household_total: _,
        resources,
        resource_total,
        placements,
        placement_total,
        recent_activity,
    } = data;

    // Active pipeline = everyone not yet exited. Percentages compute against this
    // so "Intake 27%" reads as "27% of the active pipeline" rather than "27% of everyone
    // we've ever touched." Use a floor of 1 to avoid divide-by-zero on an empty system.
    let pipeline_total = households.intake
        + households.assessment
        + households.placement
        + households.follow_up;
    let denom = pipeline_total.max(1) as f32;
    let pct = |n: u32| -> u32 { ((n as f32 / denom) * 100.0).round() as u32 };

    let p_intake = pct(households.intake);
    let p_assess = pct(households.assessment);
    let p_place = pct(households.placement);
    let p_follow = pct(households.follow_up);

    let pipeline_heading = format!("Pipeline · {pipeline_total} households");

    view! {
        <section class="metric-group">
            <h2 class="section-title">{pipeline_heading}</h2>
            <div class="metric-row metric-row--pipeline">
                <PipelineCard label="Intake" value=households.intake pct=p_intake/>
                <PipelineCard label="Assessment" value=households.assessment pct=p_assess/>
                <PipelineCard label="Placement" value=households.placement pct=p_place/>
                <PipelineCard label="Follow up" value=households.follow_up pct=p_follow/>
            </div>
        </section>

        <section class="metric-group">
            <h2 class="section-title">"Lifecycle terminals"</h2>
            <div class="metric-row metric-row--terminal">
                <MetricCard label="Exited" value=households.exited/>
                <MetricCard label="Cancelled placements" value=placements.cancelled/>
                <MetricCard label="Moved in" value=placements.moved_in/>
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
                view! {
                    <EmptyState
                        title="No activity yet"
                        body="Start recording intake decisions — the audit trail is how follow-ups find context six months from now."
                    />
                }.into_any()
            } else {
                view! {
                    <ul class="activity-list">
                        {recent_activity.into_iter().map(|note| {
                            let pill_class = entity_pill_class(&note.entity_type);
                            let entity_label = humanize(&note.entity_type);
                            view! {
                                <li class="activity-row">
                                    <div class="activity-meta">
                                        <span class=pill_class>{entity_label}</span>
                                        {note.entity_id.map(|id| view! {
                                            <span class="id-chip">"#"{id}</span>
                                        })}
                                        <time>{note.created_at.clone()}</time>
                                    </div>
                                    <p class="activity-body">{note.body.clone()}</p>
                                    <p class="activity-author">{"— "}{humanize(&note.author)}</p>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn PipelineCard(#[prop(into)] label: String, value: u32, pct: u32) -> impl IntoView {
    let style = format!("--metric-fill: {pct}%");
    let sub = format!("{pct}% of pipeline");
    view! {
        <article class="metric-card metric-card--primary" style=style>
            <span class="metric-label">{label}</span>
            <strong class="metric-value">{value}</strong>
            <span class="metric-sub">{sub}</span>
        </article>
    }
}

#[component]
fn MetricCard(#[prop(into)] label: String, value: u32) -> impl IntoView {
    view! {
        <article class="metric-card">
            <span class="metric-label">{label}</span>
            <strong class="metric-value">{value}</strong>
        </article>
    }
}
