# Projctl

[![AUR](https://img.shields.io/aur/version/projctl?color=orange)](https://aur.archlinux.org/packages/projctl)

A lightweight Rust CLI to manage project context across your terminal sessions. Quickly switch between projects, open your editor/git UI, attach tmux servers, run commands, or view logs - all tied to a "current project" state.

## Features
- **Project Context**: Switch between projects with a single command; current project state is stored in `~/.cache/current_project`.
- **Auto-detected Projects**: Folders under `~/projects` are automatically visible; enable them with `projctl add`.
- **Interactive Picker**: Run `projctl use` with no args to pick from added projects.
- **Contextual Commands**:
  - `projctl run` - run a command in the project root.
  - `projctl edit` - open your editor in the project root.
  - `projctl git` - open your git UI in the project root.
  - `projctl logs` - tail or browse logs in `./logs`
  - `projctl servers` - spin up or a attach a tmux session with pre-seeded panes (frontend/backend/docker/logs/scratch)
- **Configurable**: Defaults(editor, git UI, projects dir) come from a config file (`~/.config/projctl/config.toml`) and/or CLI flags
- **Safe Path Handling**: Canonicalizes paths, prevents duplicate tracking, and highlights your current project in `list`.

---

## Installation

### Arch Linux (AUR)
Install via an AUR helper:
```bash
yay -S projctl 
# Or paru, aurman, etc.
```
### Manual (Cargo)
```bash
git clone https://github.com/nick-montgomery/projctl.git
cd projctl
cargo install --path .
```
This will place `projctl` in `~/.cargo/bin`.

---

## Usage

```bash
# Add projects
projctl add myapp ~/code/myapp    # add by name+path
projctl add                       # interactively add from auto-detected ~/projects

# Switch projects
projctl use myapp                 # switch to myapp
projctl use                       # interactive picker
projctl list                      # shows added projects (current highlighted)

# Run commands
projctl run npm start             # runs inside the project dir

# Open tools
projctl edit                      # opens editor (default: nvim)
projctl git                       # opens git UI (default: lazygit)

# Logs
projctl logs                      # tails logs/*.log (uses lnav if available)

# Servers (tmux integration)
projctl servers                    # create session or attach existing session
projctl servers --refresh         # re-seed commands if session existing
projctl servers --reset           # kill and recreate session
projctl servers --kill            # kill session
```
---

## Configuration

Projctl resolves config in this order: **CLI flags** -> **config file** -> **defaults**.

Default config file: `~/.config/projctl/config.toml`

Example:
```toml
editor = "nvim"
git_ui = "lazygit"
projects_dir = "~/projects"
```
Or override per-invocation:
```bash
projctl --editor "code -g" edit
projctl --gitui gitui git
```
---

## Logs Setup
The `logs` command expects a `./logs/` dir with `*.log` files. Example for a Node.js app:
```bash
mkdir -p logs
pnpm run dev 2>&1 | tee logs/app.log
projctl logs
```
---
## Upcoming

Planned improvements in future versions:

- **Editor / Git UI fallbacks**:
  Automatically detect common tools (`nvim`, `vim`, `code`, `zed`, `lazygit`, `gitui`, ...).

- **Logs viewer options**
  Configurable fallback order(`lnav`, `tail  -F`, etc.)

- **Tmux integration**
  More flexible layouts and commands like `projctl servers --list`.

See the [ROADMAP](./ROADMAP.md) for a more detailed outline and ideas.

## Contributing

1. Fork the repo.
2. Create a feature branch (`git checkout -b feature/add-manpage`).
3. Commit changes (`git commit -am 'Add man page'`).
4. Push (`git push origin feature/add-manpage`).
5. Open a Pull Request.

Bug reports and ideas welcome - file an issue!

## License

MIT License. See LICENSE for details.

---

*Built with ❤️ for terminal workflows. Version 2.0.0 (2025-09-30)*

