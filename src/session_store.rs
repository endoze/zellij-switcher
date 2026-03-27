use std::time::Duration;
use zellij_tile::prelude::*;

use crate::types::SelectedSession;

/// Represents either an active or dead session in the unified list.
pub enum SessionEntry {
  /// A currently running session.
  Active(SessionInfo),
  /// A dead session that can be resurrected.
  Dead { name: String, duration: Duration },
}

impl SessionEntry {
  /// Returns the session name regardless of variant.
  pub fn name(&self) -> &str {
    match self {
      SessionEntry::Active(s) => &s.name,
      SessionEntry::Dead { name, .. } => name,
    }
  }
}

/// Holds the current unified set of sessions and available layouts
/// reported by Zellij.
#[derive(Default)]
pub struct SessionStore {
  /// All sessions (active and dead), sorted alphabetically by name.
  pub entries: Vec<SessionEntry>,
  /// Layouts available from the current session for creating new sessions.
  pub available_layouts: Vec<LayoutInfo>,
}

impl SessionStore {
  /// Returns the total number of sessions.
  pub fn total_count(&self) -> usize {
    self.entries.len()
  }

  /// Returns the session at the given index as a [`SelectedSession`].
  pub fn selected_session(&self, index: usize) -> Option<SelectedSession<'_>> {
    self.entries.get(index).map(|entry| match entry {
      SessionEntry::Active(s) => SelectedSession::Active(s),
      SessionEntry::Dead { name, .. } => SelectedSession::Dead(name.as_str()),
    })
  }

  /// Merges active and dead sessions into a single alphabetically-sorted
  /// list, and copies available layouts from the current session.
  ///
  /// Returns `true` if the store changed, `false` if the update was a no-op.
  pub fn update(
    &mut self,
    mut sessions: Vec<SessionInfo>,
    resurrectable_sessions: Vec<(String, Duration)>,
  ) -> bool {
    if let Some(current) = sessions.iter_mut().find(|s| s.is_current_session) {
      self.available_layouts = std::mem::take(&mut current.available_layouts);
    }

    let mut entries: Vec<SessionEntry> =
      Vec::with_capacity(sessions.len() + resurrectable_sessions.len());

    for s in sessions {
      entries.push(SessionEntry::Active(s));
    }

    for (name, duration) in resurrectable_sessions {
      entries.push(SessionEntry::Dead { name, duration });
    }

    entries.sort_by(|a, b| a.name().cmp(b.name()));

    let unchanged = self.entries.len() == entries.len()
      && self
        .entries
        .iter()
        .zip(entries.iter())
        .all(|(old, new)| match (old, new) {
          (SessionEntry::Active(a), SessionEntry::Active(b)) => {
            a.name == b.name && a.is_current_session == b.is_current_session
          }
          (SessionEntry::Dead { name: a, .. }, SessionEntry::Dead { name: b, .. }) => a == b,
          _ => false,
        });

    if unchanged {
      return false;
    }

    self.entries = entries;

    true
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

    match store.selected_session(0) {
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
    let changed = store.update(sessions, dead);

    assert!(changed);
    assert_eq!(store.entries.len(), 3);
  }

  #[test]
  fn update_copies_layouts_from_current() {
    let mut store = SessionStore::default();
    let mut current = make_session("s1", true);
    current.available_layouts = vec![LayoutInfo::BuiltIn("default".to_string())];
    let sessions = vec![current, make_session("s2", false)];
    let changed = store.update(sessions, vec![]);

    assert!(changed);
    assert_eq!(store.available_layouts.len(), 1);
  }

  #[test]
  fn update_sorts_entries_alphabetically() {
    let mut store = SessionStore::default();
    let sessions = vec![
      make_session("charlie", false),
      make_session("alpha", true),
      make_session("bravo", false),
    ];
    store.update(sessions, vec![]);

    let names: Vec<&str> = store.entries.iter().map(|e| e.name()).collect();

    assert_eq!(names, vec!["alpha", "bravo", "charlie"]);
  }

  #[test]
  fn update_interleaves_active_and_dead_alphabetically() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("charlie", false), make_session("alpha", true)];
    let dead = vec![
      ("bravo".to_string(), Duration::from_secs(0)),
      ("delta".to_string(), Duration::from_secs(0)),
    ];
    store.update(sessions, dead);

    let names: Vec<&str> = store.entries.iter().map(|e| e.name()).collect();

    assert_eq!(names, vec!["alpha", "bravo", "charlie", "delta"]);
  }

  #[test]
  fn update_returns_false_when_unchanged() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("s1", true), make_session("s2", false)];
    let dead = vec![("dead1".to_string(), Duration::from_secs(0))];
    store.update(sessions.clone(), dead.clone());

    let sessions2 = vec![make_session("s1", true), make_session("s2", false)];
    let dead2 = vec![("dead1".to_string(), Duration::from_secs(0))];
    let changed = store.update(sessions2, dead2);

    assert!(!changed);
  }

  #[test]
  fn update_returns_true_when_session_added() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("s1", true)];
    store.update(sessions, vec![]);

    let sessions2 = vec![make_session("s1", true), make_session("s2", false)];
    let changed = store.update(sessions2, vec![]);

    assert!(changed);
  }

  #[test]
  fn update_returns_true_when_is_current_changes() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("s1", true), make_session("s2", false)];
    store.update(sessions, vec![]);

    let sessions2 = vec![make_session("s1", false), make_session("s2", true)];
    let changed = store.update(sessions2, vec![]);

    assert!(changed);
  }

  #[test]
  fn update_returns_true_when_session_changes_state() {
    let mut store = SessionStore::default();
    let sessions = vec![make_session("s1", true), make_session("s2", false)];
    store.update(sessions, vec![]);

    let sessions2 = vec![make_session("s1", true)];
    let dead2 = vec![("s2".to_string(), Duration::from_secs(0))];
    let changed = store.update(sessions2, dead2);

    assert!(changed);
  }
}
