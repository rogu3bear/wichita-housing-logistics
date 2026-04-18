use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn TopNav() -> impl IntoView {
    view! {
        <header class="topnav">
            <div class="topnav-inner">
                <A href="/" attr:class="topnav-brand">
                    <span class="topnav-brand-mark">"WHL"</span>
                    <span class="topnav-brand-text">"Wichita Housing Logistics"</span>
                </A>
                <nav class="topnav-links" aria-label="Primary">
                    <A href="/" attr:class="topnav-link" exact=true>"Dashboard"</A>
                    <A href="/households" attr:class="topnav-link">"Households"</A>
                    <A href="/inventory" attr:class="topnav-link">"Inventory"</A>
                    <A href="/placements" attr:class="topnav-link">"Placements"</A>
                    <A href="/activity" attr:class="topnav-link">"Activity"</A>
                    <A href="/resources" attr:class="topnav-link">"Resources"</A>
                    <A href="/reference" attr:class="topnav-link">"Reference"</A>
                    <A href="/connect" attr:class="topnav-link">"Connect"</A>
                </nav>
            </div>
        </header>
    }
}

#[component]
pub fn PageHeader(#[prop(into)] title: String, #[prop(into)] subtitle: String) -> impl IntoView {
    view! {
        <header class="page-header">
            <h1 class="page-title">{title}</h1>
            <p class="page-subtitle">{subtitle}</p>
        </header>
    }
}

#[component]
pub fn ErrorBanner(message: String) -> impl IntoView {
    view! {
        <div class="feedback feedback--error" role="status">
            {message}
        </div>
    }
}

#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into)] body: String,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <span class="empty-state-title">{title}</span>
            <p>{body}</p>
        </div>
    }
}

#[component]
pub fn BuildFooter() -> impl IntoView {
    let version = crate::build_info::VERSION;
    let sha = crate::build_info::COMMIT_SHA;
    let email = crate::build_info::FEEDBACK_EMAIL;
    // Pre-fill the operator's mail draft with the exact revision they're on.
    let feedback_href = format!(
        "mailto:{email}?subject=WHL%20feedback%20(v{version}%20{sha})",
    );

    view! {
        <footer class="build-footer">
            <span class="build-id">
                "v" {version} " · " {sha}
            </span>
            <a class="build-feedback" href=feedback_href>
                "Report an issue"
            </a>
        </footer>
    }
}

/// Pill class for an activity entity type. `placement` gets the `-entity`
/// suffix so the color doesn't collide with the `placement` *stage* pill.
pub fn entity_pill_class(entity_type: &str) -> String {
    match entity_type {
        "placement" => "pill pill--placement-entity".to_string(),
        other => format!("pill pill--{other}"),
    }
}

/// Pill class for a household stage or placement/resource status token.
pub fn status_pill_class(token: &str) -> String {
    format!("pill pill--{token}")
}

pub fn humanize(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
