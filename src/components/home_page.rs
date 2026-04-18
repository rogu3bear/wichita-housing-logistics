use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="page-shell">
            <section class="hero">
                <div class="hero-copy">
                    <h1>"It works."</h1>
                    <p class="hero-lede">
                        "Your Leptos app is running on Cloudflare Workers. "
                        "Edit this component in src/components/home_page.rs."
                    </p>
                </div>
            </section>
        </main>
    }
}
