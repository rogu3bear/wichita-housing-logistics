// Leptos' view! macro produces deeply nested generic types; release-mode
// monomorphization overflows rustc's default recursion limit (128) once
// page markup grows past a handful of panels. Bump this whenever a new
// page lands and the build complains.
#![recursion_limit = "512"]

mod api;
mod app;
mod asset_hashes;
mod build_info;
mod components;
mod operator;
#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    use axum::routing::post;
    use axum::Router;
    use axum::http::{
        header::{
            CACHE_CONTROL, CONTENT_SECURITY_POLICY, HeaderName, HeaderValue,
            REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
        },
    };
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
    use tower_service::Service;

    // wasm isolates don't support `inventory`-based auto-registration, so
    // every #[server] fn must be wired by hand. Idempotent — cheap on warm
    // isolates, required on cold ones.
    crate::api::register_all();

    let conf =
        get_configuration(None).map_err(|error| worker::Error::RustError(error.to_string()))?;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(app::App);
    let state = server::AppState::new(leptos_options.clone(), env);

    // Server fns POST to `/api/<FnName>` from the browser. Without this
    // route they silently 404, which means every ServerAction::dispatch
    // from the client never reaches Rust — all admin mutations + the
    // customer /case update form would be broken.
    let state_for_fns = state.clone();
    let mut router = Router::new()
        .route(
            "/api/{*fn_name}",
            post(move |req| {
                let state = state_for_fns.clone();
                async move {
                    handle_server_fns_with_context(
                        move || {
                            leptos::prelude::provide_context(state.clone());
                        },
                        req,
                    )
                    .await
                }
            }),
        )
        .leptos_routes_with_context(&state, routes, || {}, {
            let leptos_options = leptos_options.clone();
            move || app::shell(leptos_options.clone())
        })
        .with_state(state);

    let method = req.method().clone();
    // Capture the path before consuming `req`. /case/* responses need
    // tighter Referrer-Policy + no-index headers so the share token
    // doesn't leak to any external link the household clicks.
    let is_case_path = req.uri().path().starts_with("/case/");
    let mut response = router.call(req).await?;
    let headers = response.headers_mut();

    // GET/HEAD render HTML — revalidate on every nav but keep bfcache.
    // Everything else (server-fn POSTs) must never cache.
    // `http::Method` is a newtype over a private enum, so it isn't a valid
    // const-pattern in a match arm — use equality comparison instead.
    let cache_value = if method == axum::http::Method::GET || method == axum::http::Method::HEAD {
        "private, no-cache"
    } else {
        "no-store"
    };
    headers.insert(CACHE_CONTROL, HeaderValue::from_static(cache_value));
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    // Default Referrer-Policy is strict-origin-when-cross-origin; /case/*
    // pages escalate to no-referrer so the share token never appears in
    // a Referer header if the household clicks an external link.
    let referrer_policy = if is_case_path {
        "no-referrer"
    } else {
        "strict-origin-when-cross-origin"
    };
    headers.insert(REFERRER_POLICY, HeaderValue::from_static(referrer_policy));
    if is_case_path {
        // Stop every major crawler from indexing case pages even if a
        // share URL gets pasted into a public chat by mistake.
        headers.insert(
            HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow, noarchive"),
        );
    }

    // CSP: all resources are self-hosted; WASM needs 'wasm-unsafe-eval' to
    // instantiate; the Leptos hydration bootstrap inlines a <script type=
    // "module"> so 'unsafe-inline' is required until that moves to an
    // external file. `frame-ancestors 'none'` + X-Frame-Options: DENY block
    // clickjacking. `object-src 'none'` + `base-uri 'self'` narrow legacy
    // plugin and <base> hijack surfaces.
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; \
             style-src 'self'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             object-src 'none'; \
             base-uri 'self'",
        ),
    );
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "camera=(), microphone=(), geolocation=(), interest-cohort=()",
        ),
    );

    Ok(response)
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
