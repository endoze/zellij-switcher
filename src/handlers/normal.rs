use zellij_tile::prelude::*;

use crate::session_store::SessionStore;
use crate::types::{HandlerResult, ModeTransition, PluginAction, SelectedSession};

/// Handles key events in Normal mode (session list browsing).
#[derive(Default)]
pub struct NormalHandler {
  /// Index of the currently highlighted session in the unified list.
  pub selected_index: usize,
  /// A transient hint message shown to the user (cleared on next key press).
  pub hint_message: Option<&'static str>,
}

impl NormalHandler {
  /// Ensures `selected_index` is within bounds for the given total session count.
  pub fn clamp_index(&mut self, total: usize) {
    if total == 0 {
      self.selected_index = 0;
    } else if self.selected_index >= total {
      self.selected_index = total - 1;
    }
  }

  /// Updates `selected_index` to track a session by name after a reorder.
  /// Searches the unified entries list. Falls back to `clamp_index()` if
  /// the session is no longer present.
  pub fn preserve_selection(&mut self, name: &str, store: &SessionStore) {
    if let Some(pos) = store.entries.iter().position(|e| e.name() == name) {
      self.selected_index = pos;
      return;
    }

    self.clamp_index(store.total_count());
  }

  /// Processes a key press in Normal mode, returning navigation changes,
  /// session operations, or mode transitions.
  pub fn handle_key(&mut self, key: BareKey, store: &SessionStore) -> HandlerResult {
    let had_hint = self.hint_message.take().is_some();

    match key {
      BareKey::Esc => HandlerResult::no_render().with_action(PluginAction::HideSelf),
      BareKey::Char('j') | BareKey::Down => {
        let total = store.total_count();

        if total > 0 && self.selected_index < total - 1 {
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
        if let Some(selected) = store.selected_session(self.selected_index) {
          HandlerResult::no_render()
            .with_action(PluginAction::SwitchSession(selected.name().to_owned()))
            .with_action(PluginAction::HideSelf)
        } else {
          HandlerResult::no_render()
        }
      }
      BareKey::Char(c @ '1'..='9') => {
        let index = (c as usize) - ('1' as usize);

        if index < store.total_count() {
          self.selected_index = index;

          if let Some(selected) = store.selected_session(self.selected_index) {
            return HandlerResult::no_render()
              .with_action(PluginAction::SwitchSession(selected.name().to_owned()))
              .with_action(PluginAction::HideSelf);
          }
        }

        HandlerResult::no_render()
      }
      BareKey::Char('n') => HandlerResult::render().with_transition(ModeTransition::NewSession),
      BareKey::Char('r') => {
        if let Some(selected) = store.selected_session(self.selected_index) {
          match selected {
            SelectedSession::Active(s) if s.is_current_session => {
              HandlerResult::render().with_transition(ModeTransition::RenameSession)
            }
            _ => {
              self.hint_message = Some("Can only rename current session");
              HandlerResult::render()
            }
          }
        } else {
          HandlerResult::render_if(had_hint)
        }
      }
      BareKey::Char('d') => {
        if let Some(selected) = store.selected_session(self.selected_index) {
          match selected {
            SelectedSession::Active(s) => {
              let name = s.name.clone();
              HandlerResult::render().with_action(PluginAction::KillSessions(vec![name]))
            }
            SelectedSession::Dead(name) => {
              let name = name.to_owned();
              HandlerResult::render().with_action(PluginAction::DeleteDeadSession(name))
            }
          }
        } else {
          HandlerResult::render_if(had_hint)
        }
      }
      _ => HandlerResult::render_if(had_hint),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::testutil::make_session_store;

  #[test]
  fn j_moves_down() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('j'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn j_at_bottom_stays() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler {
      selected_index: 1,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('j'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn k_moves_up() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler {
      selected_index: 1,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('k'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn k_at_top_stays() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('k'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn down_arrow_moves_down() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Down, &store);

    assert!(result.render);
    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn up_arrow_moves_up() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler {
      selected_index: 1,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Up, &store);

    assert!(result.render);
    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn esc_hides() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Esc, &store);

    assert!(!result.render);
    assert_eq!(result.actions, vec![PluginAction::HideSelf]);
  }

  #[test]
  fn enter_switches_active_session() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler {
      selected_index: 1,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Enter, &store);

    assert!(!result.render);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::SwitchSession("s2".to_string()),
        PluginAction::HideSelf,
      ]
    );
  }

  #[test]
  fn enter_switches_dead_session() {
    let store = make_session_store(&[("s1", true)], &["dead1"]);
    let mut handler = NormalHandler {
      selected_index: 0,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Enter, &store);

    assert!(!result.render);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::SwitchSession("dead1".to_string()),
        PluginAction::HideSelf,
      ]
    );
  }

  #[test]
  fn enter_no_sessions_does_nothing() {
    let store = make_session_store(&[], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Enter, &store);

    assert!(!result.render);
    assert!(result.actions.is_empty());
  }

  #[test]
  fn quick_switch_valid_index() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('2'), &store);

    assert!(!result.render);
    assert_eq!(handler.selected_index, 1);
    assert_eq!(
      result.actions,
      vec![
        PluginAction::SwitchSession("s2".to_string()),
        PluginAction::HideSelf,
      ]
    );
  }

  #[test]
  fn quick_switch_out_of_range() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('5'), &store);

    assert!(!result.render);
    assert!(result.actions.is_empty());
  }

  #[test]
  fn n_transitions_to_new_session() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('n'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(matches!(
      result.transition,
      Some(ModeTransition::NewSession)
    ));
  }

  #[test]
  fn r_transitions_to_rename_for_current() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('r'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(matches!(
      result.transition,
      Some(ModeTransition::RenameSession)
    ));
  }

  #[test]
  fn r_shows_hint_for_non_current() {
    let store = make_session_store(&[("s1", false)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('r'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());
    assert!(result.transition.is_none());
    assert!(handler.hint_message.is_some());
  }

  #[test]
  fn d_kills_active_session() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler::default();
    let result = handler.handle_key(BareKey::Char('d'), &store);

    assert!(result.render);
    assert_eq!(
      result.actions,
      vec![PluginAction::KillSessions(vec!["s1".to_string()])]
    );
  }

  #[test]
  fn d_deletes_dead_session() {
    let store = make_session_store(&[("s1", true)], &["dead1"]);
    let mut handler = NormalHandler {
      selected_index: 0,
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('d'), &store);

    assert!(result.render);
    assert_eq!(
      result.actions,
      vec![PluginAction::DeleteDeadSession("dead1".to_string())]
    );
  }

  #[test]
  fn key_clears_hint() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler {
      hint_message: Some("old hint"),
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('j'), &store);

    assert!(result.render);
    assert!(handler.hint_message.is_none());
  }

  #[test]
  fn unhandled_key_returns_had_hint() {
    let store = make_session_store(&[("s1", true)], &[]);
    let mut handler = NormalHandler {
      hint_message: Some("hint"),
      ..Default::default()
    };
    let result = handler.handle_key(BareKey::Char('x'), &store);

    assert!(result.render);
    assert!(result.actions.is_empty());

    let result = handler.handle_key(BareKey::Char('x'), &store);

    assert!(!result.render);
  }

  #[test]
  fn preserve_selection_finds_active_session() {
    let store = make_session_store(
      &[("alpha", true), ("bravo", false), ("charlie", false)],
      &[],
    );
    let mut handler = NormalHandler {
      selected_index: 0,
      ..Default::default()
    };
    handler.preserve_selection("charlie", &store);

    assert_eq!(handler.selected_index, 2);
  }

  #[test]
  fn preserve_selection_finds_dead_session() {
    let store = make_session_store(&[("s1", true)], &["dead1", "dead2"]);
    let mut handler = NormalHandler {
      selected_index: 0,
      ..Default::default()
    };
    handler.preserve_selection("dead2", &store);

    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn preserve_selection_clamps_when_missing() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);
    let mut handler = NormalHandler {
      selected_index: 5,
      ..Default::default()
    };
    handler.preserve_selection("gone", &store);

    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn clamp_index_within_bounds() {
    let mut handler = NormalHandler {
      selected_index: 1,
      ..Default::default()
    };
    handler.clamp_index(2);

    assert_eq!(handler.selected_index, 1);
  }

  #[test]
  fn clamp_index_past_end() {
    let mut handler = NormalHandler {
      selected_index: 5,
      ..Default::default()
    };
    handler.clamp_index(1);

    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn preserve_selection_tracks_session_that_moved_to_dead() {
    let store = make_session_store(&[("s1", true)], &["formerly_active"]);
    let mut handler = NormalHandler {
      selected_index: 0,
      ..Default::default()
    };
    handler.preserve_selection("formerly_active", &store);

    assert_eq!(handler.selected_index, 0);
  }

  #[test]
  fn clamp_index_empty() {
    let mut handler = NormalHandler {
      selected_index: 3,
      ..Default::default()
    };
    handler.clamp_index(0);

    assert_eq!(handler.selected_index, 0);
  }
}
