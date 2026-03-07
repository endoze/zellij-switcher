use std::collections::BTreeMap;

/// User-configurable settings parsed from the Zellij plugin configuration map.
pub struct PluginConfig {
  /// Color index for the header title text (0–3).
  pub header_text_color: usize,
  /// Color index for keyboard shortcut labels in the footer (0–3).
  pub shortcut_key_color: usize,
  /// Color index for list item index numbers (0–3).
  pub index_number_color: usize,
  /// Color index for the "(active)" marker on the current session (0–3).
  pub active_marker_color: usize,
  /// Color index for the "(dead)" marker on resurrectable sessions (0–3).
  pub dead_marker_color: usize,
  /// Color index for the "(built-in)" marker on built-in layouts (0–3).
  pub builtin_marker_color: usize,
  /// Color index for transient hint messages (0–3).
  pub hint_message_color: usize,
  /// Color index for input prompt labels (0–3).
  pub prompt_label_color: usize,
  /// Title displayed in the centered header.
  pub header_title: String,
  /// Whether to show 1-based index numbers beside list items.
  pub show_index_numbers: bool,
  /// Whether to show the j/k navigation help line in the footer.
  pub show_navigation_help: bool,
  /// Prefix string rendered beside the selected list item.
  pub selection_prefix: String,
  /// Whether to highlight the selected list item row.
  pub show_selection_highlight: bool,
  /// Left padding (in columns) for list content.
  pub list_padding: usize,
}

/// Provides sensible defaults for all configuration fields.
impl Default for PluginConfig {
  /// Returns the default plugin configuration.
  fn default() -> Self {
    Self {
      header_text_color: 0,
      shortcut_key_color: 0,
      index_number_color: 0,
      active_marker_color: 1,
      dead_marker_color: 1,
      builtin_marker_color: 1,
      hint_message_color: 2,
      prompt_label_color: 0,
      header_title: "Session Manager".to_string(),
      show_index_numbers: true,
      show_navigation_help: true,
      selection_prefix: String::new(),
      show_selection_highlight: true,
      list_padding: 2,
    }
  }
}

/// Parses a color index from the config map, clamping to the 0–3 range.
fn parse_color(map: &BTreeMap<String, String>, key: &str, default: usize) -> usize {
  map
    .get(key)
    .and_then(|v| v.parse::<usize>().ok())
    .map(|v| v.min(3))
    .unwrap_or(default)
}

/// Parses a boolean value from the config map (case-insensitive "true"/"false").
fn parse_bool(map: &BTreeMap<String, String>, key: &str, default: bool) -> bool {
  match map.get(key) {
    Some(v) if v.eq_ignore_ascii_case("true") => true,
    Some(v) if v.eq_ignore_ascii_case("false") => false,
    _ => default,
  }
}

/// Constructs a [`PluginConfig`] from a Zellij plugin configuration map.
impl From<BTreeMap<String, String>> for PluginConfig {
  /// Parses all known keys from the map, falling back to defaults for missing or invalid values.
  fn from(mut map: BTreeMap<String, String>) -> Self {
    let defaults = Self::default();

    Self {
      header_text_color: parse_color(&map, "header_text_color", defaults.header_text_color),
      shortcut_key_color: parse_color(&map, "shortcut_key_color", defaults.shortcut_key_color),
      index_number_color: parse_color(&map, "index_number_color", defaults.index_number_color),
      active_marker_color: parse_color(&map, "active_marker_color", defaults.active_marker_color),
      dead_marker_color: parse_color(&map, "dead_marker_color", defaults.dead_marker_color),
      builtin_marker_color: parse_color(
        &map,
        "builtin_marker_color",
        defaults.builtin_marker_color,
      ),
      hint_message_color: parse_color(&map, "hint_message_color", defaults.hint_message_color),
      prompt_label_color: parse_color(&map, "prompt_label_color", defaults.prompt_label_color),
      header_title: map.remove("header_title").unwrap_or(defaults.header_title),
      show_index_numbers: parse_bool(&map, "show_index_numbers", defaults.show_index_numbers),
      show_navigation_help: parse_bool(&map, "show_navigation_help", defaults.show_navigation_help),
      selection_prefix: map
        .remove("selection_prefix")
        .map(|v| {
          let trimmed = v.trim();
          if trimmed.len() == v.len() {
            v
          } else {
            trimmed.to_string()
          }
        })
        .unwrap_or(defaults.selection_prefix),
      show_selection_highlight: parse_bool(
        &map,
        "show_selection_highlight",
        defaults.show_selection_highlight,
      ),
      list_padding: map
        .get("list_padding")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(defaults.list_padding),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn empty_map_gives_defaults() {
    let config = PluginConfig::from(BTreeMap::new());
    let defaults = PluginConfig::default();

    assert_eq!(config.header_text_color, defaults.header_text_color);
    assert_eq!(config.shortcut_key_color, defaults.shortcut_key_color);
    assert_eq!(config.index_number_color, defaults.index_number_color);
    assert_eq!(config.active_marker_color, defaults.active_marker_color);
    assert_eq!(config.dead_marker_color, defaults.dead_marker_color);
    assert_eq!(config.builtin_marker_color, defaults.builtin_marker_color);
    assert_eq!(config.hint_message_color, defaults.hint_message_color);
    assert_eq!(config.prompt_label_color, defaults.prompt_label_color);
    assert_eq!(config.header_title, defaults.header_title);
    assert_eq!(config.show_index_numbers, defaults.show_index_numbers);
    assert_eq!(config.show_navigation_help, defaults.show_navigation_help);
    assert_eq!(config.selection_prefix, defaults.selection_prefix);
    assert_eq!(
      config.show_selection_highlight,
      defaults.show_selection_highlight
    );
  }

  #[test]
  fn parses_color_values() {
    let mut map = BTreeMap::new();
    map.insert("header_text_color".to_string(), "3".to_string());
    map.insert("shortcut_key_color".to_string(), "2".to_string());
    map.insert("index_number_color".to_string(), "1".to_string());

    let config = PluginConfig::from(map);

    assert_eq!(config.header_text_color, 3);
    assert_eq!(config.shortcut_key_color, 2);
    assert_eq!(config.index_number_color, 1);
  }

  #[test]
  fn clamps_colors_to_max_3() {
    let mut map = BTreeMap::new();
    map.insert("header_text_color".to_string(), "99".to_string());
    map.insert("shortcut_key_color".to_string(), "10".to_string());

    let config = PluginConfig::from(map);

    assert_eq!(config.header_text_color, 3);
    assert_eq!(config.shortcut_key_color, 3);
  }

  #[test]
  fn invalid_color_falls_back_to_default() {
    let mut map = BTreeMap::new();
    map.insert("header_text_color".to_string(), "abc".to_string());
    map.insert("shortcut_key_color".to_string(), "".to_string());

    let config = PluginConfig::from(map);

    assert_eq!(config.header_text_color, 0);
    assert_eq!(config.shortcut_key_color, 0);
  }

  #[test]
  fn parses_bool_values() {
    let mut map = BTreeMap::new();
    map.insert("show_index_numbers".to_string(), "false".to_string());
    map.insert("show_navigation_help".to_string(), "FALSE".to_string());
    map.insert("show_selection_highlight".to_string(), "True".to_string());

    let config = PluginConfig::from(map);

    assert!(!config.show_index_numbers);
    assert!(!config.show_navigation_help);
    assert!(config.show_selection_highlight);
  }

  #[test]
  fn invalid_bool_falls_back_to_default() {
    let mut map = BTreeMap::new();
    map.insert("show_index_numbers".to_string(), "yes".to_string());

    let config = PluginConfig::from(map);

    // "yes" is unrecognized, so parse_bool falls back to the default (true)
    assert!(config.show_index_numbers);
  }

  #[test]
  fn parses_string_values() {
    let mut map = BTreeMap::new();
    map.insert("header_title".to_string(), "Sessions".to_string());
    map.insert("selection_prefix".to_string(), "> ".to_string());

    let config = PluginConfig::from(map);

    assert_eq!(config.header_title, "Sessions");
    assert_eq!(config.selection_prefix, ">");
  }
}
