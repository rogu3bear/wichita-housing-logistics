mod api;
mod app;
mod asset_hashes;
mod build_info;
mod components;
#[cfg(feature = "ssr")]
mod server;

#[cfg(feature = "ssr")]
#[worker::event(fetch)]
async fn fetch(
    req: worker::HttpRequest,
    env: worker::Env,
    _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
    use axum::Router;
    use axum::http::{
        header::{
            CACHE_CONTROL, CONTENT_SECURITY_POLICY, HeaderName, HeaderValue,
            REFERRER_POLICY, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
        },
    };
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_service::Service;

    let conf =
        get_configuration(None).map_err(|error| worker::Error::RustError(error.to_string()))?;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(app::App);
    let state = server::AppState::new(leptos_options.clone(), env);

    let mut router = Router::new()
        .leptos_routes_with_context(&state, routes, || {}, {
            let leptos_options = leptos_options.clone();
            move || app::shell(leptos_options.clone())
        })
        .with_state(state);

    let method = req.method().clone();
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
    headers.insert(
        REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

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
