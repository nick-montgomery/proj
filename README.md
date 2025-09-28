# Proj

[![AUR](https://img.shields.io/aur/version/proj?color=orange)](https://aur.archlinux.org/packages/proj)

A lightweight bash script to manage project context across your terminal sessions. Quickly switch directories, run commands, edit files, browse git, or tail logs—all tied to a "current project" state file.

## Features

- **Set/Clear Project**: Switch between projects with a single command; state persists in `~/.cache/current_project`.
- **Contextual Commands**: `proj run`, `proj edit`, `proj git` automatically `cd` into the project dir.
- **Logs Viewer**: Tail or browse logs in `./logs/` using `lnav` (fallback to `tail -F`).
- **Customizable**: Override editor (`EDITOR_CMD=vim`) or git UI (`GIT_UI_CMD=lazygit`).

No dependencies beyond bash; optional tools (nvim, lazygit, lnav) enhance UX.

## Installation

### Arch Linux (AUR)
Install via an AUR helper:
```bash
yay -S proj  # Or paru, aurman, etc.
```
Manual build:

```bash
git clone https://aur.archlinux.org/proj.git
cd proj
makepg -si
```
### Manual (Any Linux/macOS)

1. Download the script:

```bash
curl -L0 https://raw.githubusercontent.com/nick-montgomery/proj/main/proj
chmod +x proj
sudo mv proj /usr/local/bin # Or ~/bin/
```
2. (Optional) Set up state dir: `mkdir -p ~/.cache`

### From Source

```bash
git clone https://github.com/nick-montgomery/proj.git
cd proj
sudo make install # TODO: Add makefile
```
## Usage

```bash
proj set /path/to/project # Set current project (absolute/relative)
proj path                 # Print current project path
proj clear                # Clear the current project
proj run <cmd...>         # Run command inside project
proj edit                 # Open $EDITOR_CMD in project (default: nvim)
proj git                  # Open $GIT_UI_CMD in project (default: lazygit)
proj logs [path]          # View logs in ./logs/ (or given path; uses lnav or tail -F)
```
## Examples:

```bash
# Set a project and edit
proj set ~/my-app
proj edit # Opens nvim in ~/my-app

# Run a dev server
proj run npm state

# View logs (create ./logs/ if needed)
mkdir -p ~/my-app/logs
proj logs # Tails all *.log files in /logs
```
For full help: `proj --help` or `proj help`

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
proj logs # Now view with lnav/tail
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

