use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{list_activity, ActivityNote, CreateNote};
use crate::components::layout::{ErrorBanner, PageHeader, TopNav};

/// Connect — two lanes for reaching people.
///
/// 1. A mail channel to the app team (reuses `FEEDBACK_EMAIL` + build identity
///    so an operator report lands with the exact revision they hit).
/// 2. A lightweight team message board: posts are `system`-typed activity
///    notes so they share the same audit trail as household/resource/placement
///    events without introducing a new table.
#[component]
pub fn ConnectPage() -> impl IntoView {
    let post_action = ServerAction::<CreateNote>::new();
    let data = Resource::new(
        move || post_action.version().get(),
        |_| async move { list_activity(50).await },
    );

    let author = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = post_action.value().get() {
            body.set(String::new());
            form_error.set(None);
        }
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let who = author.get_untracked().trim().to_string();
        let msg = body.get_untracked().trim().to_string();
        if who.is_empty() || msg.is_empty() {
            form_error.set(Some("Name and message are required.".into()));
            return;
        }
        form_error.set(None);
        post_action.dispatch(CreateNote {
            entity_type: "system".to_string(),
            entity_id: None,
            author: who,
            body: msg,
        });
    };

    let server_error = move || {
        post_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
    };

    let version = crate::build_info::VERSION;
    let sha = crate::build_info::COMMIT_SHA;
    let email = crate::build_info::FEEDBACK_EMAIL;
    let feedback_href = format!("mailto:{email}?subject=WHL%20feedback%20(v{version}%20{sha})");

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Connect"
                subtitle="Reach the app team, or post a note the whole field crew will see."
            />

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <ErrorBanner message=form_error.get().or_else(server_error).unwrap_or_default()/>
            </Show>

            <section class="panel">
                <div class="panel-head">
                    <h2>"Reach the app team"</h2>
                    <p class="muted">
                        "Bug, missing feature, or a workflow that fights you? Send a note — your \
                         draft is pre-stamped with the build you're on so we can reproduce it."
                    </p>
                </div>
                <p>
                    <a class="primary" href=feedback_href>"Email the team"</a>
                </p>
                <p class="muted">
                    {"Replies come from "}{email}{". For urgent client safety issues, use your standard on-call chain — not this inbox."}
                </p>
            </section>

            <section class="panel">
                <div class="panel-head">
                    <h2>"Team board"</h2>
                    <p class="muted">
                        "Short broadcasts to everyone who opens the app — shift notes, \
                         partner-agency updates, reminders. Posts are kept in the activity trail."
                    </p>
                </div>
                <form class="form-grid" on:submit=on_submit>
                    <div class="form-row">
                        <label for="cn-author">"Your name"</label>
                        <input id="cn-author" type="text" required placeholder="case_manager_kim"
                            prop:value=move || author.get()
                            on:input=move |ev| author.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-row form-row--wide">
                        <label for="cn-body">"Message"</label>
                        <textarea id="cn-body" rows="3" required
                            placeholder="e.g. Union Rescue Mission intake is paused until Monday — route men-only referrals to Koch Center."
                            prop:value=move || body.get()
                            on:input=move |ev| body.set(event_target_value(&ev))/>
                    </div>
                    <div class="form-actions">
                        <button class="primary" type="submit"
                            disabled=move || post_action.pending().get()>
                            {move || if post_action.pending().get() { "Posting…" } else { "Post to team" }}
                        </button>
                    </div>
                </form>
            </section>

            {move || match data.get() {
                None => view! { <p class="loading">"Loading board…"</p> }.into_any(),
                Some(Err(error)) => view! { <ErrorBanner message=error.to_string()/> }.into_any(),
                Some(Ok(notes)) => view! { <TeamBoard notes=notes/> }.into_any(),
            }}
        </main>
    }
}

#[component]
fn TeamBoard(notes: Vec<ActivityNote>) -> impl IntoView {
    let system_notes: Vec<ActivityNote> = notes
        .into_iter()
        .filter(|n| n.entity_type == "system")
        .collect();

    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Latest broadcasts"</h2>
                <p class="muted">"Last 50 entries from the activity trail, filtered to team posts."</p>
            </div>
            {if system_notes.is_empty() {
                view! {
                    <div class="empty-state">
                        <span class="empty-state-title">"Nothing posted yet"</span>
                        <p>"Be the first — a single heads-up here beats five duplicate Slack pings."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <ul class="activity-list">
                        {system_notes.into_iter().map(|note| view! {
                            <li class="activity-row">
                                <div class="activity-meta">
                                    <span class="pill pill--system">"Team"</span>
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
