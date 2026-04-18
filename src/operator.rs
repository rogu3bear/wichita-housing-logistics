//! Browser-persisted "who am I" identity for activity notes.
//!
//! `load()` returns the operator's most recent author string from
//! `localStorage["whl.author"]` on the client. Server-side the Worker
//! runs in a wasm VM with no DOM; `web_sys::window()` returns None and
//! both functions no-op. Keeps two case managers sharing the same
//! workers.dev URL from silently overwriting each other's convention.

const KEY: &str = "whl.author";

pub fn load() -> Option<String> {
    let storage = leptos::web_sys::window()?.local_storage().ok().flatten()?;
    storage
        .get_item(KEY)
        .ok()
        .flatten()
        .filter(|s| !s.is_empty())
}

pub fn save(name: &str) {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some(storage) = leptos::web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(KEY, trimmed);
    }
}
