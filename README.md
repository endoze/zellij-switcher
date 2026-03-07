# Zellij Switcher

A session manager plugin for [Zellij](https://zellij.dev) that lets you switch, create, rename, delete, and resurrect sessions without leaving zellij.

## Features

- **Switch sessions** — navigate a list of active sessions and jump to any one instantly
- **Create sessions** — start new sessions with a name and layout of your choice
- **Rename sessions** — rename the current session inline
- **Delete sessions** — kill active sessions or permanently remove dead ones
- **Resurrect sessions** — dead (exited) sessions appear in the list and can be restored with Enter
- **Quick switch** — press `1`–`9` to jump directly to a session by its index

## Installation

### Download

Download the latest `zellij-switcher.wasm` from the [Releases](../../releases) page.

Place it somewhere accessible, for example:

```
~/.config/zellij/plugins/zellij-switcher.wasm
```

### Build from source

Requires Rust with the `wasm32-wasip1` target:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The compiled plugin will be at `target/wasm32-wasip1/release/zellij-switcher.wasm`. Copy it to your plugins directory:

```bash
cp target/wasm32-wasip1/release/zellij-switcher.wasm ~/.config/zellij/plugins/
```

## Setup

### Plugin alias and preloading

Define a plugin alias in your Zellij config (`~/.config/zellij/config.kdl`). This is where you set the plugin location and all of its configuration in one place:

```kdl
plugins {
  zellij-switcher location="file:~/.config/zellij/plugins/zellij-switcher.wasm" {
    // All configuration goes here — see the Configuration section below
    header_title "Session Manager"
    show_navigation_help "true"
    selection_prefix "> "
    list_padding "2"
  }
}
```

**Preloading is strongly recommended.** Without it, the first time you open the plugin there is a noticeable delay while it waits to receive the session list from Zellij. With `load_plugins`, the plugin is already running and has the session list ready, so it opens instantly every time:

```kdl
load_plugins {
  "zellij-switcher"
}
```

### Keybinding

With the alias defined, keybindings can reference the plugin by its alias name instead of repeating the full path and configuration:

```kdl
keybinds {
  shared {
    bind "Alt s" {
      LaunchOrFocusPlugin "zellij-switcher" {
        floating true
      }
    }
  }
}
```

### Permissions

On first launch, Zellij will prompt you to grant the plugin two permissions:

- **ReadApplicationState** — to list sessions and layouts
- **ChangeApplicationState** — to switch, create, rename, and delete sessions

## Usage

| Key | Action |
|---|---|
| `j` / `Down` | Move selection down |
| `k` / `Up` | Move selection up |
| `Enter` | Switch to selected session (or resurrect if dead) |
| `1`–`9` | Quick switch to session by index |
| `n` | Create a new session |
| `r` | Rename the current session |
| `d` | Delete/kill the selected session |
| `Esc` | Close the plugin |

### Creating a session

1. Press `n` to enter the new session prompt
2. Type a name and press `Enter`
3. Select a layout from the list and press `Enter`

### Renaming a session

1. Navigate to the current (active) session
2. Press `r`
3. Type the new name and press `Enter`

### Deleting a session

- On an **active** session: `d` kills the session
- On a **dead** session: `d` permanently removes it

## Configuration

Pass configuration options in the plugin alias block in your Zellij config. All options are optional — sensible defaults are used when omitted.

```kdl
plugins {
  zellij-switcher location="file:~/.config/zellij/plugins/zellij-switcher.wasm" {
    // Display
    header_title "Session Manager"
    show_index_numbers "true"
    show_navigation_help "true"
    show_selection_highlight "true"
    selection_prefix "> "
    list_padding "2"

    // Colors (values: 0–3, mapped to your Zellij theme palette)
    header_text_color "0"
    shortcut_key_color "0"
    index_number_color "0"
    active_marker_color "1"
    dead_marker_color "1"
    builtin_marker_color "1"
    hint_message_color "2"
    prompt_label_color "0"
  }
}
```

### Display options

| Option | Type | Default | Description |
|---|---|---|---|
| `header_title` | string | `"Session Manager"` | Title displayed in the centered header |
| `show_index_numbers` | bool | `true` | Show 1-based index numbers beside list items |
| `show_navigation_help` | bool | `true` | Show the `j/k` and `1-9` navigation hints in the footer |
| `show_selection_highlight` | bool | `true` | Highlight the selected row |
| `selection_prefix` | string | *(empty)* | Prefix string rendered beside the selected item (e.g. `"> "`) |
| `list_padding` | integer | `2` | Left padding in columns for list content |

### Color options

Color values are integers from `0` to `3`, corresponding to the four accent colors in your Zellij theme. Values above `3` are clamped to `3`.

| Option | Default | What it colors |
|---|---|---|
| `header_text_color` | `0` | Header title text |
| `shortcut_key_color` | `0` | Keyboard shortcut labels in the footer |
| `index_number_color` | `0` | List item index numbers |
| `active_marker_color` | `1` | The `(active)` marker on the current session |
| `dead_marker_color` | `1` | The `(dead)` marker on resurrectable sessions |
| `builtin_marker_color` | `1` | The `(built-in)` marker on built-in layouts |
| `hint_message_color` | `2` | Transient hint messages (e.g. "Can only rename current session") |
| `prompt_label_color` | `0` | Input prompt labels |

## License

See [LICENSE](LICENSE) for details.
