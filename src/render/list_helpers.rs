use std::borrow::Cow;

use zellij_tile::prelude::*;

use crate::config::PluginConfig;

/// A colored tag appended after a list item name (e.g. "(active)", "(dead)").
pub struct ListMarker {
  /// The marker text to display.
  pub text: &'static str,
  /// The color index for this marker.
  pub color: usize,
}

/// A single entry in a rendered list, with an optional trailing marker.
pub struct ListItem<'a> {
  /// The display name of this list item.
  pub name: &'a str,
  /// An optional colored marker appended after the name.
  pub marker: Option<ListMarker>,
}

/// Calculates the left gutter width, ensuring enough room for the selection
/// prefix (minimum 3 columns) or using the configured padding.
pub fn calculate_gutter(selection_prefix: &str, list_padding: usize) -> usize {
  let prefix_width = if selection_prefix.is_empty() { 0 } else { 3 };
  list_padding.max(prefix_width)
}

/// Returns how many items to skip at the top so the selected item stays visible.
pub fn calculate_scroll_offset(selected_index: usize, list_height: usize) -> usize {
  if selected_index >= list_height {
    selected_index - list_height + 1
  } else {
    0
  }
}

/// Returns the number of list rows available after subtracting header and
/// footer chrome, with a minimum of 1.
pub fn calculate_list_height(rows: usize) -> usize {
  rows.saturating_sub(5).max(1)
}

/// Builds the display string for a single list line, including an optional
/// index number prefix and trailing marker text. Returns a borrowed reference
/// when no prefix or marker is needed, avoiding allocation.
pub fn build_line_content<'a>(
  index: usize,
  item: &ListItem<'a>,
  show_index_numbers: bool,
  idx_width: usize,
) -> Cow<'a, str> {
  if !show_index_numbers && item.marker.is_none() {
    return Cow::Borrowed(item.name);
  }

  let idx_part = if show_index_numbers {
    format!("{:>width$} ", index + 1, width = idx_width)
  } else {
    String::new()
  };
  let marker_text = item.marker.as_ref().map_or("", |m| m.text);

  Cow::Owned(format!("{}{}{}", idx_part, item.name, marker_text))
}

/// Renders a scrollable list of items with selection highlighting, index
/// numbers, markers, and an empty-state message.
pub fn render_list(
  items: &[ListItem<'_>],
  selected_index: usize,
  empty_message: &str,
  config: &PluginConfig,
  rows: usize,
  cols: usize,
) {
  let gutter = calculate_gutter(&config.selection_prefix, config.list_padding);

  if items.is_empty() {
    let empty = Text::new(empty_message);

    print_text_with_coordinates(empty, gutter, 2, Some(cols.saturating_sub(gutter)), None);

    return;
  }

  let list_height = calculate_list_height(rows);
  let scroll_offset = calculate_scroll_offset(selected_index, list_height);

  let total = items.len();
  let idx_width = if config.show_index_numbers {
    match total {
      0..=9 => 1,
      10..=99 => 2,
      _ => 3,
    }
  } else {
    0
  };

  items
    .iter()
    .enumerate()
    .skip(scroll_offset)
    .take(list_height)
    .for_each(|(i, item)| {
      let is_selected = i == selected_index;
      let y = 2 + i - scroll_offset;

      if is_selected && !config.selection_prefix.is_empty() {
        let prefix_x = gutter.saturating_sub(3);
        let prefix_text = Text::new(&config.selection_prefix);
        print_text_with_coordinates(prefix_text, prefix_x, y, None, None);
      }

      let content = build_line_content(i, item, config.show_index_numbers, idx_width);
      let mut text = Text::new(&content);

      if is_selected && config.show_selection_highlight {
        text = text.selected();
      }

      if config.show_index_numbers {
        text = text.color_range(config.index_number_color, 0..idx_width);
      }

      if let Some(marker) = &item.marker {
        let marker_start = content.len() - marker.text.len();
        text = text.color_range(marker.color, marker_start..content.len());
      }

      print_text_with_coordinates(text, gutter, y, Some(cols.saturating_sub(gutter)), None);
    });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gutter_uses_padding_when_no_prefix() {
    assert_eq!(calculate_gutter("", 4), 4);
  }

  #[test]
  fn gutter_minimum_3_when_prefix_set() {
    assert_eq!(calculate_gutter(">", 1), 3);
  }

  #[test]
  fn gutter_uses_padding_when_larger_than_prefix_width() {
    assert_eq!(calculate_gutter(">", 5), 5);
  }

  #[test]
  fn scroll_offset_zero_when_in_view() {
    assert_eq!(calculate_scroll_offset(0, 10), 0);
    assert_eq!(calculate_scroll_offset(5, 10), 0);
    assert_eq!(calculate_scroll_offset(9, 10), 0);
  }

  #[test]
  fn scroll_offset_scrolls_when_past_view() {
    assert_eq!(calculate_scroll_offset(10, 10), 1);
    assert_eq!(calculate_scroll_offset(15, 10), 6);
  }

  #[test]
  fn list_height_subtracts_chrome() {
    assert_eq!(calculate_list_height(20), 15);
    assert_eq!(calculate_list_height(10), 5);
  }

  #[test]
  fn list_height_minimum_one() {
    assert_eq!(calculate_list_height(3), 1);
    assert_eq!(calculate_list_height(0), 1);
  }

  #[test]
  fn line_content_with_index_numbers() {
    let item = ListItem {
      name: "my-session",
      marker: None,
    };
    let content = build_line_content(0, &item, true, 1);
    assert_eq!(content, "1 my-session");
  }

  #[test]
  fn line_content_without_index_numbers() {
    let item = ListItem {
      name: "my-session",
      marker: None,
    };
    let content = build_line_content(0, &item, false, 0);
    assert_eq!(content, "my-session");
  }

  #[test]
  fn line_content_with_marker() {
    let item = ListItem {
      name: "my-session",
      marker: Some(ListMarker {
        text: " (active)",
        color: 1,
      }),
    };
    let content = build_line_content(0, &item, false, 0);
    assert_eq!(content, "my-session (active)");
  }

  #[test]
  fn line_content_with_index_and_marker() {
    let item = ListItem {
      name: "session",
      marker: Some(ListMarker {
        text: " (dead)",
        color: 2,
      }),
    };
    let content = build_line_content(2, &item, true, 2);
    assert_eq!(content, " 3 session (dead)");
  }

  #[test]
  fn render_list_with_selection_prefix() {
    let items = vec![
      ListItem {
        name: "session1",
        marker: None,
      },
      ListItem {
        name: "session2",
        marker: None,
      },
    ];
    let config = PluginConfig {
      selection_prefix: ">".to_string(),
      ..Default::default()
    };
    render_list(&items, 0, "empty", &config, 20, 80);
  }

  #[test]
  fn render_list_with_selection_highlight_and_marker() {
    let items = vec![ListItem {
      name: "session1",
      marker: Some(ListMarker {
        text: " (active)",
        color: 1,
      }),
    }];
    let config = PluginConfig::default();
    render_list(&items, 0, "empty", &config, 20, 80);
  }

  #[test]
  fn render_list_empty() {
    let items: Vec<ListItem<'_>> = vec![];
    let config = PluginConfig::default();
    render_list(&items, 0, "No sessions", &config, 20, 80);
  }

  #[test]
  fn render_list_scrolls_when_selected_past_view() {
    let names: Vec<String> = (0..20).map(|i| format!("s{}", i)).collect();
    let items: Vec<ListItem<'_>> = names
      .iter()
      .map(|name| ListItem {
        name: name.as_str(),
        marker: None,
      })
      .collect();
    let config = PluginConfig::default();
    // list_height for rows=10 is 5, selected_index=15 should scroll
    render_list(&items, 15, "empty", &config, 10, 80);
  }

  #[test]
  fn line_content_pads_index_to_width() {
    let item = ListItem {
      name: "s",
      marker: None,
    };
    let content = build_line_content(0, &item, true, 2);
    assert_eq!(content, " 1 s");

    let content = build_line_content(9, &item, true, 2);
    assert_eq!(content, "10 s");
  }
}
