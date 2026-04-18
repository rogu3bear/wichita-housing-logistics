use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{
    list_households, list_placements, list_resources, CreatePlacement, Household, HousingResource,
    Placement, PlacementsResponse, SetPlacementStatus,
};
use crate::components::layout::{humanize, ErrorBanner, PageHeader, TopNav};

const STATUSES: &[&str] = &["proposed", "confirmed", "moved_in", "exited", "cancelled"];

#[component]
pub fn PlacementsPage() -> impl IntoView {
    let create_action = ServerAction::<CreatePlacement>::new();
    let status_action = ServerAction::<SetPlacementStatus>::new();

    let placements = Resource::new(
        move || (create_action.version().get(), status_action.version().get()),
        |_| async move { list_placements().await },
    );

    // Pickers refresh when a placement is created (so in-flight inventory may change).
    let households = Resource::new(
        move || create_action.version().get(),
        |_| async move { list_households().await.map(|r| r.items) },
    );
    let resources = Resource::new(
        move || (create_action.version().get(), status_action.version().get()),
        |_| async move { list_resources().await.map(|r| r.items) },
    );

    let household_id = RwSignal::new(0_i64);
    let resource_id = RwSignal::new(0_i64);
    let notes = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            notes.set(String::new());
            form_error.set(None);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let hh = household_id.get_untracked();
        let rs = resource_id.get_untracked();
        if hh <= 0 || rs <= 0 {
            form_error.set(Some("Pick a household and a resource.".into()));
            return;
        }
        form_error.set(None);
        create_action.dispatch(CreatePlacement {
            household_id: hh,
            resource_id: rs,
            notes: notes.get_untracked(),
        });
    };

    let server_error = move || {
        create_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
            .or_else(|| {
                status_action
                    .value()
                    .get()
                    .and_then(|r| r.err().map(|e| e.to_string()))
            })
    };

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Placements"
                subtitle="Match households to housing resources and walk them through the placement lifecycle."
            />

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <ErrorBanner message=form_error.get().or_else(server_error).unwrap_or_default()/>
            </Show>

            <section class="panel">
                <form class="form-grid" on:submit=on_submit>
                    <div class="form-row form-row--wide">
                        <label for="pl-hh">"Household"</label>
                        <HouseholdSelect signal=household_id resource=households/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="pl-rs">"Resource"</label>
                        <ResourceSelect signal=resource_id resource=resources/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="pl-notes">"Notes"</label>
                        <textarea id="pl-notes" rows="2"
                            prop:value=move || notes.get()
                            on:input=move |ev| notes.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-actions">
                        <button class="primary" type="submit"
                            disabled=move || create_action.pending().get()>
                            {move || if create_action.pending().get() { "Saving…" } else { "Create placement" }}
                        </button>
                    </div>
                </form>
            </section>

            {move || match placements.get() {
                None => view! { <p class="loading">"Loading placements…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(response)) => view! { <PlacementsTable response=response status_action=status_action/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn HouseholdSelect(
    signal: RwSignal<i64>,
    resource: Resource<Result<Vec<Household>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <select id="pl-hh"
            prop:value=move || signal.get().to_string()
            on:change=move |ev| signal.set(event_target_value(&ev).parse().unwrap_or(0))>
            <option value="0">"— Select household —"</option>
            {move || match resource.get() {
                Some(Ok(items)) => items.into_iter().map(|h| {
                    let id = h.id;
                    let label = format!("#{id} · {} (size {})", h.head_name, h.household_size);
                    view! { <option value=id.to_string()>{label}</option> }
                }).collect::<Vec<_>>(),
                _ => Vec::new(),
            }}
        </select>
    }
}

#[component]
fn ResourceSelect(
    signal: RwSignal<i64>,
    resource: Resource<Result<Vec<HousingResource>, ServerFnError>>,
) -> impl IntoView {
    view! {
        <select id="pl-rs"
            prop:value=move || signal.get().to_string()
            on:change=move |ev| signal.set(event_target_value(&ev).parse().unwrap_or(0))>
            <option value="0">"— Select resource —"</option>
            {move || match resource.get() {
                Some(Ok(items)) => items.into_iter().map(|r| {
                    let id = r.id;
                    let label = format!("#{id} · {} [{}]", r.label, humanize(&r.status));
                    view! { <option value=id.to_string()>{label}</option> }
                }).collect::<Vec<_>>(),
                _ => Vec::new(),
            }}
        </select>
    }
}

#[component]
fn PlacementsTable(
    response: PlacementsResponse,
    status_action: ServerAction<SetPlacementStatus>,
) -> impl IntoView {
    let PlacementsResponse { items, counts } = response;
    let summary = format!(
        "Proposed {} · Confirmed {} · Moved in {} · Exited {} · Cancelled {}",
        counts.proposed, counts.confirmed, counts.moved_in, counts.exited, counts.cancelled
    );

    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Placement board"</h2>
                <p class="muted">{summary}</p>
            </div>
            {if items.is_empty() {
                view! { <p class="muted">"No placements yet."</p> }.into_any()
            } else {
                view! {
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Household"</th>
                                <th>"Resource"</th>
                                <th>"Status"</th>
                                <th>"Started"</th>
                                <th>"Ended"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {items.into_iter().map(|p| view! {
                                <PlacementRow placement=p status_action=status_action/>
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn PlacementRow(
    placement: Placement,
    status_action: ServerAction<SetPlacementStatus>,
) -> impl IntoView {
    let Placement {
        id,
        head_name,
        resource_label,
        status,
        started_at,
        ended_at,
        ..
    } = placement;
    let status_sig = RwSignal::new(status.clone());
    let current_status = status.clone();

    view! {
        <tr>
            <td>
                <div class="strong">{head_name}</div>
                <div class="muted small">"placement #"{id}</div>
            </td>
            <td class="muted">{resource_label}</td>
            <td>
                <select
                    prop:value=move || status_sig.get()
                    on:change=move |ev| status_sig.set(event_target_value(&ev))>
                    {STATUSES.iter().map(|s| {
                        let label = humanize(s);
                        view! { <option value=*s>{label}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            </td>
            <td class="muted">{started_at.unwrap_or_else(|| "—".into())}</td>
            <td class="muted">{ended_at.unwrap_or_else(|| "—".into())}</td>
            <td class="row-actions">
                <button class="secondary"
                    disabled=move || status_action.pending().get() || status_sig.get() == current_status
                    on:click=move |_| {
                        status_action.dispatch(SetPlacementStatus { id, status: status_sig.get_untracked() });
                    }>
                    "Save"
                </button>
            </td>
        </tr>
    }
}
