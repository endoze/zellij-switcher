use crate::types::PluginAction;

/// Returns the actions needed during plugin load: subscribing to events,
/// then requesting permissions. Order matters — if permissions are
/// already cached, the host fires `PermissionRequestResult(Granted)`
/// immediately, so the subscription must be in place first or the
/// event is dropped and the plugin never learns it can pull session
/// data.
pub fn handle_load() -> Vec<PluginAction> {
  vec![PluginAction::Subscribe, PluginAction::RequestPermissions]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load_returns_permissions_and_subscribe() {
    let actions = handle_load();

    assert_eq!(
      actions,
      vec![PluginAction::Subscribe, PluginAction::RequestPermissions]
    );
  }
}
