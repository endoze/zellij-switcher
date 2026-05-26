use zellij_switcher::config::PluginConfig;
use zellij_switcher::handlers::input::InputHandler;
use zellij_switcher::handlers::layout_select::LayoutSelectHandler;
use zellij_switcher::handlers::normal::NormalHandler;
use zellij_switcher::render;
use zellij_switcher::session_store::SessionStore;
use zellij_switcher::types::{Mode, ModeTransition, PluginAction};
use zellij_tile::prelude::*;

use std::collections::BTreeMap;

/// Top-level plugin state implementing the Zellij plugin trait.
#[derive(Default)]
struct Plugin {
  /// The current UI mode.
  mode: Mode,
  /// Session and layout data from Zellij.
  store: SessionStore,
  /// State for Normal mode key handling.
  normal: NormalHandler,
  /// State for text input modes (NewSession, RenameSession).
  input: InputHandler,
  /// State for the layout picker.
  layout_select: LayoutSelectHandler,
  /// User-configurable display settings.
  config: PluginConfig,
  /// The plugin's own pane ID, used to move the pane between tabs.
  plugin_pane_id: Option<PaneId>,
  /// The 0-indexed position of the currently active tab.
  active_tab_position: Option<usize>,
  /// Whether the plugin pane is currently visible. Gates polling.
  is_visible: bool,
  /// Whether a refresh timer is currently armed. Prevents double-arming.
  refresh_timer_armed: bool,
  /// Whether the host has granted the permissions we requested.
  /// `get_session_list()` cannot be called before this is `true` — the
  /// host won't reply, and zellij-tile's shim panics on the missing
  /// stdin response.
  permissions_granted: bool,
}

register_plugin!(Plugin);

/// Dispatches a list of declarative [`PluginAction`]s to the Zellij API.
fn execute_actions(actions: Vec<PluginAction>) {
  for action in actions {
    match action {
      PluginAction::HideSelf => hide_self(),
      PluginAction::SwitchSession(name) => switch_session(Some(&name)),
      PluginAction::SwitchSessionWithLayout(name, layout) => {
        switch_session_with_layout(Some(&name), layout, None)
      }
      PluginAction::KillSessions(names) => {
        let _ = kill_sessions(&names);
      }
      PluginAction::DeleteDeadSession(name) => {
        let _ = delete_dead_session(&name);
      }
      PluginAction::RenameSession(name) => rename_session(&name),
      PluginAction::RequestPermissions => {
        request_permission(&[
          PermissionType::ChangeApplicationState,
          PermissionType::ReadApplicationState,
        ]);
      }
      PluginAction::Subscribe => {
        subscribe(&[
          EventType::Key,
          EventType::TabUpdate,
          EventType::PaneUpdate,
          EventType::Visible,
          EventType::Timer,
          EventType::PermissionRequestResult,
        ]);
      }
      PluginAction::MoveToTab(pane_id, tab_position) => {
        break_panes_to_tab_with_index(&[pane_id], tab_position, false);
      }
      PluginAction::ShowSelf => show_self(true),
      PluginAction::SetTimeout(secs) => set_timeout(secs),
    }
  }
}

/// Applies a [`ModeTransition`] by updating the current mode and resetting
/// the relevant handler state.
fn apply_transition(
  transition: ModeTransition,
  mode: &mut Mode,
  input: &mut InputHandler,
  layout_select: &mut LayoutSelectHandler,
) {
  match transition {
    ModeTransition::Normal => {
      *mode = Mode::Normal;
    }
    ModeTransition::NewSession => {
      input.clear();
      *mode = Mode::NewSession;
    }
    ModeTransition::RenameSession => {
      input.clear();
      *mode = Mode::RenameSession;
    }
    ModeTransition::LayoutSelect(name) => {
      layout_select.start(name);
      *mode = Mode::LayoutSelect;
    }
  }
}

impl Plugin {
  /// Pulls a fresh session snapshot from Zellij, preserves the user's
  /// selection by name, and applies the snapshot to `self.store`.
  /// Returns whether the store changed.
  ///
  /// Returns `false` if the host call fails — the next timer tick will
  /// retry. This is the only path that mutates session data; the plugin
  /// no longer trusts `Event::SessionUpdate` to avoid stale-broadcast
  /// poisoning.
  fn refresh_session_list(&mut self) -> bool {
    if !self.permissions_granted {
      return false;
    }

    let snapshot = match get_session_list() {
      Ok(snapshot) => snapshot,
      Err(_) => return false,
    };
    let previous_name = self
      .store
      .selected_session(self.normal.selected_index)
      .map(|s| s.name().to_owned());
    let changed = self
      .store
      .update(snapshot.live_sessions, snapshot.resurrectable_sessions);

    if changed {
      match previous_name {
        Some(name) => self.normal.preserve_selection(&name, &self.store),
        None => self.normal.clamp_index(self.store.total_count()),
      }
    }

    changed
  }

  /// Arms a 1-second refresh timer if one isn't already pending.
  /// Idempotent — safe to call repeatedly. Returns the action to
  /// execute, or `None` if a timer is already armed.
  fn arm_refresh_timer(&mut self) -> Option<PluginAction> {
    if self.refresh_timer_armed {
      return None;
    }

    self.refresh_timer_armed = true;

    Some(PluginAction::SetTimeout(1.0))
  }
}

/// Zellij plugin trait implementation wiring events to handlers and rendering.
impl ZellijPlugin for Plugin {
  /// Initializes plugin configuration and requests permissions/subscriptions.
  /// The initial session-list pull is deferred until the host fires
  /// `Event::PermissionRequestResult(Granted)` — `get_session_list()`
  /// panics in zellij-tile's shim if called before permissions are
  /// granted (the host doesn't reply and the shim unwraps `None`).
  fn load(&mut self, configuration: BTreeMap<String, String>) {
    self.config = PluginConfig::from(configuration);

    let ids = get_plugin_ids();
    self.plugin_pane_id = Some(PaneId::Plugin(ids.plugin_id));

    execute_actions(zellij_switcher::handlers::lifecycle::handle_load());
  }

  /// Handles incoming Zellij events. Session state comes exclusively
  /// from pull-based `refresh_session_list` calls driven by load,
  /// visibility transitions, and timer ticks.
  fn update(&mut self, event: Event) -> bool {
    match event {
      Event::Key(key) if key.has_no_modifiers() => {
        let result = match self.mode {
          Mode::Normal => self.normal.handle_key(key.bare_key, &self.store),
          Mode::NewSession | Mode::RenameSession => self.input.handle_key(key.bare_key, &self.mode),
          Mode::LayoutSelect => self
            .layout_select
            .handle_key(key.bare_key, &self.store.available_layouts),
        };

        execute_actions(result.actions);

        if let Some(transition) = result.transition {
          apply_transition(
            transition,
            &mut self.mode,
            &mut self.input,
            &mut self.layout_select,
          );
        }

        result.render
      }
      Event::TabUpdate(tab_infos) => {
        self.active_tab_position =
          zellij_switcher::handlers::visibility::active_tab_position(&tab_infos);

        false
      }
      Event::Visible(visible) => {
        let result = zellij_switcher::handlers::visibility::handle_visible(
          visible,
          self.plugin_pane_id,
          self.active_tab_position,
        );

        execute_actions(result.actions);

        self.is_visible = visible;

        if visible {
          let changed = self.refresh_session_list();

          if let Some(action) = self.arm_refresh_timer() {
            execute_actions(vec![action]);
          }

          result.render || changed
        } else {
          result.render
        }
      }
      Event::PermissionRequestResult(status) => {
        let granted = matches!(status, PermissionStatus::Granted);

        if !granted || self.permissions_granted {
          return false;
        }

        self.permissions_granted = true;

        // Seed the store with current data so the first PaneUpdate
        // delivering focus arrives with non-empty state. Don't arm the
        // timer here — polling is gated on focus, which we learn from
        // PaneUpdate.
        self.refresh_session_list()
      }
      Event::PaneUpdate(manifest) => {
        let focused = match self.plugin_pane_id {
          Some(PaneId::Plugin(id)) => manifest
            .panes
            .values()
            .flatten()
            .any(|p| p.is_plugin && p.id == id && p.is_focused),
          _ => false,
        };

        if focused == self.is_visible {
          return false;
        }

        self.is_visible = focused;

        if focused {
          let changed = self.refresh_session_list();

          if let Some(action) = self.arm_refresh_timer() {
            execute_actions(vec![action]);
          }

          changed
        } else {
          // The in-flight timer (if any) will fire once, see
          // is_visible == false, and not re-arm. Polling stops.
          false
        }
      }
      Event::Timer(_) => {
        self.refresh_timer_armed = false;

        if !self.is_visible {
          return false;
        }

        let changed = self.refresh_session_list();

        if let Some(action) = self.arm_refresh_timer() {
          execute_actions(vec![action]);
        }

        changed
      }
      _ => false,
    }
  }

  /// Handles pipe messages (currently unused).
  fn pipe(&mut self, _pipe_message: PipeMessage) -> bool {
    false
  }

  /// Renders the plugin UI into the available terminal area.
  fn render(&mut self, rows: usize, cols: usize) {
    render::render(
      &self.mode,
      &self.store,
      &self.normal,
      &self.input,
      &self.layout_select,
      &self.config,
      rows,
      cols,
    );
  }
}
