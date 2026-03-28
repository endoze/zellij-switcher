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
      PluginAction::KillSessions(names) => kill_sessions(&names),
      PluginAction::DeleteDeadSession(name) => delete_dead_session(&name),
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
          EventType::SessionUpdate,
          EventType::TabUpdate,
          EventType::Visible,
        ]);
      }
      PluginAction::MoveToTab(pane_id, tab_position) => {
        break_panes_to_tab_with_index(&[pane_id], tab_position, false);
      }
      PluginAction::ShowSelf => show_self(true),
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

/// Zellij plugin trait implementation wiring events to handlers and rendering.
impl ZellijPlugin for Plugin {
  /// Initializes plugin configuration and requests permissions/subscriptions.
  fn load(&mut self, configuration: BTreeMap<String, String>) {
    self.config = PluginConfig::from(configuration);

    let ids = get_plugin_ids();
    self.plugin_pane_id = Some(PaneId::Plugin(ids.plugin_id));

    execute_actions(zellij_switcher::handlers::lifecycle::handle_load());
  }

  /// Handles incoming Zellij events (session updates and key presses).
  fn update(&mut self, event: Event) -> bool {
    match event {
      Event::SessionUpdate(sessions, resurrectable_sessions) => {
        let previous_name = self
          .store
          .selected_session(self.normal.selected_index)
          .map(|s| s.name().to_owned());
        let changed = self.store.update(sessions, resurrectable_sessions);

        if changed {
          match previous_name {
            Some(name) => self.normal.preserve_selection(&name, &self.store),
            None => self.normal.clamp_index(self.store.total_count()),
          }
        }

        changed
      }
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

        result.render
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
