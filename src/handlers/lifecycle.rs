use crate::types::PluginAction;

/// Returns the actions needed during plugin load: requesting permissions
/// and subscribing to events.
pub fn handle_load() -> Vec<PluginAction> {
  vec![PluginAction::RequestPermissions, PluginAction::Subscribe]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load_returns_permissions_and_subscribe() {
    let actions = handle_load();

    assert_eq!(
      actions,
      vec![PluginAction::RequestPermissions, PluginAction::Subscribe]
    );
  }
}
