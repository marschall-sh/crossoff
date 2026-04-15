# crossoff

A fast, keyboard-driven terminal task manager — built with Rust and [ratatui](https://github.com/ratatui/ratatui).

Manage projects and tasks entirely from the terminal. Clean three-pane layout with a task list, scrollable detail view, fuzzy search, labels, checklists, due dates, and multiple themes.

![Demo](demo/demo.gif)

---

## Features

- **Projects** — Inbox always pinned to top, others sorted alphabetically
- **Tasks** — title, description (multi-line, paste-friendly), due date, labels, checklist
- **Detail pane** — scrollable task preview with bullet rendering and checklist progress
- **Fuzzy search** — search across all tasks with highlighted matches
- **Labels** — color-coded tags with automatic contrast
- **Pin to top** — manually surface any task above the auto-sorted list
- **Themes** — eight built-in themes
- **Auto-cleanup** — completed tasks disappear after one hour

---

## Install

### Pre-built binaries

Download the latest binary for your platform from the [Releases](../../releases) page.

```sh
# macOS / Linux
chmod +x crossoff-*
mv crossoff-* /usr/local/bin/crossoff
```

### From source

```sh
cargo install --git https://github.com/marschall-sh/crossoff
```

Or clone and build:

```sh
git clone https://github.com/marschall-sh/crossoff
cd crossoff
cargo build --release
# binary at target/release/crossoff
```

---

## Keybinds

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Navigate items |
| `Tab` / `Shift+Tab` | Cycle panes (Projects → Tasks → Detail) |
| `Esc` | Go back / cancel |

### Tasks

| Key | Action |
|-----|--------|
| `Enter` / `Space` | Toggle done |
| `n` | New task |
| `e` | Edit task |
| `d` | Delete task |
| `p` | Pin / unpin to top |

### Projects

| Key | Action |
|-----|--------|
| `n` | New project |
| `e` | Rename project |
| `d` | Delete project |

### Global

| Key | Action |
|-----|--------|
| `/` | Fuzzy search all tasks |
| `?` | Help |
| `q` | Quit |

### Editors

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Tab` / `Shift+Tab` | Next / previous field |
| `Enter` | Confirm / open sub-editor |
| `Esc` | Cancel |

---

## Configuration

Place `config.toml` at `~/.config/crossoff/config.toml` (or `$XDG_CONFIG_HOME/crossoff/config.toml`):

```toml
theme = "tokyo-night"
```

### Available themes

| Name | Style |
|------|-------|
| `tokyo-night` | Dark, blue-purple (default) |
| `catppuccin-mocha` | Dark, warm pastels |
| `catppuccin-latte` | Light, warm pastels |
| `dracula` | Dark, high contrast |
| `gruvbox-dark` | Dark, earthy retro |
| `nord` | Dark, arctic blues |
| `solarized-light` | Light, classic |
| `rose-pine-dawn` | Light, warm rose |

---

## Data

Tasks and projects are stored as JSON at:

```
~/.local/share/crossoff/data.json
```

(or `$XDG_DATA_HOME/crossoff/data.json`)

---

## License

MIT
