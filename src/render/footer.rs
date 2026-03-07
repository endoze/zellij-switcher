use zellij_tile::prelude::*;

use crate::config::PluginConfig;
use crate::handlers::input::InputHandler;
use crate::handlers::layout_select::LayoutSelectHandler;
use crate::handlers::normal::NormalHandler;
use crate::types::Mode;

/// Returns the byte offset of a shortcut key within a footer string.
/// Searches for three exact patterns:
/// - ` X: ` — a standalone key (e.g. ` Enter: `, ` n: `)
/// - `/X: ` — second key in a slash pair (e.g. `/k: `)
/// - ` X/` — first key in a slash pair (e.g. ` j/`)
fn find_shortcut_key(text: &str, key: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  let key_len = key.len();
  let mut start = 0;

  // Search for `key` in progressively smaller slices of `text`.
  // `find` returns a position relative to the slice, so `abs` converts
  // it back to an absolute index in the original string.
  while let Some(pos) = text[start..].find(key) {
    let abs = start + pos;
    let after = abs + key_len;
    // Safely get the byte before the match; returns None if the match
    // is at the very start of the string (position 0).
    let before = abs.checked_sub(1).and_then(|i| bytes.get(i).copied());

    // Only accept the match if the surrounding delimiters indicate a
    // shortcut context: " key:" or "/key:" or " key/"
    let matched = matches!(
      (before, bytes.get(after)),
      (Some(b' '), Some(b':')) | (Some(b'/'), Some(b':')) | (Some(b' '), Some(b'/'))
    );

    if matched {
      return Some(abs);
    }

    // Advance past this occurrence so the next iteration doesn't
    // rediscover the same match.
    start = abs + 1;
  }

  None
}

/// Applies shortcut key coloring to a [`Text`] value for each key that
/// matches the ` {key}:` pattern in the text content.
fn color_shortcut_keys(text: Text, content: &str, color: usize, keys: &[&str]) -> Text {
  keys.iter().fold(text, |t, key| {
    if let Some(start) = find_shortcut_key(content, key) {
      t.color_range(color, start..start + key.len())
    } else {
      t
    }
  })
}

/// Renders an input-mode footer with a prompt label, the current buffer
/// contents, and an action hint line.
fn render_input_footer(
  label: &str,
  action_verb: &str,
  input: &InputHandler,
  config: &PluginConfig,
  footer_y: usize,
  cols: usize,
) {
  let prompt = Text::new(format!("{}{}|", label, input.buffer))
    .color_range(config.prompt_label_color, ..label.len());
  let hint_content = format!("  Enter: {}  Esc: cancel", action_verb);
  let hint = color_shortcut_keys(
    Text::new(&hint_content),
    &hint_content,
    config.shortcut_key_color,
    &["Enter", "Esc"],
  );

  print_text_with_coordinates(prompt, 0, footer_y, Some(cols), None);
  print_text_with_coordinates(hint, 0, footer_y + 1, Some(cols), None);
}

/// Renders the footer area with mode-specific content: action shortcuts,
/// input prompts, or layout selection hints.
pub fn render_footer(
  mode: &Mode,
  normal: &NormalHandler,
  input: &InputHandler,
  layout_handler: &LayoutSelectHandler,
  config: &PluginConfig,
  rows: usize,
  cols: usize,
) {
  let footer_y = rows.saturating_sub(2);

  match mode {
    Mode::Normal => {
      if let Some(hint) = &normal.hint_message {
        let hint_text = Text::new(format!("  {}", hint)).color_range(config.hint_message_color, ..);
        print_text_with_coordinates(hint_text, 0, footer_y.saturating_sub(1), Some(cols), None);
      }

      let actions_text = "  Enter: switch  n: new  r: rename  d: delete  Esc: close";
      let color = config.shortcut_key_color;
      let actions = color_shortcut_keys(
        Text::new(actions_text),
        actions_text,
        color,
        &["Enter", "n", "r", "d", "Esc"],
      );

      print_text_with_coordinates(actions, 0, footer_y, Some(cols), None);

      if config.show_navigation_help {
        let nav_text = "  j/k: navigate  1-9: quick switch";
        let nav = color_shortcut_keys(
          Text::new(nav_text),
          nav_text,
          config.shortcut_key_color,
          &["j", "k", "1-9"],
        );

        print_text_with_coordinates(nav, 0, footer_y + 1, Some(cols), None);
      }
    }
    Mode::NewSession => {
      render_input_footer(
        "  New session name: ",
        "create",
        input,
        config,
        footer_y,
        cols,
      );
    }
    Mode::RenameSession => {
      render_input_footer(
        "  Rename session: ",
        "rename",
        input,
        config,
        footer_y,
        cols,
      );
    }
    Mode::LayoutSelect => {
      let prompt = Text::new(format!(
        "  Select layout for '{}'",
        layout_handler.session_name
      ))
      .color_range(config.prompt_label_color, ..);
      let hint_text = "  Enter: create  Esc: cancel  j/k: navigate";
      let hint = color_shortcut_keys(
        Text::new(hint_text),
        hint_text,
        config.shortcut_key_color,
        &["Enter", "Esc", "j", "k"],
      );

      print_text_with_coordinates(prompt, 0, footer_y, Some(cols), None);
      print_text_with_coordinates(hint, 0, footer_y + 1, Some(cols), None);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn default_deps() -> (
    NormalHandler,
    InputHandler,
    LayoutSelectHandler,
    PluginConfig,
  ) {
    (
      NormalHandler::default(),
      InputHandler::default(),
      LayoutSelectHandler::default(),
      PluginConfig::default(),
    )
  }

  #[test]
  fn find_shortcut_key_space_colon() {
    let text = "  Enter: switch  n: new  r: rename  d: delete  Esc: close";
    assert_eq!(find_shortcut_key(text, "Enter"), Some(2));
    assert_eq!(find_shortcut_key(text, "n"), Some(17));
    assert_eq!(find_shortcut_key(text, "r"), Some(25));
    assert_eq!(find_shortcut_key(text, "d"), Some(36));
    assert_eq!(find_shortcut_key(text, "Esc"), Some(47));
  }

  #[test]
  fn find_shortcut_key_slash_separated() {
    let text = "  j/k: navigate  1-9: quick switch";
    assert_eq!(find_shortcut_key(text, "j"), Some(2));
    assert_eq!(find_shortcut_key(text, "k"), Some(4));
    assert_eq!(find_shortcut_key(text, "1-9"), Some(17));
  }

  #[test]
  fn find_shortcut_key_not_found() {
    let text = "  Enter: switch";
    assert_eq!(find_shortcut_key(text, "z"), None);
  }

  #[test]
  fn render_normal_mode_does_not_panic() {
    let (normal, input, layout, config) = default_deps();
    render_footer(&Mode::Normal, &normal, &input, &layout, &config, 20, 80);
  }

  #[test]
  fn render_normal_mode_with_hint_does_not_panic() {
    let (mut normal, input, layout, config) = default_deps();
    normal.hint_message = Some("Can only rename current session");
    render_footer(&Mode::Normal, &normal, &input, &layout, &config, 20, 80);
  }

  #[test]
  fn render_normal_mode_without_nav_help_does_not_panic() {
    let (normal, input, layout, mut config) = default_deps();
    config.show_navigation_help = false;
    render_footer(&Mode::Normal, &normal, &input, &layout, &config, 20, 80);
  }

  #[test]
  fn render_new_session_mode_does_not_panic() {
    let (normal, mut input, layout, config) = default_deps();
    input.buffer = "my-session".to_string();
    render_footer(&Mode::NewSession, &normal, &input, &layout, &config, 20, 80);
  }

  #[test]
  fn render_rename_session_mode_does_not_panic() {
    let (normal, mut input, layout, config) = default_deps();
    input.buffer = "new-name".to_string();
    render_footer(
      &Mode::RenameSession,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn render_layout_select_mode_does_not_panic() {
    let (normal, input, mut layout, config) = default_deps();
    layout.session_name = "test-session".to_string();
    render_footer(
      &Mode::LayoutSelect,
      &normal,
      &input,
      &layout,
      &config,
      20,
      80,
    );
  }

  #[test]
  fn render_with_small_rows_does_not_panic() {
    let (normal, input, layout, config) = default_deps();
    render_footer(&Mode::Normal, &normal, &input, &layout, &config, 1, 80);
  }
}
