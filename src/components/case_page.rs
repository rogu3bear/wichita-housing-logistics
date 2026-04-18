use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::use_params_map;

use crate::api::{case_view, CasePlacement, CaseUpdate, CaseView, SubmitHouseholdUpdate};

/// `/case/:token` — customer-facing page for a single household.
///
/// Share-token authentication: whoever holds the URL reads the case.
/// No topnav, no roster, no audit trail across households. Plain
/// language for the stage, a clear "your current placement" block when
/// one exists, the five most recent updates tagged to the household,
/// and a single-field form to send a message back to the case manager.
#[component]
pub fn CasePage() -> impl IntoView {
    let params = use_params_map();
    let token = Memo::new(move |_| params.read().get("token").unwrap_or_default());

    let update_action = ServerAction::<SubmitHouseholdUpdate>::new();

    let case = Resource::new(
        move || (token.get(), update_action.version().get()),
        |(token, _)| async move {
            if token.is_empty() {
                return Err(leptos::prelude::ServerFnError::ServerError(
                    "This case link isn't active.".into(),
                ));
            }
            case_view(token).await
        },
    );

    view! {
        <main class="case-shell">
            {move || match case.get() {
                None => view! { <p class="loading">"Loading your case…"</p> }.into_any(),
                Some(Err(err)) => view! {
                    <section class="case-error" role="status">
                        <h1>"We can't find that case."</h1>
                        <p>{err.to_string()}</p>
                        <p class="muted small">
                            "If this link came from your case manager and it stopped working, "
                            "send them a message and ask for a fresh one."
                        </p>
                    </section>
                }.into_any(),
                Some(Ok(data)) => view! {
                    <CaseBody data=data token=token update_action=update_action/>
                }.into_any(),
            }}
        </main>
    }
}

#[component]
fn CaseBody(
    data: CaseView,
    token: Memo<String>,
    update_action: ServerAction<SubmitHouseholdUpdate>,
) -> impl IntoView {
    let CaseView {
        head_name,
        household_size,
        stage,
        intake_notes,
        current_placement,
        recent_updates,
        updated_at,
    } = data;

    let stage_plain = plain_language(&stage);
    let stage_clone = stage.clone();

    view! {
        <header class="case-header">
            <p class="case-eyebrow">"Your housing case"</p>
            <h1 class="case-title">{format!("Hi, {}.", head_name)}</h1>
            <p class="case-status">
                {stage_plain.headline}
            </p>
        </header>

        <section class="case-panel">
            <StageTimeline current=stage_clone/>
            <p class="case-stage-body">{stage_plain.body}</p>
        </section>

        {current_placement.map(|p| view! { <CasePlacementCard placement=p/> })}

        {intake_notes.clone().map(|note| view! {
            <section class="case-panel">
                <h2 class="case-h2">"What we know so far"</h2>
                <p class="case-notes">{note}</p>
            </section>
        })}

        <section class="case-panel">
            <div class="case-panel-head">
                <h2 class="case-h2">"Recent updates"</h2>
                <span class="muted small">{format!("as of {updated_at}")}</span>
            </div>
            {if recent_updates.is_empty() {
                view! {
                    <p class="muted">"No updates yet. Your case manager will post here as things move."</p>
                }.into_any()
            } else {
                view! {
                    <ul class="case-updates">
                        {recent_updates.into_iter().map(|u| view! { <CaseUpdateRow entry=u/> }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>

        <HouseholdMessageForm token=token update_action=update_action/>

        <footer class="case-footer">
            <p class="muted small">
                {format!("Household of {}. ", pluralize(household_size))}
                "This page is private to you — please don't share the link."
            </p>
        </footer>
    }
}

#[component]
fn CasePlacementCard(placement: CasePlacement) -> impl IntoView {
    let CasePlacement {
        resource_label,
        status,
        started_at,
    } = placement;

    let headline = match status.as_str() {
        "proposed" => "We're holding a spot for you.",
        "confirmed" => "Your placement is confirmed.",
        "moved_in" => "You're moved in.",
        "exited" => "Your placement has ended.",
        "cancelled" => "This placement didn't work out.",
        _ => "Your current placement",
    };
    let pill_class = format!("pill pill--{status}");
    let humanized = humanize_status(&status);

    view! {
        <section class="case-panel case-panel--highlight">
            <p class="case-eyebrow">"Current placement"</p>
            <h2 class="case-h2">{headline}</h2>
            <div class="case-placement">
                <div class="strong">{resource_label}</div>
                <div class="case-placement-meta">
                    <span class=pill_class>{humanized}</span>
                    {started_at.map(|s| view! {
                        <span class="muted small">{format!("since {s}")}</span>
                    })}
                </div>
            </div>
        </section>
    }
}

#[component]
fn CaseUpdateRow(entry: CaseUpdate) -> impl IntoView {
    let CaseUpdate {
        body,
        author_is_household,
        created_at,
    } = entry;
    let author_label = if author_is_household {
        "you"
    } else {
        "case team"
    };
    let row_class = if author_is_household {
        "case-update case-update--self"
    } else {
        "case-update"
    };

    view! {
        <li class=row_class>
            <div class="case-update-meta">
                <span class="case-update-from">{author_label}</span>
                <time>{created_at}</time>
            </div>
            <p class="case-update-body">{body}</p>
        </li>
    }
}

#[component]
fn HouseholdMessageForm(
    token: Memo<String>,
    update_action: ServerAction<SubmitHouseholdUpdate>,
) -> impl IntoView {
    let body = RwSignal::new(String::new());
    let local_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = update_action.value().get() {
            body.set(String::new());
            local_error.set(None);
        }
    });

    let server_error = move || {
        update_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let msg = body.get_untracked().trim().to_string();
        if msg.is_empty() {
            local_error.set(Some("Add a few words before sending.".into()));
            return;
        }
        local_error.set(None);
        update_action.dispatch(SubmitHouseholdUpdate {
            token: token.get_untracked(),
            body: msg,
        });
    };

    view! {
        <section class="case-panel">
            <h2 class="case-h2">"Send a message"</h2>
            <p class="muted small">
                "Anything you write here goes to your case manager's feed. "
                "Use it for updates — new phone number, appointment you can't make, "
                "questions you'd rather put in writing."
            </p>
            <Show when=move || local_error.get().is_some() || server_error().is_some()>
                <div class="feedback feedback--error" role="status">
                    {move || local_error.get().or_else(server_error).unwrap_or_default()}
                </div>
            </Show>
            <Show when=move || {
                matches!(update_action.value().get(), Some(Ok(_))) && body.with(|s| s.is_empty())
            }>
                <div class="feedback feedback--success" role="status">
                    "Sent. Your case manager will see this in their feed."
                </div>
            </Show>
            <form class="case-form" on:submit=on_submit>
                <label for="case-body" class="case-form-label">"Your message"</label>
                <textarea id="case-body" rows="3"
                    placeholder="Type whatever you'd like them to know…"
                    prop:value=move || body.get()
                    on:input=move |ev| body.set(event_target_value(&ev))/>
                <div class="case-form-actions">
                    <button type="submit" class="primary"
                        disabled=move || update_action.pending().get()>
                        {move || if update_action.pending().get() { "Sending…" } else { "Send" }}
                    </button>
                </div>
            </form>
        </section>
    }
}

#[component]
fn StageTimeline(current: String) -> impl IntoView {
    const STAGES: &[(&str, &str)] = &[
        ("intake", "Intake"),
        ("assessment", "Assessment"),
        ("placement", "Placement"),
        ("follow_up", "Follow-up"),
    ];
    let current_idx = STAGES
        .iter()
        .position(|(s, _)| *s == current)
        .unwrap_or(usize::MAX);
    let is_exited = current == "exited";

    view! {
        <ol class="case-timeline" aria-label="Your progress">
            {STAGES.iter().enumerate().map(|(i, (key, label))| {
                let state = if is_exited {
                    "case-step case-step--past"
                } else if i < current_idx {
                    "case-step case-step--past"
                } else if i == current_idx {
                    "case-step case-step--current"
                } else {
                    "case-step case-step--future"
                };
                let marker = if is_exited || i < current_idx { "✓" } else { "" };
                view! {
                    <li class=state data-stage=*key>
                        <span class="case-step-dot" aria-hidden="true">{marker}</span>
                        <span class="case-step-label">{*label}</span>
                    </li>
                }
            }).collect::<Vec<_>>()}
        </ol>
    }
}

struct StageCopy {
    headline: &'static str,
    body: &'static str,
}

fn plain_language(stage: &str) -> StageCopy {
    match stage {
        "intake" => StageCopy {
            headline: "We've received your information.",
            body: "Your case manager is reviewing what you've shared. They'll reach out soon about the next step.",
        },
        "assessment" => StageCopy {
            headline: "We're working on the right fit.",
            body: "We're matching your situation to the housing options we coordinate. Expect a call to confirm details and talk through choices.",
        },
        "placement" => StageCopy {
            headline: "We're placing you.",
            body: "We're lining up a specific place and working through move-in logistics. Watch for updates here and a call from your case manager.",
        },
        "follow_up" => StageCopy {
            headline: "You're housed — we're checking in.",
            body: "Your placement is active. Your case manager will follow up periodically to make sure things are going well.",
        },
        "exited" => StageCopy {
            headline: "Your case is closed.",
            body: "If your situation changes, reach out and we'll reopen your case.",
        },
        _ => StageCopy {
            headline: "Your case is active.",
            body: "Your case manager will update this page as things move.",
        },
    }
}

fn humanize_status(value: &str) -> String {
    match value {
        "moved_in" => "Moved in".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn pluralize(size: i64) -> String {
    if size == 1 {
        "1 person".to_string()
    } else {
        format!("{size} people")
    }
}
