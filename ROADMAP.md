
# Projctl Roadmap

This document tracks ideas and planned improvements for future versions of **Projctl**.  
Nothing here is guaranteed — it’s a living plan. Contributions welcome!

---

## Core UX

- **Editor / Git UI detection**  
  - Try configured tool first.  
  - If missing, fall back through a list (`nvim` → `vim` → `code` / `zed`, etc.).  
  - Same for Git UI (`lazygit` → `gitui` → plain `git`).

- **Logs viewer selection**  
  - Config option to set order of preference (`lnav`, `glow`, `tail -F`).  
  - Support JSON logs with highlighting.

- **Config improvements**  
  - `projctl config edit` to open config file in editor.  
  - Validation of config schema.  
  - Project-specific overrides via `.projctl.toml` in repo.

---

## Tmux Features

- `projctl servers --list` → list active project sessions.  
- Configurable layouts (e.g., split backend/frontend/docker differently).  
- Auto-reconnect behavior if session dies.

---

## Quality of Life

- `projctl open` → open project in file manager.  
- `projctl browse` → open project in browser (for dashboards/repos).  
- `projctl info` → show summary:  
  - project path  
  - VCS info (branch, remote)  
  - detected dev command  

---

## Platform & Ecosystem

- **Shell completions**: bash, zsh, fish.  
- **Cross-platform polish**:  
  - macOS/Linux parity  
  - WSL/Windows support if demand grows  
- **Packaging**:  
  - Prebuilt binaries via GitHub releases  
  - Nix package

---

## Stretch Ideas

- **Plugin system** (small Rust crates providing extra commands).  
- **Cloud sync** of project state (dotfile sync style).  
- **Integration with task runners** (justfile, make, cargo, etc.).

---

*Have an idea? Open an [issue](https://github.com/nick-montgomery/projctl/issues) or a PR!*
