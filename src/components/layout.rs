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
