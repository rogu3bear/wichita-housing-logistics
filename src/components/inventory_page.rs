use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{
    list_resources, CreateResource, HousingResource, ResourcesResponse, SetResourceStatus,
};
use crate::components::layout::{humanize, ErrorBanner, PageHeader, TopNav};

const KINDS: &[&str] = &[
    "shelter_bed",
    "transitional",
    "permanent_supportive",
    "rental_unit",
    "other",
];
const STATUSES: &[&str] = &["available", "held", "occupied", "offline"];

#[component]
pub fn InventoryPage() -> impl IntoView {
    let create_action = ServerAction::<CreateResource>::new();
    let status_action = ServerAction::<SetResourceStatus>::new();

    let data = Resource::new(
        move || (create_action.version().get(), status_action.version().get()),
        |_| async move { list_resources().await },
    );

    let label = RwSignal::new(String::new());
    let kind = RwSignal::new(KINDS[0].to_string());
    let address = RwSignal::new(String::new());
    let capacity = RwSignal::new(1_i64);
    let notes = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            label.set(String::new());
            address.set(String::new());
            capacity.set(1);
            notes.set(String::new());
            form_error.set(None);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let l = label.get_untracked().trim().to_string();
        if l.is_empty() {
            form_error.set(Some("Label is required.".into()));
            return;
        }
        form_error.set(None);
        create_action.dispatch(CreateResource {
            label: l,
            kind: kind.get_untracked(),
            address: address.get_untracked(),
            capacity: capacity.get_untracked(),
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
                title="Inventory"
                subtitle="Beds, transitional units, and permanent supportive housing tracked through their lifecycle."
            />

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <ErrorBanner message=form_error.get().or_else(server_error).unwrap_or_default()/>
            </Show>

            <section class="panel">
                <form class="form-grid" on:submit=on_submit>
                    <div class="form-row form-row--wide">
                        <label for="rs-label">"Label"</label>
                        <input id="rs-label" type="text" required
                            prop:value=move || label.get()
                            on:input=move |ev| label.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row">
                        <label for="rs-kind">"Kind"</label>
                        <select id="rs-kind"
                            prop:value=move || kind.get()
                            on:change=move |ev| kind.set(event_target_value(&ev))>
                            {KINDS.iter().map(|k| {
                                let label = humanize(k);
                                view! { <option value=*k>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                    <div class="form-row">
                        <label for="rs-capacity">"Capacity"</label>
                        <input id="rs-capacity" type="number" min="1" max="1000"
                            prop:value=move || capacity.get().to_string()
                            on:input=move |ev| capacity.set(event_target_value(&ev).parse().unwrap_or(1))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="rs-address">"Address"</label>
                        <input id="rs-address" type="text"
                            prop:value=move || address.get()
                            on:input=move |ev| address.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="rs-notes">"Notes"</label>
                        <textarea id="rs-notes" rows="2"
                            prop:value=move || notes.get()
                            on:input=move |ev| notes.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-actions">
                        <button class="primary" type="submit"
                            disabled=move || create_action.pending().get()>
                            {move || if create_action.pending().get() { "Saving…" } else { "Add resource" }}
                        </button>
                    </div>
                </form>
            </section>

            {move || match data.get() {
                None => view! { <p class="loading">"Loading inventory…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(response)) => view! { <InventoryTable response=response status_action=status_action/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn InventoryTable(
    response: ResourcesResponse,
    status_action: ServerAction<SetResourceStatus>,
) -> impl IntoView {
    let ResourcesResponse { items, counts } = response;
    let summary = format!(
        "Available {} · Held {} · Occupied {} · Offline {}",
        counts.available, counts.held, counts.occupied, counts.offline
    );

    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Housing resources"</h2>
                <p class="muted">{summary}</p>
            </div>
            {if items.is_empty() {
                view! { <p class="muted">"No inventory yet."</p> }.into_any()
            } else {
                view! {
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>"Label"</th>
                                <th>"Kind"</th>
                                <th>"Capacity"</th>
                                <th>"Address"</th>
                                <th>"Status"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {items.into_iter().map(|r| view! {
                                <InventoryRow resource=r status_action=status_action/>
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn InventoryRow(
    resource: HousingResource,
    status_action: ServerAction<SetResourceStatus>,
) -> impl IntoView {
    let HousingResource {
        id,
        label,
        kind,
        address,
        capacity,
        status,
        ..
    } = resource;
    let status_sig = RwSignal::new(status.clone());
    let current_status = status.clone();

    view! {
        <tr>
            <td>
                <div class="strong">{label}</div>
                <div class="muted small">"#"{id}</div>
            </td>
            <td class="muted">{humanize(&kind)}</td>
            <td>{capacity}</td>
            <td class="muted">{address.unwrap_or_else(|| "—".into())}</td>
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
            <td class="row-actions">
                <button class="secondary"
                    disabled=move || status_action.pending().get() || status_sig.get() == current_status
                    on:click=move |_| {
                        status_action.dispatch(SetResourceStatus { id, status: status_sig.get_untracked() });
                    }>
                    "Save"
                </button>
            </td>
        </tr>
    }
}
