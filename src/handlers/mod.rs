//! Mode-specific key event handlers that return declarative [`HandlerResult`](crate::types::HandlerResult)s.

/// Text input handling for NewSession and RenameSession modes.
pub mod input;
/// Layout picker handling for the LayoutSelect mode.
pub mod layout_select;
/// Plugin load-time setup (permissions and subscriptions).
pub mod lifecycle;
/// Session list navigation and operations in Normal mode.
pub mod normal;
