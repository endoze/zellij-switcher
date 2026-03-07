//! Rendering functions that produce terminal output via the Zellij text API.

/// Footer rendering with mode-specific shortcuts and input prompts.
mod footer;
/// Centered header title rendering.
mod header;
/// Layout selection list rendering.
mod layout_select;
/// Shared list rendering utilities (scrolling, gutters, line formatting).
mod list_helpers;
/// Session list rendering for Normal/NewSession/RenameSession modes.
mod session_list;

use crate::config::PluginConfig;
use crate::handlers::input::InputHandler;
use crate::handlers::layout_select::LayoutSelectHandler;
use crate::handlers::normal::NormalHandler;
use crate::session_store::SessionStore;
use crate::types::Mode;

/// Renders the full plugin UI (header, list content, and footer) for the
/// current mode. Skips rendering if the terminal is too small.
#[allow(clippy::too_many_arguments)]
pub fn render(
  mode: &Mode,
  store: &SessionStore,
  normal: &NormalHandler,
  input: &InputHandler,
  layout_handler: &LayoutSelectHandler,
  config: &PluginConfig,
  rows: usize,
  cols: usize,
) {
  if rows < 4 || cols < 10 {
    return;
  }

  header::render_header(config, cols);

  match mode {
    Mode::Normal | Mode::NewSession | Mode::RenameSession => {
      session_list::render_session_list(store, normal, config, rows, cols);
    }
    Mode::LayoutSelect => {
      layout_select::render_layout_select(store, layout_handler, config, rows, cols);
    }
  }

  footer::render_footer(mode, normal, input, layout_handler, config, rows, cols);
}

#[cfg(test)]
mod tests {
  use super::*;

  fn default_deps() -> (
    SessionStore,
    NormalHandler,
    InputHandler,
    LayoutSelectHandler,
    PluginConfig,
  ) {
    (
      SessionStore::default(),
      NormalHandler::default(),
      InputHandler::default(),
      LayoutSelectHandler::default(),
      PluginConfig::default(),
    )
  }

  #[test]
  fn skips_render_when_rows_too_small() {
    let (store, normal, input, layout, config) = default_deps();
    // Should return early without panicking
    render(
      &Mode::Normal,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      3,
      80,
    );
  }

  #[test]
  fn skips_render_when_cols_too_small() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::Normal,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      9,
    );
  }

  #[test]
  fn renders_normal_mode() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::Normal,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn renders_new_session_mode() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::NewSession,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn renders_rename_session_mode() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::RenameSession,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn renders_layout_select_mode() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::LayoutSelect,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn renders_with_sessions() {
    use crate::testutil::make_session_store;
    let store = make_session_store(&[("s1", true), ("s2", false)], &["dead1"]);
    let normal = NormalHandler::default();
    let input = InputHandler::default();
    let layout = LayoutSelectHandler::default();
    let config = PluginConfig::default();
    render(
      &Mode::Normal,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn renders_at_boundary_dimensions() {
    let (store, normal, input, layout, config) = default_deps();
    render(
      &Mode::Normal,
      &store,
      &normal,
      &input,
      &layout,
      &config,
      4,
      10,
    );
  }
}
