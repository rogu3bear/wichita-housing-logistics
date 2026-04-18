//! Build-time identity baked into the Worker binary.
//!
//! `VERSION` comes from `CARGO_PKG_VERSION` — always available.
//! `COMMIT_SHA` is a short git SHA exported by `scripts/build-edge.sh`;
//! defaults to "unknown" when the build happens outside a git tree (e.g.
//! a cached CI environment that doesn't preserve `.git`).
//!
//! Surface this as a footer so a case manager reporting an issue can
//! quote the revision they hit.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const COMMIT_SHA: &str = match option_env!("GIT_COMMIT_SHA") {
    Some(sha) => sha,
    None => "unknown",
};

/// URL the footer links to for operator feedback. Kept as a compile-time
/// const so every deployed build points at the same issue tracker even
/// if the browser's view is stale.
pub const FEEDBACK_URL: &str =
    "https://github.com/rogu3bear/wichita-housing-logistics/issues/new";
