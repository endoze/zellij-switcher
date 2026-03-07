use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use zellij_tile::prelude::*;

use crate::session_store::SessionStore;

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
  }
}

pub fn make_session_store(active: &[(&str, bool)], dead: &[&str]) -> SessionStore {
  let sessions: Vec<SessionInfo> = active
    .iter()
    .map(|(name, current)| make_session(name, *current))
    .collect();
  let resurrectable: Vec<(String, Duration)> = dead
    .iter()
    .map(|name| (name.to_string(), Duration::from_secs(0)))
    .collect();

  SessionStore {
    sessions,
    resurrectable_sessions: resurrectable,
    ..Default::default()
  }
}
