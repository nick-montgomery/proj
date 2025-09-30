use anyhow::{Context, Result};
use shellexpand::tilde;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const PROJECTS_DB: &str = "~/.config/projctl/projects.json";
pub const STATE: &str = "~/.cache/current_project";
pub const PROJECTS_DIR: &str = "~/projects";

pub fn expand_tilde(path: &str) -> PathBuf {
    PathBuf::from(tilde(path).into_owned())
}

pub fn get_projects_db() -> PathBuf {
    expand_tilde(PROJECTS_DB)
}

pub fn get_state() -> PathBuf {
    expand_tilde(STATE)
}

pub fn get_projects_dir() -> PathBuf {
    expand_tilde(PROJECTS_DIR)
}

pub fn get_current_projdir() -> Result<PathBuf> {
    let state_path = get_state();
    let mut file = std::fs::File::open(&state_path).context("No current project set")?;
    let mut path_str = String::new();
    file.read_to_string(&mut path_str)?;
    let path = PathBuf::from(path_str.trim());
    if !path.exists() {
        anyhow::bail!("Current project path does not exist");
    }
    Ok(path)
}

pub fn get_projdir(name: &str) -> Result<Option<PathBuf>> {
    let projects = super::models::load_projects()?;
    Ok(projects.projects.get(name).map(PathBuf::from))
}

/// Detects a dev command for a given directory (JS, Rust, Go, Python).
pub fn detect_dev_cmd(dir: &Path) -> String {
    if dir.join("package.json").exists() {
        "mkdir -p logs && (pnpm run dev || npm run dev || yarn dev) 2>&1 | tee logs/app.log"
            .to_string()
    } else if dir.join("Cargo.toml").exists() {
        if which::which("cargo-watch").is_ok() {
            "mkdir -p logs && cargo watch -x run 2>&1 | tee logs/app.log".to_string()
        } else {
            "mkdir -p logs && cargo run 2>&1 | tee logs/app.log".to_string()
        }
    } else if dir.join("go.mod").exists() {
        if which::which("air").is_ok() {
            "mkdir -p logs && air 2>&1 | tee logs/app.log".to_string()
        } else {
            "mkdir -p logs && go run ./... 2>&1 | tee logs/app.log".to_string()
        }
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        if which::which("uv").is_ok() {
            "mkdir -p logs && uv run python -m app 2>&1 | tee logs/app.log".to_string()
        } else {
            "mkdir -p logs && python -m app 2>&1 | tee logs/app.log".to_string()
        }
    } else {
        format!(
            "echo 'No dev command detected'; {}",
            env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
        )
    }
}

pub fn guess_frontend_dir(proj_dir: &Path) -> Option<PathBuf> {
    for sub in ["apps/web", "web", "frontend", "client", "packages/web"] {
        let candidate = proj_dir.join(sub);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn guess_backend_dir(proj_dir: &Path) -> Option<PathBuf> {
    for sub in [
        "apps/api",
        "api",
        "backend",
        "server",
        "services/api",
        "packages/api",
    ] {
        let candidate = proj_dir.join(sub);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn compose_file(proj_dir: &Path) -> Option<PathBuf> {
    for file in ["compose.yml", "docker-compose.yml"] {
        let candidate = proj_dir.join(file);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn get_autodetected_projdir(name: &str) -> Option<PathBuf> {
    let cand = get_projects_dir().join(name);
    if cand.is_dir() { Some(cand) } else { None }
}

pub fn canon(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap().join(path)
        }
    })
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    canon(a) == canon(b)
}

/// Return all auto-detected projects in `~/projets`, sorted by name.
pub fn autodetected_projects() -> Vec<(String, PathBuf)> {
    let dir = get_projects_dir();
    if !dir.exists() {
        return vec![];
    }
    let mut v = vec![];
    if let Ok(rd) = fs::read_dir(&dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_dir() {
                let name = ent.file_name().to_string_lossy().to_string();
                v.push((name, p));
            }
        }
    }
    v
}

pub fn parse_cmd(cmd: &str) -> (String, Vec<String>) {
    if let Some(mut parts) = shlex::split(cmd) {
        if parts.is_empty() {
            return ("".into(), vec![]);
        }
        let bin = parts.remove(0);
        return (bin, parts);
    }
    (cmd.to_string(), vec![])
}
