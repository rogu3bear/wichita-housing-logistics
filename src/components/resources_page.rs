use leptos::prelude::*;

use crate::components::layout::{PageHeader, TopNav};

/// Curated directory of Wichita-area community services intake staff can
/// refer households to. Distinct from `/inventory` (which tracks our own
/// shelter beds and units in D1). Entries are intentionally contact-method
/// agnostic — staff should confirm current hours, eligibility, and intake
/// details by calling 2-1-1 (United Way of the Plains).
struct Service {
    name: &'static str,
    area: &'static str,
    blurb: &'static str,
}

struct Category {
    title: &'static str,
    lede: &'static str,
    services: &'static [Service],
}

const DIRECTORY: &[Category] = &[
    Category {
        title: "Crisis & emergency shelter",
        lede: "First-night beds and crisis intake for adults, families, and DV survivors.",
        services: &[
            Service {
                name: "HumanKind Ministries — Inter-Faith Inn",
                area: "Downtown Wichita",
                blurb: "Emergency shelter for families and single women; day shelter and case management.",
            },
            Service {
                name: "Union Rescue Mission of Wichita",
                area: "North Wichita",
                blurb: "Emergency shelter and recovery programming for men.",
            },
            Service {
                name: "Wichita Family Crisis Center",
                area: "Confidential location",
                blurb: "24/7 shelter and advocacy for survivors of domestic violence.",
            },
            Service {
                name: "Salvation Army Koch Family Center",
                area: "Central Wichita",
                blurb: "Emergency shelter, meals, and case management for families.",
            },
        ],
    },
    Category {
        title: "Rent, utilities & housing stability",
        lede: "Keep a household in place or bridge the gap between assessment and placement.",
        services: &[
            Service {
                name: "Catholic Charities of Wichita",
                area: "Central Wichita",
                blurb: "Rental and utility assistance, case management, and financial coaching.",
            },
            Service {
                name: "Salvation Army Center of Hope",
                area: "Central Wichita",
                blurb: "Rent, utility, and prescription assistance for households in crisis.",
            },
            Service {
                name: "City of Wichita Housing & Community Services",
                area: "332 Riverview",
                blurb: "Public housing, Housing Choice Vouchers (Section 8), and eligibility intake.",
            },
            Service {
                name: "Mennonite Housing Rehabilitation Services",
                area: "Wichita metro",
                blurb: "Affordable housing development and homeowner repair programs.",
            },
        ],
    },
    Category {
        title: "Food & basic needs",
        lede: "Same-day meals and pantry referrals when cash is short.",
        services: &[
            Service {
                name: "Kansas Food Bank",
                area: "Wichita warehouse + partner pantries",
                blurb: "Regional food bank; connects to 250+ partner pantries across south-central Kansas.",
            },
            Service {
                name: "The Lord's Diner",
                area: "Three Wichita locations",
                blurb: "Free hot evening meal every day of the year; no ID or sign-in required.",
            },
            Service {
                name: "HumanKind Food Ministry",
                area: "Central Wichita",
                blurb: "Weekly pantry box with fresh and shelf-stable staples.",
            },
        ],
    },
    Category {
        title: "Health & behavioral health",
        lede: "Medical, mental health, and substance-use services on a sliding scale.",
        services: &[
            Service {
                name: "COMCARE of Sedgwick County",
                area: "Multiple clinic sites",
                blurb: "Community mental-health center: crisis, outpatient, and case management.",
            },
            Service {
                name: "GraceMed Health Clinic",
                area: "Multiple clinic sites",
                blurb: "FQHC — medical, dental, behavioral, and pharmacy on a sliding scale.",
            },
            Service {
                name: "HealthCore Clinic",
                area: "North-central Wichita",
                blurb: "FQHC primary care, behavioral health, and care coordination.",
            },
            Service {
                name: "Sedgwick County Substance Abuse Center",
                area: "Central Wichita",
                blurb: "Assessment and outpatient substance-use treatment.",
            },
        ],
    },
    Category {
        title: "Legal, benefits & work",
        lede: "Fight an eviction, apply for benefits, or find a paycheck.",
        services: &[
            Service {
                name: "Kansas Legal Services — Wichita",
                area: "Central Wichita",
                blurb: "Civil legal aid: evictions, landlord disputes, benefits appeals, family law.",
            },
            Service {
                name: "DCF Wichita Service Center",
                area: "East Wichita",
                blurb: "SNAP, TANF, child care assistance, and Medicaid applications.",
            },
            Service {
                name: "Workforce Alliance of South Central Kansas",
                area: "Workforce Center on S. Main",
                blurb: "Job placement, training funds, and career coaching.",
            },
        ],
    },
    Category {
        title: "Transportation & accessibility",
        lede: "A placement only holds if the household can get there.",
        services: &[
            Service {
                name: "Wichita Transit",
                area: "Citywide",
                blurb: "Fixed-route bus service; reduced-fare passes available for qualifying riders.",
            },
            Service {
                name: "Wichita Transit Paratransit",
                area: "Citywide (ADA)",
                blurb: "Curb-to-curb service for riders unable to use fixed-route buses.",
            },
            Service {
                name: "Sedgwick County Department on Aging",
                area: "Countywide",
                blurb: "Senior transportation, meals, and in-home support referrals.",
            },
        ],
    },
];

#[component]
pub fn ResourcesPage() -> impl IntoView {
    view! {
        <TopNav/>
        <main class="page-shell">
            <PageHeader
                title="Resources"
                subtitle="Wichita-area community services staff can refer households to — outside our own housing inventory."
            />

            <section class="panel">
                <div class="panel-head">
                    <h2>"Start with 2-1-1"</h2>
                    <p class="muted">
                        "Dial 2-1-1 (United Way of the Plains) for a real-time screen of open beds, \
                         current hours, and warm referrals. Treat the entries below as a map, not a \
                         schedule — confirm intake details before sending a household."
                    </p>
                </div>
            </section>

            {DIRECTORY.iter().map(|cat| view! {
                <section class="panel">
                    <div class="panel-head">
                        <h2>{cat.title}</h2>
                        <p class="muted">{cat.lede}</p>
                    </div>
                    <ul class="activity-list">
                        {cat.services.iter().map(|svc| view! {
                            <li class="activity-row">
                                <div class="activity-meta">
                                    <span class="pill">{svc.area}</span>
                                </div>
                                <p class="activity-body"><strong>{svc.name}</strong></p>
                                <p class="activity-author muted">{svc.blurb}</p>
                            </li>
                        }).collect::<Vec<_>>()}
                    </ul>
                </section>
            }).collect::<Vec<_>>()}
        </main>
    }
}
