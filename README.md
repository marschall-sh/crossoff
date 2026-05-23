# crossoff

A fast, keyboard-driven terminal task manager built in Rust.

![Demo](demo/demo.gif)

## Features

- Board-first Kanban UI with ToDo, In Progress, and Done lanes
- Task cards with due dates, label pills, checklist progress, and timer chips
- Tasks with labels, due dates, descriptions, and checklists
- Prioritize tasks to the top of a lane
- Start/stop time tracking per task with one active timer globally
- Centered detail and edit views, plus fuzzy search
- Eight built-in themes
- XDG-aware storage with atomic saves and backup fallback

## Install

**Pre-built binaries**

Download the matching binary from the [Releases](../../releases) page, then:

```sh
chmod +x crossoff-*
mv crossoff-* /usr/local/bin/crossoff
```

**From source**

```sh
cargo install --git https://github.com/marschall-sh/crossoff
```

## Uninstall

```sh
cargo uninstall crossoff
```

If installed manually:

```sh
rm /usr/local/bin/crossoff
rm -rf ~/.config/crossoff ~/.local/share/crossoff
```

## Keybinds

| Key | Action |
|---|---|
| `↑` / `↓` or `j` / `k` | Navigate tasks |
| `←` / `→` or `h` / `l` or `Tab` / `Shift+Tab` | Switch kanban lane |
| `H` / `L` | Move task between lanes |
| `q` / `Esc` | Close task details |
| `Enter` | Open task details |
| `Space` | Mark task done / undo done |
| `n` / `e` / `d` | New / edit / delete task |
| `p` | Toggle task priority |
| `t` | Start / stop timer on task |
| `/` | Fuzzy search |
| `Ctrl+S` | Save in editors |
| `?` | Help |
| `q` | Quit app / close details |

## Configuration

Config file:

`~/.config/crossoff/config.toml`

```toml
theme = "tokyo-night"

# Optional custom storage path.
# Can be either a directory or a full .json file path.
# data_dir = "/absolute/path/to/crossoff-data"
# data_dir = "/absolute/path/to/crossoff-data/data.json"
```

Available themes:

`tokyo-night` · `catppuccin-mocha` · `catppuccin-latte` · `dracula` · `gruvbox-dark` · `nord` · `solarized-light` · `rose-pine-dawn`

## Storage

Default data path:

- `~/.local/share/crossoff/data.json`
- or `XDG_DATA_HOME/crossoff/data.json`

Storage is designed to be safer for synced folders:

- atomic saves via `data.json.tmp` + rename
- automatic backup file `data.json.bak`
- optional custom data path via `data_dir`

## License

MIT
