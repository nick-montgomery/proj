# Proj

[![AUR](https://img.shields.io/aur/version/projctl?color=orange)](https://aur.archlinux.org/packages/projctl)

A lightweight bash script to manage project context across your terminal sessions. Quickly switch directories, run commands, edit files, browse git, or tail logs—all tied to a "current project" state file.

## Features

- **Set/Clear Project**: Switch between projects with a single command; state persists in `~/.cache/current_project`.
- **Contextual Commands**: `projctl run`, `projctl edit`, `projctl git` automatically `cd` into the project dir.
- **Logs Viewer**: Tail or browse logs in `./logs/` using `lnav` (fallback to `tail -F`).
- **Customizable**: Override editor (`EDITOR_CMD=vim`) or git UI (`GIT_UI_CMD=lazygit`).

No dependencies beyond bash; optional tools (nvim, lazygit, lnav) enhance UX.

## Installation

### Arch Linux (AUR)
Install via an AUR helper:
```bash
yay -S projctl # Or paru, aurman, etc.
```
Manual build:

```bash
git clone https://aur.archlinux.org/projctl.git
cd projctl
makepg -si
```
### Manual (Any Linux/macOS)

1. Download the script:

```bash
curl -L0 https://raw.githubusercontent.com/nick-montgomery/projctl/main/projctl
chmod +x projctl
sudo mv projctl /usr/local/bin # Or ~/bin/
```
2. (Optional) Set up state dir: `mkdir -p ~/.cache`

### From Source

```bash
git clone https://github.com/nick-montgomery/projctl.git
cd projctl
sudo make install # TODO: Add makefile
```
## Usage

```bash
projctl set /path/to/project # Set current project (absolute/relative)
projctl path                 # Print current project path
projctl clear                # Clear the current project
projctl run <cmd...>         # Run command inside project
projctl edit                 # Open $EDITOR_CMD in project (default: nvim)
projctl git                  # Open $GIT_UI_CMD in project (default: lazygit)
projctl logs [path]          # View logs in ./logs/ (or given path; uses lnav or tail -F)
```
## Examples:

```bash
# Set a project and edit
projctl set ~/my-app
projctl edit # Opens nvim in ~/my-app

# Run a dev server
projctl run npm state

# View logs (create ./logs/ if needed)
mkdir -p ~/my-app/logs
projctl logs # Tails all *.log files in /logs
```
For full help: `projctl --help` or `projctl help`

## Configuration

Override defaults via environment variables:
- `EDITOR_CMD=vim` (default: `nvim`)
- `GIT_UI_CMD=gitui` (default: `lazyvim`)
- `STATE=~/my-states/current_project` (default: `~/.cache/current_project`)

Export in ~/.bashrc or per-session

## Logs Setup into

The `logs` command expects a `./logs/` dir with `*.log` files. Example for a Node.js app:

```bash
mkdir -p logs
pnpm run dev  2>&1 | tee logs/app.log
projctl logs # Now view with lnav/tail
```
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

*Built with ❤️ for terminal workflows. Version 1.0.0 (2025-09-2028)*

