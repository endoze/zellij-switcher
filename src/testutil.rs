use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use zellij_tile::prelude::*;

use crate::session_store::{SessionEntry, SessionStore};

pub fn make_session(name: &str, is_current: bool) -> SessionInfo {
  SessionInfo {
    name: name.to_string(),
    tabs: Vec::new(),
    panes: PaneManifest {
      panes: HashMap::new(),
    },
    connected_clients: 0,
    is_current_session: is_current,
    available_layouts: Vec::new(),
    plugins: BTreeMap::new(),
    web_clients_allowed: false,
    web_client_count: 0,
    tab_history: BTreeMap::new(),
    pane_history: BTreeMap::new(),
    creation_time: Duration::from_secs(0),
  }
}

pub fn make_session_store(active: &[(&str, bool)], dead: &[&str]) -> SessionStore {
  let mut entries: Vec<SessionEntry> = Vec::new();

  for (name, current) in active {
    entries.push(SessionEntry::Active(make_session(name, *current)));
  }

  for name in dead {
    entries.push(SessionEntry::Dead {
      name: name.to_string(),
      duration: Duration::from_secs(0),
    });
  }

  entries.sort_by(|a, b| a.name().cmp(b.name()));

  SessionStore {
    entries,
    ..Default::default()
  }
}
