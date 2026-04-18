use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{
    list_resources, CreateResource, HousingResource, ResourcesResponse, SetResourceStatus,
};
use crate::components::layout::{
    humanize, status_pill_class, EmptyState, ErrorBanner, PageHeader, RequiredMark, TopNav,
};

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
                    <div class="form-row form-row--span-8">
                        <label for="rs-label">"Label"<RequiredMark/></label>
                        <input id="rs-label" type="text" required
                            placeholder="e.g. Harry St Shelter — Bed 12"
                            prop:value=move || label.get()
                            on:input=move |ev| label.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--span-4">
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
                    <div class="form-row form-row--span-3">
                        <label for="rs-capacity">"Capacity"</label>
                        <input id="rs-capacity" type="number" min="1" max="1000"
                            prop:value=move || capacity.get().to_string()
                            on:input=move |ev| capacity.set(event_target_value(&ev).parse().unwrap_or(1))/>
                    </div>
                    <div class="form-row form-row--span-9">
                        <label for="rs-address">"Address"</label>
                        <input id="rs-address" type="text" placeholder="Street, city"
                            prop:value=move || address.get()
                            on:input=move |ev| address.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="rs-notes">"Notes"</label>
                        <textarea id="rs-notes" rows="2"
                            placeholder="Access info, restrictions, provider contact…"
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

            <Suspense fallback=|| view! { <p class="loading">"Loading inventory…"</p> }>
                {move || data.get().map(|res| match res {
                    Err(error) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                    Ok(response) => view! { <InventoryTable response=response status_action=status_action/> }.into_any(),
                })}
            </Suspense>
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
        <section class="panel panel--flush">
            <div class="panel-head">
                <div>
                    <h2>"Housing resources"</h2>
                    <p>{summary}</p>
                </div>
            </div>
            {if items.is_empty() {
                view! {
                    <EmptyState
                        title="No inventory yet"
                        body="Add a housing resource above — shelter bed, transitional unit, or permanent supportive."
                    />
                }.into_any()
            } else {
                view! {
                    <div class="data-table-scroll">
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
                    </div>
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

    let current_for_row = current_status.clone();
    let row_class = move || {
        let mut c = String::new();
        if status_action.pending().get() {
            c.push_str("row--pending");
        } else if status_sig.get() != current_for_row {
            c.push_str("row--dirty");
        }
        c
    };

    let current_for_btn = current_status.clone();
    let pill_class = status_pill_class(&status);
    let status_label = humanize(&status);

    view! {
        <tr class=row_class>
            <td>
                <div class="cell-title">
                    <span class="strong">{label}</span>
                    <span class="id-chip">"#"{id}</span>
                </div>
            </td>
            <td class="muted">{humanize(&kind)}</td>
            <td>{capacity}</td>
            <td class="muted">{address.unwrap_or_else(|| "—".into())}</td>
            <td>
                <span class=pill_class>{status_label}</span>
            </td>
            <td class="row-actions">
                <select
                    prop:value=move || status_sig.get()
                    attr:data-status=move || status_sig.get()
                    on:change=move |ev| status_sig.set(event_target_value(&ev))>
                    {STATUSES.iter().map(|s| {
                        let label = humanize(s);
                        view! { <option value=*s>{label}</option> }
                    }).collect::<Vec<_>>()}
                </select>
                <button class="secondary compact save-btn"
                    disabled=move || status_action.pending().get() || status_sig.get() == current_for_btn
                    on:click=move |_| {
                        status_action.dispatch(SetResourceStatus { id, status: status_sig.get_untracked() });
                    }>
                    {move || if status_action.pending().get() { "Saving…" } else { "Save" }}
                </button>
            </td>
        </tr>
    }
}
