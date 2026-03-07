//! Zellij plugin for session management — switching, creating, renaming,
//! deleting, and resurrecting sessions.

/// Plugin configuration types and parsing.
pub mod config;
/// Mode-specific key event handlers.
pub mod handlers;
/// Terminal rendering functions.
pub mod render;
/// Session and layout data store.
pub mod session_store;
/// Core enums, actions, and handler result types.
pub mod types;

#[cfg(test)]
pub(crate) mod testutil;
