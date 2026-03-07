use std::time::Duration;
use zellij_tile::prelude::*;

use crate::types::SelectedSession;

/// Holds the current set of active sessions, resurrectable (dead) sessions,
/// and available layouts reported by Zellij.
#[derive(Default)]
pub struct SessionStore {
  /// Currently running sessions.
  pub sessions: Vec<SessionInfo>,
  /// Dead sessions that can be resurrected, each paired with their age.
  pub resurrectable_sessions: Vec<(String, Duration)>,
  /// Layouts available from the current session for creating new sessions.
  pub available_layouts: Vec<LayoutInfo>,
}

impl SessionStore {
  /// Returns the combined count of active and resurrectable sessions.
  pub fn total_count(&self) -> usize {
    self.sessions.len() + self.resurrectable_sessions.len()
  }

  /// Returns the session at the given unified index, where active sessions
  /// come first followed by resurrectable sessions.
  pub fn selected_session(&self, index: usize) -> Option<SelectedSession<'_>> {
    let active_count = self.sessions.len();

    if index < active_count {
      self.sessions.get(index).map(SelectedSession::Active)
    } else {
      let dead_index = index - active_count;

      self
        .resurrectable_sessions
        .get(dead_index)
        .map(|(name, _)| SelectedSession::Dead(name.as_str()))
    }
  }

  /// Replaces stored sessions and resurrectable sessions, and copies
  /// available layouts from the current session if present.
  pub fn update(
    &mut self,
    mut sessions: Vec<SessionInfo>,
    resurrectable_sessions: Vec<(String, Duration)>,
  ) {
    if let Some(current) = sessions.iter_mut().find(|s| s.is_current_session) {
      self.available_layouts = std::mem::take(&mut current.available_layouts);
    }

    self.sessions = sessions;
    self.resurrectable_sessions = resurrectable_sessions;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::testutil::{make_session, make_session_store};

  #[test]
  fn total_count_empty() {
    let store = SessionStore::default();

    assert_eq!(store.total_count(), 0);
  }

  #[test]
  fn total_count_active_only() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);

    assert_eq!(store.total_count(), 2);
  }

  #[test]
  fn total_count_mixed() {
    let store = make_session_store(&[("s1", true)], &["dead1", "dead2"]);

    assert_eq!(store.total_count(), 3);
  }

  #[test]
  fn selected_session_active() {
    let store = make_session_store(&[("s1", true), ("s2", false)], &[]);

    match store.selected_session(0) {
      Some(SelectedSession::Active(s)) => assert_eq!(s.name, "s1"),
      _ => panic!("expected active session"),
    }
  }

  #[test]
  fn selected_session_dead() {
    let store = make_session_store(&[("s1", true)], &["dead1"]);

    match store.selected_session(1) {
      Some(SelectedSession::Dead(name)) => assert_eq!(name, "dead1"),
      _ => panic!("expected dead session"),
    }
  }

  #[test]
  fn selected_session_out_of_bounds() {
    let store = make_session_store(&[("s1", true)], &[]);

    assert!(store.selected_session(5).is_none());
  }

  #[test]
  fn selected_session_empty() {
    let store = SessionStore::default();

    assert!(store.selected_session(0).is_none());
  }

  #[test]
  fn update_populates_state() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("s1", true), make_session("s2", false)];
    let dead = vec![("dead1".to_string(), Duration::from_secs(60))];
    store.update(sessions, dead);

    assert_eq!(store.sessions.len(), 2);
    assert_eq!(store.resurrectable_sessions.len(), 1);
  }

  #[test]
  fn update_copies_layouts_from_current() {
    let mut store = SessionStore::default();
    let mut current = make_session("s1", true);
    current.available_layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let sessions = vec![current, make_session("s2", false)];
    store.update(sessions, vec![]);

    assert_eq!(store.available_layouts.len(), 1);
  }
}
