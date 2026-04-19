use leptos::{ev::SubmitEvent, prelude::*};

use crate::api::{get_sitrep, SetSitrep, Sitrep};
use crate::components::layout::{PageHeader, TopNav};

/// `/situational` — the ops view of active stress events.
///
/// Static for v0.4: three hard-coded event cards, a three-layer cost
/// stack, stress-mode routing rules, and a 72h forecast. Content is
/// updated at release time — when a real crisis hits we change these
/// strings and redeploy. Turning this into live data is a follow-up
/// that requires an `events` table and a way for ops to mark an event
/// active, and it isn't worth the complexity until the second stress
/// event is in sight.
#[component]
pub fn SituationalPage() -> impl IntoView {
    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Situational · 2026-04-18 13:00"
                subtitle="Live view of stress events affecting intake capacity, placement throughput, and access costs. Routing rules auto-adjust while the sitrep banner is up."
            />

            <SitrepControl/>

            <div class="sit-grid">
                <div class="sit-col">
                    <ActiveEvents/>
                    <ThreeLayerCost/>
                </div>
                <aside class="sit-col">
                    <RoutingRules/>
                    <Forecast/>
                </aside>
            </div>
        </main>
    }
}

#[component]
fn SitrepControl() -> impl IntoView {
    let set_action = ServerAction::<SetSitrep>::new();
    let sitrep = Resource::new(
        move || set_action.version().get(),
        |_| async move { get_sitrep().await },
    );

    view! {
        <Suspense fallback=|| view! { <p class="loading">"Loading sitrep control…"</p> }>
            {move || sitrep.get().map(|res| match res {
                Err(err) => view! {
                    <section class="panel">
                        <div class="feedback feedback--error" role="status">
                            {err.to_string()}
                        </div>
                    </section>
                }.into_any(),
                Ok(current) => view! {
                    <SitrepForm current=current set_action=set_action/>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn SitrepForm(current: Sitrep, set_action: ServerAction<SetSitrep>) -> impl IntoView {
    let active = RwSignal::new(current.active);
    let summary = RwSignal::new(current.summary.clone());
    let level = RwSignal::new(current.level.clone());
    let updated_by = RwSignal::new(crate::operator::load().unwrap_or_default());
    let form_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(Ok(_)) = set_action.value().get() {
            crate::operator::save(&updated_by.get_untracked());
            form_error.set(None);
        }
    });

    let server_error = move || {
        set_action
            .value()
            .get()
            .and_then(|r| r.err().map(|e| e.to_string()))
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let summary_v = summary.get_untracked().trim().to_string();
        if active.get_untracked() && summary_v.is_empty() {
            form_error.set(Some(
                "Write a one-line summary before activating — the banner needs a message.".into(),
            ));
            return;
        }
        form_error.set(None);
        set_action.dispatch(SetSitrep {
            active: active.get_untracked(),
            summary: summary_v,
            level: level.get_untracked(),
            updated_by: updated_by.get_untracked(),
        });
    };

    let was_active = current.active;
    let status_line = {
        let updated_at = current.updated_at.clone();
        let updated_by_prev = current.updated_by.clone();
        let started_at = current.started_at.clone();
        move || {
            let mut bits: Vec<String> = Vec::new();
            if was_active {
                if let Some(started) = started_at.as_ref() {
                    bits.push(format!("active since {started}"));
                }
            } else {
                bits.push("inactive".to_string());
            }
            bits.push(format!("last change {updated_at}"));
            if let Some(by) = updated_by_prev.as_ref() {
                bits.push(format!("by {by}"));
            }
            bits.join(" · ")
        }
    };

    view! {
        <section class="panel">
            <div class="panel-head">
                <div>
                    <h2>"Sitrep control"</h2>
                    <p>"Toggle the red banner above every page. Every operator sees the change on the next page load."</p>
                </div>
                <span class="muted small">{status_line}</span>
            </div>

            <Show when=move || form_error.get().is_some() || server_error().is_some()>
                <div class="feedback feedback--error" role="status">
                    {move || form_error.get().or_else(server_error).unwrap_or_default()}
                </div>
            </Show>
            <Show when=move || matches!(set_action.value().get(), Some(Ok(_)))>
                <div class="feedback feedback--success" role="status">
                    "Saved. Banner reflects the new state on the next page load."
                </div>
            </Show>

            <form class="form-grid" on:submit=on_submit>
                <div class="form-row form-row--span-4">
                    <label for="sit-active">"State"</label>
                    <select id="sit-active"
                        prop:value=move || if active.get() { "on" } else { "off" }
                        on:change=move |ev| active.set(event_target_value(&ev) == "on")>
                        <option value="off">"Off — no banner"</option>
                        <option value="on">"On — banner visible"</option>
                    </select>
                </div>
                <div class="form-row form-row--span-4">
                    <label for="sit-level">"Level"</label>
                    <select id="sit-level"
                        prop:value=move || level.get()
                        on:change=move |ev| level.set(event_target_value(&ev))>
                        <option value="warn">"Warn — single-system issue"</option>
                        <option value="red">"Red — cascading crisis"</option>
                    </select>
                </div>
                <div class="form-row form-row--span-4">
                    <label for="sit-by">"Your handle"</label>
                    <input id="sit-by" type="text" placeholder="case_manager_kim"
                        prop:value=move || updated_by.get()
                        on:input=move |ev| updated_by.set(event_target_value(&ev))/>
                    <p class="form-hint">"Recorded on the change. Remembered from /activity if you've posted there."</p>
                </div>
                <div class="form-row form-row--wide">
                    <label for="sit-summary">"Summary"</label>
                    <textarea id="sit-summary" rows="2" maxlength="240"
                        placeholder="e.g. Red Cross activation · Open Door offline · CrossRoads 1/12 beds"
                        prop:value=move || summary.get()
                        on:input=move |ev| summary.set(event_target_value(&ev))/>
                    <p class="form-hint">"240-char cap. This is the whole point of the banner — narrate."</p>
                </div>
                <div class="form-actions">
                    <button type="submit" class="primary"
                        disabled=move || set_action.pending().get()>
                        {move || if set_action.pending().get() { "Saving…" } else { "Save sitrep" }}
                    </button>
                </div>
            </form>
        </section>
    }
}

#[component]
fn ActiveEvents() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <div>
                    <h2>"Active events"</h2>
                    <p>"Three concurrent signals at 13:00."</p>
                </div>
            </div>

            <article class="event-card event-card--surge">
                <div class="event-head">
                    <span class="pill pill--cancelled">"Surge"</span>
                    <time>"started 06:10"</time>
                </div>
                <h3 class="event-title">"Red Cross activation — severe-weather cell overnight"</h3>
                <p class="event-body">
                    "~60 displaced residents absorbed into the regular shelter network. "
                    "211 Kansas overloaded; standard intake queue backing up behind emergency caseload."
                </p>
                <div class="event-impact">
                    <span class="impact-chip up">"intake queue +14"</span>
                    <span class="impact-chip up">"211 wait +22 min"</span>
                    <span class="impact-chip">"shelter occupancy 94%"</span>
                </div>
            </article>

            <article class="event-card event-card--outage">
                <div class="event-head">
                    <span class="pill pill--offline">"Outage"</span>
                    <time>"since 12:00"</time>
                </div>
                <h3 class="event-title">"Open Door offline — emergency roof repair (48–72h)"</h3>
                <p class="event-body">
                    "Highest-volume daytime walk-in hub closed. Center of Hope absorbing overflow "
                    "at reduced hours — exactly during the surge. Walk-in bottleneck imminent."
                </p>
                <div class="event-impact">
                    <span class="impact-chip up">"walk-in capacity −52%"</span>
                    <span class="impact-chip">"COH reduced hours"</span>
                    <span class="impact-chip">"ETA 48–72h"</span>
                </div>
            </article>

            <article class="event-card event-card--youth">
                <div class="event-head">
                    <span class="pill pill--held">"Capacity"</span>
                    <time>"flagged 12:22"</time>
                </div>
                <h3 class="event-title">"CrossRoads at 1/12 — 4 youth incoming (DCF)"</h3>
                <p class="event-body">
                    "Transition-age youth with foster-placement disruptions from the same weather event "
                    "expected to present within 72h. BRIDGES has no vacancy. Youth-specific pathway is narrow."
                </p>
                <div class="event-impact">
                    <span class="impact-chip up">"youth demand +4"</span>
                    <span class="impact-chip up">"youth capacity 1"</span>
                    <span class="impact-chip">"BRIDGES full"</span>
                </div>
            </article>
        </section>
    }
}

#[component]
fn ThreeLayerCost() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <div>
                    <h2>"Three-layer cost — stress mode"</h2>
                    <p>"Every displacement event compounds across placement, mobility, and food access."</p>
                </div>
            </div>
            <div class="cost-stack">
                <div class="cost-layer cost-layer--crit">
                    <span class="cost-layer-num">"01"</span>
                    <div>
                        <div class="cost-layer-title">"Housing placement"</div>
                        <div class="cost-layer-sub">
                            "Fewer available beds. Voucher-friendly units lengthen waitlists. "
                            <strong>"17/87 emergency beds"</strong>
                            " reachable; reduced further during outage."
                        </div>
                    </div>
                    <div class="cost-layer-meter" aria-label="critical">
                        <span style="width: 88%"></span>
                    </div>
                </div>
                <div class="cost-layer cost-layer--warn">
                    <span class="cost-layer-num">"02"</span>
                    <div>
                        <div class="cost-layer-title">"Mobility / access"</div>
                        <div class="cost-layer-sub">
                            "$50 transit pass out of reach for zero-income clients. Weekday-only intake "
                            "means weekend crises have no coordinated entry — only 2 sites 24/7."
                        </div>
                    </div>
                    <div class="cost-layer-meter" aria-label="warn">
                        <span style="width: 62%"></span>
                    </div>
                </div>
                <div class="cost-layer cost-layer--warn">
                    <span class="cost-layer-num">"03"</span>
                    <div>
                        <div class="cost-layer-title">"Food insecurity"</div>
                        <div class="cost-layer-sub">
                            "Standard SNAP = 30 days. Expedited 7-day for homeless applicants is "
                            <strong>"underused"</strong>
                            ". Flag homeless status explicitly at application — do not assume."
                        </div>
                    </div>
                    <div class="cost-layer-meter" aria-label="warn">
                        <span style="width: 70%"></span>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn RoutingRules() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Default routing — stress mode"</h2>
            </div>
            <ol class="rule-list">
                <li>
                    <div>
                        <strong>"Route to 211 Kansas or CrossRoads first"</strong>
                        " when outside M–F business hours. Only 24/7 pathways."
                    </div>
                </li>
                <li>
                    <div>
                        <strong>"Flag homeless status explicitly"</strong>
                        " at SNAP application to trigger 7-day expedited processing. "
                        "Do not assume intake staff will."
                    </div>
                </li>
                <li>
                    <div>
                        <strong>"Prioritize HumanKind Villas & Hilltop Village"</strong>
                        " for clients with no income or deposit capacity — both $0 move-in, "
                        "no income prereqs."
                    </div>
                </li>
                <li>
                    <div>
                        <strong>"Skip Open Door referrals"</strong>
                        " (offline 48–72h). Center of Hope absorbs overflow at reduced hours only."
                    </div>
                </li>
                <li>
                    <div>
                        <strong>"Hold CrossRoads capacity"</strong>
                        " for 4 incoming DCF youth — do not burn the last bed on non-youth intake."
                    </div>
                </li>
            </ol>
        </section>
    }
}

#[component]
fn Forecast() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Next 72h forecast"</h2>
            </div>
            <dl class="kv">
                <dt>"Expected youth arrivals"</dt><dd>"4 (DCF-flagged)"</dd>
                <dt>"Open Door reopens"</dt><dd>"~2026-04-20 · 48–72h"</dd>
                <dt>"211 backlog burn-down"</dt><dd>"~36h at current staffing"</dd>
                <dt>"HumanKind Villas waitlist"</dt><dd>"only barrier — moved to top"</dd>
                <dt>"Hilltop Village waitlist"</dt><dd>"4–6 weeks · strong for families"</dd>
                <dt>"SNAP expedited cohort"</dt><dd>"14 clients this week"</dd>
            </dl>
        </section>
    }
}
