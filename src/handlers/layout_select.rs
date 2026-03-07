use zellij_tile::prelude::*;

use crate::types::{HandlerResult, ModeTransition, PluginAction};

/// Handles key events in LayoutSelect mode (layout picker for new sessions).
#[derive(Default)]
pub struct LayoutSelectHandler {
  /// Index of the currently highlighted layout.
  pub selected_index: usize,
  /// Name of the session being created.
  pub session_name: String,
}

impl LayoutSelectHandler {
  /// Initializes the handler for a new layout selection with the given session name.
  pub fn start(&mut self, session_name: String) {
    self.session_name = session_name;
    self.selected_index = 0;
  }

  /// Processes a key press in LayoutSelect mode, handling navigation,
  /// layout selection, and cancellation.
  pub fn handle_key(&mut self, key: BareKey, layouts: &[LayoutInfo]) -> HandlerResult {
    match key {
      BareKey::Esc => {
        self.session_name.clear();
        self.selected_index = 0;

        HandlerResult::render().with_transition(ModeTransition::Normal)
      }
      BareKey::Char('j') | BareKey::Down => {
        if !layouts.is_empty() && self.selected_index < layouts.len() - 1 {
          self.selected_index += 1;
        }

        HandlerResult::render()
      }
      BareKey::Char('k') | BareKey::Up => {
        if self.selected_index > 0 {
          self.selected_index -= 1;
        }

        HandlerResult::render()
      }
      BareKey::Enter => {
        if let Some(layout) = layouts.get(self.selected_index) {
          HandlerResult::no_render()
            .with_action(PluginAction::SwitchSessionWithLayout(
              std::mem::take(&mut self.session_name),
              layout.clone(),
            ))
            .with_action(PluginAction::HideSelf)
        } else {
          HandlerResult::no_render()
        }
      }
      BareKey::Char(c @ '1'..='9') => {
        let index = (c as usize) - ('1' as usize);

        if let Some(layout) = layouts.get(index) {
          self.selected_index = index;

          HandlerResult::no_render()
            .with_action(PluginAction::SwitchSessionWithLayout(
              std::mem::take(&mut self.session_name),
              layout.clone(),
            ))
            .with_action(PluginAction::HideSelf)
        } else {
          HandlerResult::no_render()
        }
      }
      _ => HandlerResult::no_render(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn j_moves_down() {
    let layouts = vec![
      LayoutInfo::BuiltIn("default".to_string()),
      LayoutInfo::BuiltIn("compact".to_string()),
    ];
    let mut handler = LayoutSelectHandler::default();
    let result = handler.handle_key(BareKey::Char('j'), &layouts);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn j_at_bottom_stays() {
    let layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let mut handler = LayoutSelectHandler::default();
    let result = handler.handle_key(BareKey::Char('j'), &layouts);

    assert!(result.render);
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn k_moves_up() {
    let layouts = vec![
      LayoutInfo::BuiltIn("default".to_string()),
      LayoutInfo::BuiltIn("compact".to_string()),
    ];
    let mut handler = LayoutSelectHandler {
      selected_index: 1,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('k'), &layouts);

    assert!(result.render);
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn k_at_top_stays() {
    let layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let mut handler = LayoutSelectHandler::default();
    let result = handler.handle_key(BareKey::Char('k'), &layouts);

    assert!(result.render);
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn enter_creates_session() {
    let layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let mut handler = LayoutSelectHandler {
      session_name: "my-session".to_string(),
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Enter, &layouts);

    assert!(!result.render);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::SwitchSessionWithLayout(
          "my-session".to_string(),
          LayoutInfo::BuiltIn("default".to_string()),
        ),
        PluginAction::HideSelf,
      ]
    );
  }

  #[test]
  fn enter_no_layouts_does_nothing() {
    let mut handler = LayoutSelectHandler::default();
    let result = handler.handle_key(BareKey::Enter, &[]);

    assert!(!result.render);
    assert!(result.actions.is_empty());
  }

  #[test]
  fn number_key_selects_layout() {
    let layouts = vec![
      LayoutInfo::BuiltIn("default".to_string()),
      LayoutInfo::BuiltIn("compact".to_string()),
      LayoutInfo::BuiltIn("disable-status-bar".to_string()),
    ];
    let mut handler = LayoutSelectHandler {
      session_name: "my-session".to_string(),
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('2'), &layouts);

    assert!(!result.render);
    assert_eq!(handler.selected_index, 1);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::SwitchSessionWithLayout(
          "my-session".to_string(),
          LayoutInfo::BuiltIn("compact".to_string()),
        ),
        PluginAction::HideSelf,
      ]
    );
  }

  #[test]
  fn number_key_beyond_layouts_does_nothing() {
    let layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let mut handler = LayoutSelectHandler::default();
    let result = handler.handle_key(BareKey::Char('5'), &layouts);

    assert!(!result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn esc_returns_to_normal() {
    let mut handler = LayoutSelectHandler {
      session_name: "leftover".to_string(),
      selected_index: 2,
    };
    let result = handler.handle_key(BareKey::Esc, &[]);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(matches!(result.transition, Some(ModeTransition::Normal)));
    assert!(handler.session_name.is_empty());
    assert_eq!(handler.selected_index, 0);
  }
}
