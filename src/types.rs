use zellij_tile::prelude::*;

/// The current UI mode of the plugin.
#[derive(Debug, Default, PartialEq)]
pub enum Mode {
  /// Browsing the session list.
  #[default]
  Normal,
  /// Typing a name for a new session.
  NewSession,
  /// Typing a new name for the current session.
  RenameSession,
  /// Picking a layout for a newly created session.
  LayoutSelect,
}

/// A declarative mode change returned by handlers.
pub enum ModeTransition {
  /// Return to the normal session list.
  Normal,
  /// Enter the new-session input prompt.
  NewSession,
  /// Enter the rename-session input prompt.
  RenameSession,
  /// Enter layout selection with the given session name.
  LayoutSelect(String),
}

/// A reference to either an active or resurrectable (dead) session.
pub enum SelectedSession<'a> {
  /// A currently running session.
  Active(&'a SessionInfo),
  /// A resurrectable session identified by name.
  Dead(&'a str),
}

impl<'a> SelectedSession<'a> {
  /// Returns the session name regardless of active/dead status.
  pub fn name(&self) -> &str {
    match self {
      SelectedSession::Active(s) => &s.name,
      SelectedSession::Dead(name) => name,
    }
  }
}

/// A side-effecting action to be executed against the Zellij API.
#[derive(Debug, Clone, PartialEq)]
pub enum PluginAction {
  /// Hide the plugin pane.
  HideSelf,
  /// Switch to the session with the given name.
  SwitchSession(String),
  /// Switch to a session with the given name and layout.
  SwitchSessionWithLayout(String, LayoutInfo),
  /// Kill the sessions with the given names.
  KillSessions(Vec<String>),
  /// Permanently delete a dead (resurrectable) session.
  DeleteDeadSession(String),
  /// Rename the current session.
  RenameSession(String),
  /// Request the permissions needed by this plugin.
  RequestPermissions,
  /// Subscribe to the Zellij events this plugin listens for.
  Subscribe,
  /// Move a pane to the specified tab position.
  MoveToTab(PaneId, usize),
  /// Show the plugin pane, focus it, and switch to its tab.
  ShowSelf,
  /// Arm a one-shot timer that will fire `Event::Timer` after `secs` seconds.
  SetTimeout(f64),
}

/// The result returned by every key handler, carrying render intent,
/// actions to execute, and an optional mode transition.
pub struct HandlerResult {
  /// Whether the UI should re-render after this event.
  pub render: bool,
  /// Actions to execute against the Zellij API.
  pub actions: Vec<PluginAction>,
  /// An optional mode transition to apply after executing actions.
  pub transition: Option<ModeTransition>,
}

impl HandlerResult {
  /// Creates a result that triggers a re-render with no actions.
  pub fn render() -> Self {
    Self {
      render: true,
      actions: Vec::new(),
      transition: None,
    }
  }

  /// Creates a result that skips re-rendering with no actions.
  pub fn no_render() -> Self {
    Self {
      render: false,
      actions: Vec::new(),
      transition: None,
    }
  }

  /// Creates a result that conditionally re-renders based on `condition`.
  pub fn render_if(condition: bool) -> Self {
    Self {
      render: condition,
      actions: Vec::new(),
      transition: None,
    }
  }

  /// Appends a [`PluginAction`] to this result.
  pub fn with_action(mut self, action: PluginAction) -> Self {
    self.actions.push(action);

    self
  }

  /// Sets the mode transition for this result.
  pub fn with_transition(mut self, transition: ModeTransition) -> Self {
    self.transition = Some(transition);

    self
  }
}
