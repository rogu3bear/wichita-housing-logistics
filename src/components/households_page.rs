use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{
    list_households, CreateHousehold, Household, HouseholdsResponse, RotateShareToken,
    SetHouseholdStage,
};
use crate::components::layout::{
    humanize, status_pill_class, EmptyState, ErrorBanner, PageHeader, TopNav,
};

const STAGES: &[&str] = &["intake", "assessment", "placement", "follow_up", "exited"];

/// Admin-side share-link chip: short id + copy + rotate. The raw anchor
/// lets right-click-copy-link work; the copy button handles phones where
/// right-click doesn't exist; the rotate button invalidates a compromised
/// link and surfaces the new one.
#[component]
fn ShareLinkCell(hh_id: i64, initial_token: Option<String>) -> impl IntoView {
    let token = RwSignal::new(initial_token);
    let rotate_action = ServerAction::<RotateShareToken>::new();

    Effect::new(move |_| {
        if let Some(Ok(fresh)) = rotate_action.value().get() {
            token.set(Some(fresh));
        }
    });

    let on_copy = move |_| {
        let Some(current) = token.get_untracked() else {
            return;
        };
        let origin = leptos::web_sys::window()
            .map(|w| w.location().origin().unwrap_or_default())
            .unwrap_or_default();
        let url = format!("{origin}/case/{current}");
        if let Some(clipboard) = leptos::web_sys::window().map(|w| w.navigator().clipboard()) {
            let _ = clipboard.write_text(&url);
        }
    };

    let on_rotate = move |_| {
        rotate_action.dispatch(RotateShareToken { id: hh_id });
    };

    view! {
        <div class="share-cell">
            {move || token.get().map(|t| {
                let href = format!("/case/{t}");
                let short = t.chars().take(8).collect::<String>();
                view! {
                    <a class="share-copy mono small"
                       href=href.clone()
                       target="_blank"
                       rel="noopener noreferrer"
                       title=format!("Open {href}")>
                        {short}
                    </a>
                }
            })}
            <button type="button" class="ghost compact"
                on:click=on_copy
                title="Copy the full /case/<token> URL to clipboard">
                "Copy"
            </button>
            <button type="button" class="ghost compact"
                on:click=on_rotate
                disabled=move || rotate_action.pending().get()
                title="Generate a fresh link. The old one stops working.">
                {move || if rotate_action.pending().get() { "…" } else { "New" }}
            </button>
        </div>
    }
}

#[component]
pub fn HouseholdsPage() -> impl IntoView {
    let create_action = ServerAction::<CreateHousehold>::new();
    let stage_action = ServerAction::<SetHouseholdStage>::new();

    let data = Resource::new(
        move || (create_action.version().get(), stage_action.version().get()),
        |_| async move { list_households().await },
    );

    let head_name = RwSignal::new(String::new());
    let household_size = RwSignal::new(1_i64);
    let phone = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let intake_notes = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            head_name.set(String::new());
            household_size.set(1);
            phone.set(String::new());
            email.set(String::new());
            intake_notes.set(String::new());
            form_error.set(None);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name = head_name.get_untracked().trim().to_string();
        if name.is_empty() {
            form_error.set(Some("Head of household name is required.".into()));
            return;
        }
        form_error.set(None);
        create_action.dispatch(CreateHousehold {
            head_name: name,
            household_size: household_size.get_untracked(),
            phone: phone.get_untracked(),
            email: email.get_untracked(),
            intake_notes: intake_notes.get_untracked(),
        });
    };

    let server_error = move || {
        create_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
            .or_else(|| {
                stage_action
                    .value()
                    .get()
                    .and_then(|r| r.err().map(|e| e.to_string()))
            })
    };

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Households"
                subtitle="Track households through intake, assessment, placement, and follow-up."
            />

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <ErrorBanner message=form_error.get().or_else(server_error).unwrap_or_default()/>
            </Show>

            <section class="panel">
                <form class="form-grid" on:submit=on_submit>
                    <div class="form-row form-row--span-8">
                        <label for="hh-name">"Head of household"</label>
                        <input id="hh-name" type="text" required placeholder="First Last"
                            prop:value=move || head_name.get()
                            on:input=move |ev| head_name.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--span-4">
                        <label for="hh-size">"Size"</label>
                        <input id="hh-size" type="number" min="1" max="32"
                            prop:value=move || household_size.get().to_string()
                            on:input=move |ev| {
                                household_size.set(event_target_value(&ev).parse().unwrap_or(1));
                            }/>
                        <p class="form-hint">"Include every person at the address."</p>
                    </div>
                    <div class="form-row form-row--span-6">
                        <label for="hh-phone">"Phone"</label>
                        <input id="hh-phone" type="text" placeholder="316-555-0000"
                            prop:value=move || phone.get()
                            on:input=move |ev| phone.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--span-6">
                        <label for="hh-email">"Email"</label>
                        <input id="hh-email" type="email" placeholder="name@example.org"
                            prop:value=move || email.get()
                            on:input=move |ev| email.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="hh-notes">"Intake notes"</label>
                        <textarea id="hh-notes" rows="2"
                            placeholder="Presenting situation, referral source, urgency…"
                            prop:value=move || intake_notes.get()
                            on:input=move |ev| intake_notes.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-actions">
                        <span class="form-hint">"Name is required. Everything else is optional at intake."</span>
                        <button class="primary" type="submit"
                            disabled=move || create_action.pending().get()>
                            {move || if create_action.pending().get() { "Saving…" } else { "Add household" }}
                        </button>
                    </div>
                </form>
            </section>

            {move || match data.get() {
                None => view! { <p class="loading">"Loading households…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(response)) => view! { <HouseholdsTable response=response stage_action=stage_action/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn HouseholdsTable(
    response: HouseholdsResponse,
    stage_action: ServerAction<SetHouseholdStage>,
) -> impl IntoView {
    let HouseholdsResponse { items, counts } = response;
    let stage_summary = format!(
        "Intake {} · Assessment {} · Placement {} · Follow-up {} · Exited {}",
        counts.intake, counts.assessment, counts.placement, counts.follow_up, counts.exited
    );

    view! {
        <section class="panel panel--flush">
            <div class="panel-head">
                <div>
                    <h2>"Household roster"</h2>
                    <p>{stage_summary}</p>
                </div>
            </div>
            {if items.is_empty() {
                view! {
                    <EmptyState
                        title="No households yet"
                        body="Add a household above to start the pipeline."
                    />
                }.into_any()
            } else {
                view! {
                    <div class="data-table-scroll">
                        <table class="data-table">
                            <thead>
                                <tr>
                                    <th>"Head"</th>
                                    <th>"Size"</th>
                                    <th>"Contact"</th>
                                    <th>"Stage"</th>
                                    <th>"Case link"</th>
                                    <th>"Updated"</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                {items.into_iter().map(|h| view! {
                                    <HouseholdRow household=h stage_action=stage_action/>
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
fn HouseholdRow(
    household: Household,
    stage_action: ServerAction<SetHouseholdStage>,
) -> impl IntoView {
    let Household {
        id,
        head_name,
        household_size,
        phone,
        email,
        stage,
        intake_notes,
        share_token,
        updated_at,
        ..
    } = household;
    let stage_sig = RwSignal::new(stage.clone());
    let current_stage = stage.clone();
    let contact = match (phone.as_deref(), email.as_deref()) {
        (Some(p), Some(e)) => format!("{p} · {e}"),
        (Some(p), None) => p.to_string(),
        (None, Some(e)) => e.to_string(),
        (None, None) => "—".to_string(),
    };

    let current_for_row = current_stage.clone();
    let row_class = move || {
        let mut c = String::new();
        if stage_action.pending().get() {
            c.push_str("row--pending");
        } else if stage_sig.get() != current_for_row {
            c.push_str("row--dirty");
        }
        c
    };

    let current_for_btn = current_stage.clone();
    let pill_class = status_pill_class(&stage);
    let stage_label = humanize(&stage);

    view! {
        <tr class=row_class>
            <td>
                <div class="strong">{head_name}</div>
                <div class="id-chip">"#"{id}</div>
                {intake_notes.map(|note| view! {
                    <div class="cell-note">{note}</div>
                })}
            </td>
            <td>{household_size}</td>
            <td class="muted">{contact}</td>
            <td>
                <span class=pill_class>{stage_label}</span>
            </td>
            <td>
                <ShareLinkCell hh_id=id initial_token=share_token/>
            </td>
            <td class="muted mono small">{updated_at}</td>
            <td class="row-actions">
                <select
                    prop:value=move || stage_sig.get()
                    attr:data-status=move || stage_sig.get()
                    on:change=move |ev| stage_sig.set(event_target_value(&ev))>
                    {STAGES.iter().map(|s| {
                        let label = humanize(s);
                        view! { <option value=*s>{label}</option> }
                    }).collect::<Vec<_>>()}
                </select>
                <button class="secondary compact save-btn"
                    disabled=move || stage_action.pending().get() || stage_sig.get() == current_for_btn
                    on:click=move |_| {
                        stage_action.dispatch(SetHouseholdStage { id, stage: stage_sig.get_untracked() });
                    }>
                    {move || if stage_action.pending().get() { "Saving…" } else { "Save" }}
                </button>
            </td>
        </tr>
    }
}
