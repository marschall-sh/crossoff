# crossoff

A fast, keyboard-driven terminal task manager built in Rust.

![Demo](demo/demo.gif)

## Features

- Projects with a pinned Inbox
- Tasks with labels, due dates, descriptions, and checklists
- Pin tasks to the top of a project
- Start/stop time tracking per task with one active timer globally
- Move tasks between projects
- Scrollable detail pane and fuzzy search
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
| `↑` / `↓` or `j` / `k` | Navigate |
| `Tab` / `Shift+Tab` | Cycle panes |
| `Enter` / `Space` | Toggle task done |
| `n` / `e` / `d` | New / edit / delete |
| `m` | Move task to another project |
| `p` | Pin task to top |
| `t` | Start / stop timer on task |
| `/` | Fuzzy search |
| `Ctrl+S` | Save in editors |
| `?` | Help |
| `q` | Quit |

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
