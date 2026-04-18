use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{list_activity, ActivityNote, CreateNote};
use crate::components::layout::{humanize, ErrorBanner, PageHeader, TopNav};

const ENTITIES: &[&str] = &["household", "resource", "placement", "system"];

#[component]
pub fn ActivityPage() -> impl IntoView {
    let create_action = ServerAction::<CreateNote>::new();
    let data = Resource::new(
        move || create_action.version().get(),
        |_| async move { list_activity(100).await },
    );

    let entity_type = RwSignal::new(ENTITIES[0].to_string());
    let entity_id = RwSignal::new(String::new());
    let author = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_action.value().get() {
            entity_id.set(String::new());
            body.set(String::new());
            form_error.set(None);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let kind = entity_type.get_untracked();
        let id_raw = entity_id.get_untracked();
        let id_parsed = if kind == "system" {
            None
        } else {
            match id_raw.trim().parse::<i64>() {
                Ok(n) if n > 0 => Some(n),
                _ => {
                    form_error.set(Some(format!("Entity id required for {kind}.")));
                    return;
                }
            }
        };
        let who = author.get_untracked().trim().to_string();
        let msg = body.get_untracked().trim().to_string();
        if who.is_empty() || msg.is_empty() {
            form_error.set(Some("Author and body are required.".into()));
            return;
        }
        form_error.set(None);
        create_action.dispatch(CreateNote {
            entity_type: kind,
            entity_id: id_parsed,
            author: who,
            body: msg,
        });
    };

    let server_error = move || {
        create_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
    };

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Activity"
                subtitle="Audit trail across households, resources, placements, and system events."
            />

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <ErrorBanner message=form_error.get().or_else(server_error).unwrap_or_default()/>
            </Show>

            <section class="panel">
                <form class="form-grid" on:submit=on_submit>
                    <div class="form-row">
                        <label for="ac-type">"About"</label>
                        <select id="ac-type"
                            prop:value=move || entity_type.get()
                            on:change=move |ev| entity_type.set(event_target_value(&ev))>
                            {ENTITIES.iter().map(|e| {
                                let label = humanize(e);
                                view! { <option value=*e>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                    <div class="form-row">
                        <label for="ac-id">"Entity id"</label>
                        <input id="ac-id" type="text" placeholder="e.g. 3 (skip for system)"
                            prop:value=move || entity_id.get()
                            on:input=move |ev| entity_id.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row">
                        <label for="ac-author">"Author"</label>
                        <input id="ac-author" type="text" required placeholder="case_manager_kim"
                            prop:value=move || author.get()
                            on:input=move |ev| author.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="ac-body">"Note"</label>
                        <textarea id="ac-body" rows="2" required
                            prop:value=move || body.get()
                            on:input=move |ev| body.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-actions">
                        <button class="primary" type="submit"
                            disabled=move || create_action.pending().get()>
                            {move || if create_action.pending().get() { "Saving…" } else { "Add note" }}
                        </button>
                    </div>
                </form>
            </section>

            {move || match data.get() {
                None => view! { <p class="loading">"Loading activity…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(notes)) => view! { <ActivityFeed notes=notes/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn ActivityFeed(notes: Vec<ActivityNote>) -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Activity feed"</h2>
                <p class="muted">"Latest 100 entries."</p>
            </div>
            {if notes.is_empty() {
                view! { <p class="muted">"No activity yet."</p> }.into_any()
            } else {
                view! {
                    <ul class="activity-list">
                        {notes.into_iter().map(|note| view! {
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
