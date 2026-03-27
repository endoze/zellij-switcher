use zellij_tile::prelude::*;

use crate::types::{HandlerResult, PluginAction};

/// Extracts the 0-indexed position of the currently active tab.
pub fn active_tab_position(tab_infos: &[TabInfo]) -> Option<usize> {
  tab_infos.iter().find(|t| t.active).map(|t| t.position)
}

/// Handles a visibility change event by moving the plugin pane to the
/// active tab when it becomes visible. Returns a render trigger on
/// visibility gain so the UI refreshes.
pub fn handle_visible(
  visible: bool,
  plugin_pane_id: Option<PaneId>,
  active_tab_position: Option<usize>,
) -> HandlerResult {
  if visible && let (Some(pane_id), Some(tab_pos)) = (plugin_pane_id, active_tab_position) {
    return HandlerResult::render()
      .with_action(PluginAction::MoveToTab(pane_id, tab_pos))
      .with_action(PluginAction::ShowSelf);
  }

  HandlerResult::render_if(visible)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_tab_info(position: usize, active: bool) -> TabInfo {
    TabInfo {
      position,
      active,
      name: format!("tab-{}", position),
      panes_to_hide: 0,
      is_fullscreen_active: false,
      is_sync_panes_active: false,
      are_floating_panes_visible: false,
      other_focused_clients: Vec::new(),
      active_swap_layout_name: None,
      is_swap_layout_dirty: false,
      viewport_rows: 24,
      viewport_columns: 80,
      display_area_rows: 24,
      display_area_columns: 80,
      selectable_tiled_panes_count: 0,
      selectable_floating_panes_count: 0,
      tab_id: position,
      has_bell_notification: false,
      is_flashing_bell: false,
    }
  }

  #[test]
  fn active_tab_position_finds_active() {
    let tabs = vec![make_tab_info(0, false), make_tab_info(1, true)];

    assert_eq!(active_tab_position(&tabs), Some(1));
  }

  #[test]
  fn active_tab_position_none_when_empty() {
    assert_eq!(active_tab_position(&[]), None);
  }

  #[test]
  fn active_tab_position_none_when_no_active() {
    let tabs = vec![make_tab_info(0, false), make_tab_info(1, false)];

    assert_eq!(active_tab_position(&tabs), None);
  }

  #[test]
  fn visible_with_pane_and_tab_moves_and_shows() {
    let pane_id = PaneId::Plugin(42);
    let result = handle_visible(true, Some(pane_id), Some(2));

    assert!(result.render);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::MoveToTab(PaneId::Plugin(42), 2),
        PluginAction::ShowSelf,
      ]
    );
  }

  #[test]
  fn visible_without_pane_id_still_renders() {
    let result = handle_visible(true, None, Some(2));

    assert!(result.render);
    assert!(result.actions.is_empty());
  }

  #[test]
  fn visible_without_tab_position_still_renders() {
    let pane_id = PaneId::Plugin(42);
    let result = handle_visible(true, Some(pane_id), None);

    assert!(result.render);
    assert!(result.actions.is_empty());
  }

  #[test]
  fn invisible_does_not_render() {
    let pane_id = PaneId::Plugin(42);
    let result = handle_visible(false, Some(pane_id), Some(2));

    assert!(!result.render);
    assert!(result.actions.is_empty());
  }
}
