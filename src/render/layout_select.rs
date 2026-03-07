use crate::config::PluginConfig;
use crate::handlers::layout_select::LayoutSelectHandler;
use crate::session_store::SessionStore;

use super::list_helpers::{self, ListItem, ListMarker};

/// Converts available layouts into [`ListItem`]s, marking built-in layouts
/// with a colored "(built-in)" tag.
pub fn build_layout_items<'a>(store: &'a SessionStore, config: &PluginConfig) -> Vec<ListItem<'a>> {
  store
    .available_layouts
    .iter()
    .map(|layout| ListItem {
      name: layout.name(),
      marker: if layout.is_builtin() {
        Some(ListMarker {
          text: " (built-in)",
          color: config.builtin_marker_color,
        })
      } else {
        None
      },
    })
    .collect()
}

/// Renders the layout selection list for the LayoutSelect mode.
pub fn render_layout_select(
  store: &SessionStore,
  handler: &LayoutSelectHandler,
  config: &PluginConfig,
  rows: usize,
  cols: usize,
) {
  let items = build_layout_items(store, config);

  list_helpers::render_list(
    &items,
    handler.selected_index,
    "No layouts available",
    config,
    rows,
    cols,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::session_store::SessionStore;
  use zellij_tile::prelude::*;

  #[test]
  fn builds_builtin_layout_with_marker() {
    let store = SessionStore {
      available_layouts: vec![LayoutInfo::BuiltIn("default".to_string())],
      ..Default::default()
    };
    let config = PluginConfig::default();
    let items = build_layout_items(&store, &config);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "default");
    assert_eq!(items[0].marker.as_ref().unwrap().text, " (built-in)");
  }

  #[test]
  fn builds_custom_layout_without_marker() {
    let store = SessionStore {
      available_layouts: vec![LayoutInfo::File("custom".to_string())],
      ..Default::default()
    };
    let config = PluginConfig::default();
    let items = build_layout_items(&store, &config);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "custom");
    assert!(items[0].marker.is_none());
  }

  #[test]
  fn builds_empty_when_no_layouts() {
    let store = SessionStore::default();
    let config = PluginConfig::default();
    let items = build_layout_items(&store, &config);

    assert!(items.is_empty());
  }

  #[test]
  fn builds_mixed_builtin_and_custom() {
    let store = SessionStore {
      available_layouts: vec![
        LayoutInfo::BuiltIn("default".to_string()),
        LayoutInfo::File("my-layout".to_string()),
      ],
      ..Default::default()
    };
    let config = PluginConfig::default();
    let items = build_layout_items(&store, &config);

    assert_eq!(items.len(), 2);
    assert!(items[0].marker.is_some());
    assert!(items[1].marker.is_none());
  }

  #[test]
  fn uses_config_color_for_builtin_marker() {
    let store = SessionStore {
      available_layouts: vec![LayoutInfo::BuiltIn("default".to_string())],
      ..Default::default()
    };
    let config = PluginConfig {
      builtin_marker_color: 3,
      ..Default::default()
    };
    let items = build_layout_items(&store, &config);

    assert_eq!(items[0].marker.as_ref().unwrap().color, 3);
  }
}
