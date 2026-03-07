use crate::config::PluginConfig;
use crate::handlers::normal::NormalHandler;
use crate::session_store::SessionStore;

use super::list_helpers::{self, ListItem, ListMarker};

/// Converts active and resurrectable sessions into [`ListItem`]s, marking
/// the current session as "(active)" and dead sessions as "(dead)".
pub fn build_session_items<'a>(
  store: &'a SessionStore,
  config: &PluginConfig,
) -> Vec<ListItem<'a>> {
  store
    .sessions
    .iter()
    .map(|s| ListItem {
      name: &s.name,
      marker: if s.is_current_session {
        Some(ListMarker {
          text: " (active)",
          color: config.active_marker_color,
        })
      } else {
        None
      },
    })
    .chain(
      store
        .resurrectable_sessions
        .iter()
        .map(|(name, _)| ListItem {
          name: name.as_str(),
          marker: Some(ListMarker {
            text: " (dead)",
            color: config.dead_marker_color,
          }),
        }),
    )
    .collect()
}

/// Renders the session list for Normal, NewSession, and RenameSession modes.
pub fn render_session_list(
  store: &SessionStore,
  normal: &NormalHandler,
  config: &PluginConfig,
  rows: usize,
  cols: usize,
) {
  let items = build_session_items(store, config);

  list_helpers::render_list(
    &items,
    normal.selected_index,
    "No sessions found",
    config,
    rows,
    cols,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testutil::make_session_store;

  #[test]
  fn builds_active_sessions_with_marker() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let config = PluginConfig::default();
    let items = build_session_items(&store, &config);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "s1");
    assert_eq!(items[0].marker.as_ref().unwrap().text, " (active)");
    assert_eq!(items[1].name, "s2");
    assert!(items[1].marker.is_none());
  }

  #[test]
  fn builds_dead_sessions_with_marker() {
    let store = make_session_store(&[], &["dead1", "dead2"]);
    let config = PluginConfig::default();
    let items = build_session_items(&store, &config);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "dead1");
    assert_eq!(items[0].marker.as_ref().unwrap().text, " (dead)");
    assert_eq!(items[1].name, "dead2");
    assert_eq!(items[1].marker.as_ref().unwrap().text, " (dead)");
  }

  #[test]
  fn builds_mixed_active_and_dead() {
    let store = make_session_store(&[("s1", true)], &["dead1"]);
    let config = PluginConfig::default();
    let items = build_session_items(&store, &config);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].name, "s1");
    assert_eq!(items[0].marker.as_ref().unwrap().text, " (active)");
    assert_eq!(items[1].name, "dead1");
    assert_eq!(items[1].marker.as_ref().unwrap().text, " (dead)");
  }

  #[test]
  fn builds_empty_when_no_sessions() {
    let store = make_session_store(&[], &[]);
    let config = PluginConfig::default();
    let items = build_session_items(&store, &config);

    assert!(items.is_empty());
  }

  #[test]
  fn uses_config_colors_for_markers() {
    let store = make_session_store(&[("s1", true)], &["dead1"]);
    let config = PluginConfig {
      active_marker_color: 3,
      dead_marker_color: 2,
      ..Default::default()
    };
    let items = build_session_items(&store, &config);

    assert_eq!(items[0].marker.as_ref().unwrap().color, 3);
    assert_eq!(items[1].marker.as_ref().unwrap().color, 2);
  }
}
