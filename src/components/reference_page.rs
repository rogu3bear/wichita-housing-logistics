use leptos::prelude::*;

use crate::components::layout::{PageHeader, TopNav};

struct HousingOption {
    name: &'static str,
    rent: &'static str,
    deposit: Tri,
    voucher: bool,
    transit: &'static str,
    note: &'static str,
}

enum Tri {
    Yes,
    No,
    None_,
}

const HOUSING: &[HousingOption] = &[
    HousingOption {
        name: "HumanKind Villas",
        rent: "$0",
        deposit: Tri::No,
        voucher: true,
        transit: "0.3 mi",
        note: "Housing First, no prerequisites. Rent = 30% of income. Waitlist is the only barrier.",
    },
    HousingOption {
        name: "NEXTenant — Riverside Commons",
        rent: "$725",
        deposit: Tri::Yes,
        voucher: true,
        transit: "0.8 mi",
        note: "Flexible screening. Deposit assistance via NEXTenant. Route 11 nearby.",
    },
    HousingOption {
        name: "NEXTenant — Prairie Wind",
        rent: "$695",
        deposit: Tri::Yes,
        voucher: true,
        transit: "1.2 mi",
        note: "1–2 units turn monthly. Landlord known for flexibility with formerly homeless tenants.",
    },
    HousingOption {
        name: "NEXTenant — Douglas Ave Flats",
        rent: "$775",
        deposit: Tri::Yes,
        voucher: true,
        transit: "0.4 mi",
        note: "Above fair-market value, but walkable to Open Door and Center of Hope.",
    },
    HousingOption {
        name: "VASH Housing — Wichita VA",
        rent: "$0",
        deposit: Tri::No,
        voucher: true,
        transit: "1.5 mi",
        note: "Veterans only. HUD-VASH voucher + VA case management. 52 placements in 2025.",
    },
    HousingOption {
        name: "Maple Ridge Apartments",
        rent: "$825",
        deposit: Tri::Yes,
        voucher: false,
        transit: "2.1 mi",
        note: "Private market baseline. Credit/income/background screening blocks most exiting homelessness.",
    },
    HousingOption {
        name: "Sunflower Court",
        rent: "$750",
        deposit: Tri::Yes,
        voucher: false,
        transit: "3.4 mi",
        note: "No vouchers. 2-year rental history required. Nearest bus stop 1.8 mi away.",
    },
    HousingOption {
        name: "Pawnee Crossing Duplexes",
        rent: "$650",
        deposit: Tri::Yes,
        voucher: true,
        transit: "4.2 mi",
        note: "Cheapest HCV option, but south Wichita location creates distance from intake + transit.",
    },
    HousingOption {
        name: "Central Park Towers",
        rent: "$950",
        deposit: Tri::Yes,
        voucher: false,
        transit: "0.2 mi",
        note: "Luxury downtown stock. Best access — functionally inaccessible to anyone exiting homelessness.",
    },
    HousingOption {
        name: "Hilltop Village",
        rent: "$575",
        deposit: Tri::None_,
        voucher: true,
        transit: "1.8 mi",
        note: "Project-based vouchers. No deposit. Lighter screening. Waitlist 4–6 weeks. Strong for families.",
    },
];

struct IntakeSite {
    name: &'static str,
    hours: &'static str,
    mode: &'static str,
    live_transfer: bool,
}

const INTAKE_SITES: &[IntakeSite] = &[
    IntakeSite { name: "211 Kansas / United Way",      hours: "24/7",                 mode: "Phone",    live_transfer: true },
    IntakeSite { name: "H.O.T. Team",                  hours: "Mon–Fri, 7AM–3:30PM",  mode: "Walk-in",  live_transfer: true },
    IntakeSite { name: "United Methodist Open Door",   hours: "Mon–Fri, 8AM–4PM",     mode: "Walk-in",  live_transfer: true },
    IntakeSite { name: "CrossRoads Youth Intake",      hours: "24/7",                 mode: "Walk-in",  live_transfer: true },
    IntakeSite { name: "Second Light Coordinated Entry", hours: "Mon–Fri, 9AM–5PM",   mode: "Walk-in",  live_transfer: false },
    IntakeSite { name: "Family Promise Intake Line",   hours: "Mon–Fri, 9AM–4PM",     mode: "Phone",    live_transfer: false },
    IntakeSite { name: "DCF Wichita Service Center",   hours: "Mon–Fri, 8AM–5PM",     mode: "Walk-in",  live_transfer: false },
    IntakeSite { name: "Center of Hope",               hours: "Mon–Fri, 9AM–3PM",     mode: "Walk-in",  live_transfer: true },
];

#[component]
pub fn ReferencePage() -> impl IntoView {
    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Field reference"
                subtitle="Local housing stock, intake sites, access costs, and stress-mode routing rules. Static reference — not live pipeline state."
            />

            <ContextPanel/>
            <StressRulesPanel/>
            <HousingPanel/>
            <IntakePanel/>
            <AccessCostsPanel/>
        </main>
    }
}

#[component]
fn ContextPanel() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Housing cost context"</h2>
                <p>"1-bedroom rent distribution and voucher share across the 10 tracked properties."</p>
            </div>
            <div class="ref-stats">
                <ContextStat label="Rent range"    value="$575 – $950"/>
                <ContextStat label="Median rent"   value="$738"/>
                <ContextStat label="Average rent"  value="$743"/>
                <ContextStat label="Voucher-accepting" value="7 of 10"/>
                <ContextStat label="Emergency beds reachable" value="17 of 87"/>
            </div>
        </section>
    }
}

#[component]
fn ContextStat(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="ref-stat">
            <span class="ref-stat-label">{label}</span>
            <strong class="ref-stat-value">{value}</strong>
        </div>
    }
}

#[component]
fn StressRulesPanel() -> impl IntoView {
    view! {
        <section class="callout callout--warn">
            <h2 class="callout-title">"Stress-mode routing"</h2>
            <p class="callout-lede">
                "When a surge, shelter closure, or intake outage hits, treat it as a simultaneous three-layer cost event: housing, mobility, and food all spike."
            </p>
            <ol class="callout-list">
                <li>
                    <strong>"Outside M–F business hours: "</strong>
                    "route to 211 Kansas or CrossRoads Youth first — everyone else is closed."
                </li>
                <li>
                    <strong>"SNAP at intake: "</strong>
                    "always flag homeless status explicitly to trigger 7-day expedited processing. Do not assume staff will do this."
                </li>
                <li>
                    <strong>"Zero-income, no-deposit: "</strong>
                    "prioritize HumanKind Villas and Hilltop Village — $0 move-in, no income prerequisites."
                </li>
            </ol>
        </section>
    }
}

#[component]
fn HousingPanel() -> impl IntoView {
    view! {
        <section class="panel panel--flush">
            <div class="panel-head">
                <div>
                    <h2>"Housing options"</h2>
                    <p>"10 local properties · rent, deposit, voucher acceptance, transit distance."</p>
                </div>
            </div>
            <div class="data-table-scroll">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Property"</th>
                            <th>"Rent"</th>
                            <th>"Deposit"</th>
                            <th>"Voucher"</th>
                            <th>"Transit"</th>
                            <th>"Notes"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {HOUSING.iter().map(|opt| view! {
                            <tr>
                                <td><span class="strong">{opt.name}</span></td>
                                <td>{opt.rent}</td>
                                <td>{deposit_cell(&opt.deposit)}</td>
                                <td>{yes_no_pill(opt.voucher)}</td>
                                <td class="muted">{opt.transit}</td>
                                <td class="muted ref-note">{opt.note}</td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

#[component]
fn IntakePanel() -> impl IntoView {
    view! {
        <section class="panel panel--flush">
            <div class="panel-head">
                <div>
                    <h2>"Intake network"</h2>
                    <p>"Only 211 Kansas and CrossRoads Youth run 24/7. Live transfer is available at a subset of sites."</p>
                </div>
            </div>
            <div class="data-table-scroll">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>"Site"</th>
                            <th>"Hours"</th>
                            <th>"Mode"</th>
                            <th>"Live transfer"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {INTAKE_SITES.iter().map(|s| view! {
                            <tr>
                                <td><span class="strong">{s.name}</span></td>
                                <td>{s.hours}</td>
                                <td class="muted">{s.mode}</td>
                                <td>{yes_no_pill(s.live_transfer)}</td>
                            </tr>
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

#[component]
fn AccessCostsPanel() -> impl IntoView {
    view! {
        <section class="panel">
            <div class="panel-head">
                <h2>"Access costs"</h2>
                <p>"Cash-gated logistics that block intake if not budgeted for up-front."</p>
            </div>
            <dl class="ref-costs">
                <CostRow
                    label="Transit pass"
                    value="$50 / month"
                    note="Prohibitive for zero-income clients. Purchase requires transit center or online access. Reduced fare needs a separate application; 1-day processing."
                />
                <CostRow
                    label="SNAP application"
                    value="$0"
                    note="Standard 30-day processing. 7-day expedited available for homeless applicants but consistently underused. Average single-adult benefit ≈ $234/month (~$7.80/day). Recertification every 6 months is a common drop-off point."
                />
                <CostRow label="State ID"         value="$22"  note="Required for most downstream benefits."/>
                <CostRow label="Birth certificate" value="$15"  note="Often the blocker for obtaining State ID."/>
                <CostRow label="GED registration" value="$120" note="Employment gate for many stabilization paths."/>
            </dl>
        </section>
    }
}

#[component]
fn CostRow(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] note: String,
) -> impl IntoView {
    view! {
        <div class="ref-cost">
            <dt class="ref-cost-head">
                <span class="strong">{label}</span>
                <span class="ref-cost-value">{value}</span>
            </dt>
            <dd class="ref-cost-note muted">{note}</dd>
        </div>
    }
}

fn yes_no_pill(v: bool) -> impl IntoView {
    let (class, label) = if v {
        ("pill pill--available", "Yes")
    } else {
        ("pill pill--exited", "No")
    };
    view! { <span class=class>{label}</span> }
}

fn deposit_cell(tri: &Tri) -> impl IntoView {
    match tri {
        Tri::Yes => view! { <span class="pill pill--available">"Yes"</span> }.into_any(),
        Tri::No => view! { <span class="pill pill--exited">"No"</span> }.into_any(),
        Tri::None_ => view! { <span class="muted">"None"</span> }.into_any(),
    }
}
