use leptos::{ev::SubmitEvent, prelude::*};

use crate::components::layout::{PageHeader, TopNav};

/// `/routing` — feasibility calculator over a curated Wichita property
/// list. Given a client profile (income, voucher, deposit capacity, max
/// distance, household type, presenting time), rank available options
/// and surface next-step intake + access-cost checklist.
///
/// Everything is client-side — no server fns. `PROPERTIES` is a static
/// const; scoring is pure; no D1 access. This is deliberately separate
/// from the D1-backed `/inventory` page: that table holds our own beds
/// and units, this directory holds the broader market we route into.
#[component]
pub fn RoutingPage() -> impl IntoView {
    let income = RwSignal::new(0_i64);
    let voucher = RwSignal::new("hcv".to_string());
    let deposit = RwSignal::new(0_i64);
    let max_dist = RwSignal::new(2.0_f64);
    let family = RwSignal::new("single".to_string());
    let when_present = RwSignal::new("biz".to_string());

    // Trigger signal — bumped on form submit. Keeps scoring reactive
    // without recomputing on every keystroke.
    let run = RwSignal::new(0_u32);

    let result = Memo::new(move |_| {
        // Depend on `run` so only an explicit "Run routing" refreshes.
        let _ = run.get();
        score_all(
            income.get_untracked(),
            &voucher.get_untracked(),
            deposit.get_untracked(),
            max_dist.get_untracked(),
            &family.get_untracked(),
            &when_present.get_untracked(),
        )
    });

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        run.update(|n| *n = n.wrapping_add(1));
    };
    let on_reset = move |_| {
        income.set(0);
        voucher.set("hcv".to_string());
        deposit.set(0);
        max_dist.set(2.0);
        family.set("single".to_string());
        when_present.set("biz".to_string());
        run.update(|n| *n = n.wrapping_add(1));
    };

    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Routing helper"
                subtitle="Given a client's situation, rank feasible housing options and surface the access-cost checklist. Stress-mode rules applied automatically."
            />

            <div class="route-wrap">
                <section class="panel">
                    <div class="panel-head"><h2>"Client profile"</h2></div>
                    <form class="form-grid" on:submit=on_submit>
                        <div class="form-row form-row--span-6">
                            <label for="r-income">"Monthly income ($)"</label>
                            <input id="r-income" type="number" min="0"
                                prop:value=move || income.get().to_string()
                                on:input=move |ev| income.set(event_target_value(&ev).parse().unwrap_or(0))/>
                        </div>
                        <div class="form-row form-row--span-6">
                            <label for="r-voucher">"Voucher"</label>
                            <select id="r-voucher"
                                prop:value=move || voucher.get()
                                on:change=move |ev| voucher.set(event_target_value(&ev))>
                                <option value="none">"None"</option>
                                <option value="hcv">"HCV"</option>
                                <option value="vash">"VASH"</option>
                            </select>
                        </div>
                        <div class="form-row form-row--span-6">
                            <label for="r-deposit">"Deposit capacity ($)"</label>
                            <input id="r-deposit" type="number" min="0"
                                prop:value=move || deposit.get().to_string()
                                on:input=move |ev| deposit.set(event_target_value(&ev).parse().unwrap_or(0))/>
                        </div>
                        <div class="form-row form-row--span-6">
                            <label for="r-maxdist">"Max walk/transit distance (mi)"</label>
                            <input id="r-maxdist" type="number" min="0" step="0.1"
                                prop:value=move || format!("{:.1}", max_dist.get())
                                on:input=move |ev| max_dist.set(event_target_value(&ev).parse().unwrap_or(99.0))/>
                        </div>
                        <div class="form-row form-row--span-6">
                            <label for="r-family">"Household type"</label>
                            <select id="r-family"
                                prop:value=move || family.get()
                                on:change=move |ev| family.set(event_target_value(&ev))>
                                <option value="single">"Single adult"</option>
                                <option value="family">"Family"</option>
                                <option value="youth">"Transition-age youth"</option>
                                <option value="veteran">"Veteran"</option>
                            </select>
                        </div>
                        <div class="form-row form-row--span-6">
                            <label for="r-when">"Presenting"</label>
                            <select id="r-when"
                                prop:value=move || when_present.get()
                                on:change=move |ev| when_present.set(event_target_value(&ev))>
                                <option value="biz">"Business hours"</option>
                                <option value="after">"After hours / weekend"</option>
                            </select>
                        </div>
                        <div class="form-actions">
                            <button type="button" class="ghost" on:click=on_reset>"Reset"</button>
                            <button type="submit" class="primary">"Run routing"</button>
                        </div>
                    </form>
                </section>

                <section class="panel">
                    <div class="panel-head">
                        <h2>"Result"</h2>
                        <p class="muted">
                            {move || result.with(|r| r.summary.clone())}
                        </p>
                    </div>
                    <div class="route-result">
                        <ScoreHeader result=result/>
                        <TopMatches result=result/>
                        <AccessChecklist/>
                        <IntakeNextStep family=family when_present=when_present/>
                    </div>
                </section>
            </div>
        </main>
    }
}

#[component]
fn ScoreHeader(result: Memo<RouteResult>) -> impl IntoView {
    view! {
        <div class="route-score">
            <span class="route-score-num">
                {move || result.with(|r| r.top_score.to_string())}
            </span>
            <span class="route-score-label">
                "feasibility (0–100) · "
                <span>{move || result.with(|r| r.hint)}</span>
            </span>
        </div>
    }
}

#[component]
fn TopMatches(result: Memo<RouteResult>) -> impl IntoView {
    view! {
        <div>
            <h3 class="section-title">"Top matches"</h3>
            <ol class="route-recs">
                {move || result.with(|r| {
                    r.top.iter().enumerate().map(|(i, m)| {
                        let rank = format!("{:02} · {}", i + 1, m.score);
                        let headline = format!(
                            "{} · ${}{} · {:.1} mi",
                            m.name,
                            m.rent,
                            if m.has_deposit { " + dep" } else { "" },
                            m.dist,
                        );
                        let rationale = if m.reasons.is_empty() {
                            "baseline match".to_string()
                        } else {
                            m.reasons.join(" · ")
                        };
                        view! {
                            <li>
                                <div>
                                    <strong>{headline}</strong>
                                    <div class="rec-rationale">{rationale}</div>
                                </div>
                                <span class="rec-rank">{rank}</span>
                            </li>
                        }
                    }).collect::<Vec<_>>()
                })}
            </ol>
        </div>
    }
}

#[component]
fn AccessChecklist() -> impl IntoView {
    let items: &[(&str, &str, bool)] = &[
        ("State ID", "$22", false),
        ("Birth certificate", "$15", false),
        ("SNAP — flag homeless for 7-day expedited", "$0", true),
        ("Reduced-fare transit application", "1 day", false),
        ("Monthly transit pass", "$50", false),
        ("GED registration", "$120", false),
    ];
    view! {
        <div>
            <h3 class="section-title">"Access-cost checklist"</h3>
            <ul class="checklist">
                {items.iter().enumerate().map(|(i, (label, cost, checked))| {
                    let id = format!("c-{i}");
                    let checked = *checked;
                    view! {
                        <li>
                            <input type="checkbox" id=id.clone() prop:checked=checked/>
                            <label for=id.clone()>{*label}</label>
                            <span class="cost">{*cost}</span>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

#[component]
fn IntakeNextStep(
    family: RwSignal<String>,
    when_present: RwSignal<String>,
) -> impl IntoView {
    let intakes = Memo::new(move |_| intake_for(&family.get(), &when_present.get()));
    view! {
        <div>
            <h3 class="section-title">"Where to start"</h3>
            <ol class="route-recs">
                {move || intakes.with(|list| {
                    list.iter().enumerate().map(|(i, (name, rationale))| {
                        let rank = format!("{:02}", i + 1);
                        view! {
                            <li>
                                <div>
                                    <strong>{*name}</strong>
                                    <div class="rec-rationale">{*rationale}</div>
                                </div>
                                <span class="rec-rank">{rank}</span>
                            </li>
                        }
                    }).collect::<Vec<_>>()
                })}
            </ol>
        </div>
    }
}

// ------- Static property catalog + scoring -------------------------------

#[derive(Clone, Copy)]
struct Property {
    name: &'static str,
    rent: u32,
    voucher: bool,
    has_deposit: bool,
    dist: f32,
    fam: bool,
    vets: bool,
    youth: bool,
}

const PROPERTIES: &[Property] = &[
    Property { name: "HumanKind Villas",               rent:   0, voucher: true,  has_deposit: false, dist: 0.3, fam: true,  vets: false, youth: true  },
    Property { name: "Hilltop Village",                rent: 575, voucher: true,  has_deposit: false, dist: 1.8, fam: true,  vets: false, youth: true  },
    Property { name: "VASH — Wichita VA",              rent:   0, voucher: true,  has_deposit: false, dist: 1.5, fam: false, vets: true,  youth: false },
    Property { name: "NEXTenant — Prairie Wind",       rent: 695, voucher: true,  has_deposit: true,  dist: 1.2, fam: true,  vets: true,  youth: true  },
    Property { name: "NEXTenant — Riverside Commons",  rent: 725, voucher: true,  has_deposit: true,  dist: 0.8, fam: true,  vets: true,  youth: true  },
    Property { name: "Pawnee Crossing Duplexes",       rent: 650, voucher: true,  has_deposit: true,  dist: 4.2, fam: true,  vets: true,  youth: false },
    Property { name: "NEXTenant — Douglas Ave Flats",  rent: 775, voucher: true,  has_deposit: true,  dist: 0.4, fam: true,  vets: true,  youth: true  },
    Property { name: "Sunflower Court",                rent: 750, voucher: false, has_deposit: true,  dist: 3.4, fam: true,  vets: true,  youth: false },
    Property { name: "Maple Ridge Apartments",         rent: 825, voucher: false, has_deposit: true,  dist: 2.1, fam: true,  vets: true,  youth: false },
    Property { name: "Central Park Towers",            rent: 950, voucher: false, has_deposit: true,  dist: 0.2, fam: false, vets: false, youth: false },
];

#[derive(Clone, Default, PartialEq)]
struct RouteMatch {
    name: &'static str,
    rent: u32,
    has_deposit: bool,
    dist: f32,
    score: i32,
    reasons: Vec<String>,
}

#[derive(Clone, Default, PartialEq)]
struct RouteResult {
    top: Vec<RouteMatch>,
    top_score: i32,
    hint: &'static str,
    summary: String,
}

fn score_all(
    income: i64,
    voucher: &str,
    deposit: i64,
    max_dist: f64,
    family: &str,
    _when_present: &str,
) -> RouteResult {
    let mut scored: Vec<RouteMatch> = PROPERTIES
        .iter()
        .map(|p| score_one(p, income, voucher, deposit, max_dist, family))
        .collect();
    scored.sort_by(|a, b| b.score.cmp(&a.score));

    let top: Vec<RouteMatch> = scored.into_iter().take(3).collect();
    let top_score = top.first().map(|m| m.score).unwrap_or(0);
    let hint = feasibility_hint(top_score);
    let summary = match top.first() {
        Some(m) => format!("{} leads ({}/100). {}.", m.name, top_score, hint),
        None => "No feasible match. Escalate to a human.".to_string(),
    };

    RouteResult {
        top,
        top_score,
        hint,
        summary,
    }
}

fn score_one(
    p: &Property,
    income: i64,
    voucher: &str,
    deposit: i64,
    max_dist: f64,
    family: &str,
) -> RouteMatch {
    let mut score: i32 = 50;
    let mut reasons: Vec<String> = Vec::new();

    // VASH-only housing: hard fail anyone not a veteran.
    let is_vash = p.name.contains("VASH");
    if is_vash && family != "veteran" {
        return RouteMatch {
            name: p.name,
            rent: p.rent,
            has_deposit: p.has_deposit,
            dist: p.dist,
            score: 0,
            reasons: vec!["veterans only".into()],
        };
    }

    // Voucher fit.
    let has_voucher = voucher == "hcv" || voucher == "vash";
    if p.voucher && has_voucher {
        score += 15;
        reasons.push("voucher match".into());
    } else if !p.voucher && has_voucher {
        score -= 20;
        reasons.push("no voucher accepted".into());
    } else if !p.voucher && !has_voucher && income < (p.rent as i64) * 3 {
        score -= 35;
        reasons.push("income below 3× rent".into());
    }

    // Move-in cost.
    if p.rent == 0 {
        score += 25;
        reasons.push("$0 rent".into());
    }
    if !p.has_deposit {
        score += 10;
        reasons.push("no deposit".into());
    } else if deposit < p.rent as i64 {
        score -= 15;
        reasons.push("deposit gap".into());
    }

    // Distance.
    if (p.dist as f64) <= max_dist {
        score += 8;
    } else {
        score -= 12;
        reasons.push("beyond max distance".into());
    }

    // Household-type fit.
    match family {
        "veteran" => {
            if p.vets {
                score += 5;
            }
            if is_vash {
                score += 20;
                reasons.push("VASH-aligned".into());
            }
        }
        "family" => {
            if p.fam {
                score += 3;
            }
        }
        "youth" => {
            if !p.youth {
                score -= 10;
            }
        }
        _ => {}
    }

    // Stress-mode preference: zero-income clients biased to $0 move-in,
    // no-barrier properties. Reinforces the Situational routing rule.
    if income == 0 && (p.name.contains("HumanKind") || p.name.contains("Hilltop")) {
        score += 15;
        reasons.push("stress-mode preferred".into());
    }

    let score = score.clamp(0, 100);
    // Truncate reasons list for UI cleanliness.
    reasons.truncate(3);

    RouteMatch {
        name: p.name,
        rent: p.rent,
        has_deposit: p.has_deposit,
        dist: p.dist,
        score,
        reasons,
    }
}

fn feasibility_hint(score: i32) -> &'static str {
    if score >= 70 {
        "feasible · proceed to placement"
    } else if score >= 45 {
        "feasible with assistance"
    } else {
        "structural barriers — escalate"
    }
}

fn intake_for(family: &str, when_present: &str) -> Vec<(&'static str, &'static str)> {
    let after_hours = when_present == "after" || family == "youth";
    let mut intakes: Vec<(&str, &str)> = Vec::new();

    if family == "veteran" {
        intakes.push((
            "VASH Housing — Wichita VA",
            "veterans only · HUD-VASH + VA case mgmt",
        ));
    }

    if after_hours {
        if family == "youth" {
            intakes.push((
                "CrossRoads Youth Intake",
                "24/7 walk-in · hold capacity for DCF youth",
            ));
        }
        intakes.push((
            "211 Kansas",
            "24/7 phone · live transfer · overloaded today",
        ));
    } else {
        intakes.push(("H.O.T. Team", "M–F 7:00–15:30 · live transfer"));
        intakes.push(("Center of Hope", "reduced hours · Open Door overflow"));
    }

    intakes.truncate(3);
    intakes
}
