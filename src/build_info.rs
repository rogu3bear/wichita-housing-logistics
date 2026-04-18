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

/// Stress-mode flag. When true, the TopNav renders a red `<SitrepBanner/>`
/// above the page shell with jump buttons to `/situational` and `/routing`.
/// Operators flip this and redeploy during an active multi-event crisis
/// (severe-weather shelter surge, major intake outage, etc.). Default off
/// so ordinary days stay calm.
pub const SITREP_ACTIVE: bool = false;

/// One-line summary rendered in the banner when `SITREP_ACTIVE` is true.
/// Update at the same time as the flag — the message is the whole point.
pub const SITREP_SUMMARY: &str =
    "Red Cross activation · Open Door offline · CrossRoads 1/12 beds";

pub const COMMIT_SHA: &str = match option_env!("GIT_COMMIT_SHA") {
    Some(sha) => sha,
    None => "unknown",
};

/// URL the footer links to for operator feedback. `mailto:` so a case
/// manager can send context without creating a GitHub account. The
/// subject is pre-filled with the version + commit SHA in the body at
/// render time (see `layout.rs::BuildFooter`).
pub const FEEDBACK_EMAIL: &str = "a.james.apple@icloud.com";
