pub mod activity;
pub mod dashboard;
pub mod households;
pub mod placements;
pub mod resources;
pub mod sitrep;
pub mod state;

pub use state::AppState;

use leptos::prelude::ServerFnError;
use serde::Deserialize;

/// Shared shape for `SELECT <column> AS key, COUNT(*) AS n … GROUP BY <column>`
/// aggregates the entity modules use to build stage/status counts.
#[derive(Debug, Deserialize)]
pub(crate) struct GroupCountRow {
    pub key: String,
    pub n: u32,
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Client(String),
    Internal {
        context: &'static str,
        source: String,
    },
}

impl AppError {
    pub fn client(message: impl Into<String>) -> Self {
        Self::Client(message.into())
    }

    pub fn internal(context: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Internal {
            context,
            source: error.to_string(),
        }
    }
}

pub fn server_error(error: AppError) -> ServerFnError {
    match error {
        AppError::Client(message) => ServerFnError::ServerError(message),
        AppError::Internal { context, source } => {
            worker::console_error!("{context}: {source}");
            ServerFnError::ServerError("Request failed. Try again later.".to_string())
        }
    }
}

pub(crate) fn app_state() -> AppResult<AppState> {
    leptos::prelude::use_context::<AppState>().ok_or_else(|| {
        AppError::internal(
            "Missing app state in Leptos server function context.",
            "state was not provided to the request",
        )
    })
}

pub(crate) fn database() -> AppResult<worker::D1Database> {
    app_state()?
        .db()
        .map_err(|error| AppError::internal("Failed to access D1 binding from app state.", error))
}

pub(crate) fn d1_error(context: &'static str, error: impl std::fmt::Display) -> AppError {
    AppError::internal(context, error)
}

/// Narrow a free-form string to an allow-list. Used to validate enum-like
/// columns (stage, status, kind) at the server boundary so D1 CHECK
/// constraints are never the first line of defense.
pub(crate) fn validate_allowed<'a>(
    label: &'static str,
    value: &'a str,
    allowed: &[&'static str],
) -> AppResult<&'a str> {
    if allowed.iter().any(|candidate| *candidate == value) {
        Ok(value)
    } else {
        Err(AppError::client(format!(
            "{label} must be one of: {}",
            allowed.join(", ")
        )))
    }
}

pub(crate) fn normalize_text(
    label: &'static str,
    value: &str,
    max_len: usize,
    required: bool,
) -> AppResult<Option<String>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return if required {
            Err(AppError::client(format!("{label} is required.")))
        } else {
            Ok(None)
        };
    }
    if trimmed.chars().count() > max_len {
        return Err(AppError::client(format!(
            "{label} is capped at {max_len} characters."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

pub(crate) fn row_id_arg(id: i64) -> AppResult<worker::D1Type<'static>> {
    let id = i32::try_from(id).map_err(|_| AppError::client("Record id is out of range."))?;
    Ok(worker::D1Type::Integer(id))
}
