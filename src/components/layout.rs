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
                    <A href="/situational" attr:class="topnav-link">"Situational"</A>
                    <A href="/households" attr:class="topnav-link">"Households"</A>
                    <A href="/inventory" attr:class="topnav-link">"Inventory"</A>
                    <A href="/placements" attr:class="topnav-link">"Placements"</A>
                    <A href="/routing" attr:class="topnav-link">"Routing"</A>
                    <A href="/activity" attr:class="topnav-link">"Activity"</A>
                    <A href="/resources" attr:class="topnav-link">"Resources"</A>
                    <A href="/reference" attr:class="topnav-link">"Reference"</A>
                    <A href="/connect" attr:class="topnav-link">"Connect"</A>
                </nav>
            </div>
        </header>
    }
}

/// Red/amber banner below the top nav during a declared stress event.
/// Reads live D1 state via `get_sitrep` — ops flip `active` in the
/// `/situational` control panel and the banner appears for everyone
/// without a redeploy.
#[component]
pub fn SitrepBanner() -> impl IntoView {
    let sitrep = Resource::new(|| (), |_| async move { crate::api::get_sitrep().await });

    view! {
        <Suspense fallback=|| ()>
            {move || sitrep.get().and_then(|res| match res {
                Ok(s) if s.active => Some(view! { <SitrepBar summary=s.summary level=s.level/> }),
                _ => None,
            })}
        </Suspense>
    }
}

#[component]
fn SitrepBar(summary: String, level: String) -> impl IntoView {
    let class = if level == "red" { "sitrep sitrep--red" } else { "sitrep" };
    view! {
        <aside class=class role="status" aria-live="polite">
            <div class="sitrep-inner">
                <span class="sitrep-tag">"SITREP"</span>
                <div class="sitrep-title">
                    "Stress conditions active"
                    <span>{summary}</span>
                </div>
                <div class="sitrep-actions">
                    <A href="/situational" attr:class="">"Open situational view"</A>
                    <A href="/routing" attr:class="primary">"Run routing"</A>
                </div>
            </div>
        </aside>
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
            <span class="build-credit">
                "Built by "
                <span class="build-credit-name">"James KC Auchterlonie"</span>
                <span class="build-credit-sep" aria-hidden="true"></span>
                <span class="build-credit-tag">"MLNavigator"</span>
                <span class="build-credit-sep" aria-hidden="true"></span>
                <span class="build-credit-tag">"adapterOS"</span>
            </span>
            <span class="build-meta">
                <span class="build-id">"v" {version} " · " {sha}</span>
                <a class="build-feedback" href=feedback_href>"Report an issue"</a>
            </span>
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

/// Sentence-case the first letter and turn underscores into spaces.
/// `"moved_in"` → `"Moved in"`, `"follow_up"` → `"Follow up"`,
/// `"system"` → `"System"`. Title-casing every word (the old behavior)
/// produced "Moved In" in some places and "Moved in" in others —
/// sentence case gives us one rule everywhere.
pub fn humanize(value: &str) -> String {
    let spaced = value.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Inline "*" marker for required form labels. `aria-hidden` so screen
/// readers don't announce "asterisk"; the `required` attribute on the
/// input already conveys the semantic requirement.
#[component]
pub fn RequiredMark() -> impl IntoView {
    view! {
        <span class="required-mark" aria-hidden="true">"*"</span>
    }
}

