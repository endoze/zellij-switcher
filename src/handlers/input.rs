use zellij_tile::prelude::*;

use crate::types::{HandlerResult, Mode, ModeTransition, PluginAction};

/// Handles text input key events for NewSession and RenameSession modes.
#[derive(Default)]
pub struct InputHandler {
  /// The current text input buffer.
  pub buffer: String,
}

impl InputHandler {
  /// Clears the input buffer.
  pub fn clear(&mut self) {
    self.buffer.clear();
  }

  /// Processes a key press in an input mode, handling character entry,
  /// backspace, escape (cancel), and enter (submit).
  pub fn handle_key(&mut self, key: BareKey, mode: &Mode) -> HandlerResult {
    match key {
      BareKey::Esc => {
        self.buffer.clear();

        HandlerResult::render().with_transition(ModeTransition::Normal)
      }
      BareKey::Backspace => {
        self.buffer.pop();

        HandlerResult::render()
      }
      BareKey::Enter => {
        if !self.buffer.is_empty() {
          match mode {
            Mode::NewSession => {
              let name = std::mem::take(&mut self.buffer);

              HandlerResult::render().with_transition(ModeTransition::LayoutSelect(name))
            }
            Mode::RenameSession => {
              let name = std::mem::take(&mut self.buffer);

              HandlerResult::render()
                .with_action(PluginAction::RenameSession(name))
                .with_transition(ModeTransition::Normal)
            }
            Mode::Normal | Mode::LayoutSelect => unreachable!(),
          }
        } else {
          HandlerResult::render()
        }
      }
      BareKey::Char(c) => {
        if !c.is_control() && self.buffer.len() < 128 {
          self.buffer.push(c);
        }

        HandlerResult::render()
      }
      _ => HandlerResult::no_render(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn char_appends() {
    let mut handler = InputHandler::default();
    let result = handler.handle_key(BareKey::Char('a'), &Mode::NewSession);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.buffer, "a");

    handler.handle_key(BareKey::Char('b'), &Mode::NewSession);

    assert_eq!(handler.buffer, "ab");
  }

  #[test]
  fn backspace_removes() {
    let mut handler = InputHandler {
      buffer: "abc".to_string(),
    };
    let result = handler.handle_key(BareKey::Backspace, &Mode::NewSession);

    assert!(result.render);
    assert_eq!(handler.buffer, "ab");
  }

  #[test]
  fn backspace_on_empty() {
    let mut handler = InputHandler::default();
    let result = handler.handle_key(BareKey::Backspace, &Mode::NewSession);

    assert!(result.render);
    assert!(handler.buffer.is_empty());
  }

  #[test]
  fn esc_clears_and_transitions_to_normal() {
    let mut handler = InputHandler {
      buffer: "partial".to_string(),
    };
    let result = handler.handle_key(BareKey::Esc, &Mode::NewSession);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(matches!(result.transition, Some(ModeTransition::Normal)));
    assert!(handler.buffer.is_empty());
  }

  #[test]
  fn enter_new_session_transitions_to_layout_select() {
    let mut handler = InputHandler {
      buffer: "my-session".to_string(),
    };
    let result = handler.handle_key(BareKey::Enter, &Mode::NewSession);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(matches!(
      result.transition,
      Some(ModeTransition::LayoutSelect(ref name)) if name == "my-session"
    ));
    assert!(handler.buffer.is_empty());
  }

  #[test]
  fn enter_rename_renames_and_transitions() {
    let mut handler = InputHandler {
      buffer: "new-name".to_string(),
    };
    let result = handler.handle_key(BareKey::Enter, &Mode::RenameSession);

    assert!(result.render);
    assert_eq!(
      result.actions,
      vec![PluginAction::RenameSession("new-name".to_string())]
    );
    assert!(matches!(result.transition, Some(ModeTransition::Normal)));
    assert!(handler.buffer.is_empty());
  }

  #[test]
  fn enter_empty_buffer_does_nothing() {
    let mut handler = InputHandler::default();
    let result = handler.handle_key(BareKey::Enter, &Mode::NewSession);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(result.transition.is_none());
  }

  #[test]
  fn control_char_rejected() {
    let mut handler = InputHandler::default();
    handler.handle_key(BareKey::Char('\x01'), &Mode::NewSession);

    assert!(handler.buffer.is_empty());
  }

  #[test]
  fn buffer_rejects_input_at_limit() {
    let mut handler = InputHandler {
      buffer: "a".repeat(128),
    };
    handler.handle_key(BareKey::Char('b'), &Mode::NewSession);

    assert_eq!(handler.buffer.len(), 128);
  }

  #[test]
  fn unhandled_key() {
    let mut handler = InputHandler::default();
    let result = handler.handle_key(BareKey::Tab, &Mode::NewSession);

    assert!(!result.render);
    assert!(result.actions.is_empty());
  }
}
