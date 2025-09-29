use crate::config::ResolvedConfig;
use crate::models::{Projects, load_projects, save_projects};
use crate::servers::setup_servers;
use crate::utils::{
    autodetected_projects, canon, get_autodetected_projdir, get_current_projdir, get_projdir,
    get_projects_dir, get_state, parse_cmd, same_path,
};
use anyhow::{Result, anyhow};
use console::{Emoji, style};
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{cmp, env, fs};
use which;

pub fn add_args(args: crate::cli::AddArgs) -> Result<()> {
    match (args.name.as_deref(), args.path.as_deref()) {
        (None, None) => add_interactive_from_auto(),
        (Some(name), None) => add_auto_by_name(name),
        (Some(name), Some(path)) => add_named_path(name, path),
        (None, Some(_)) => anyhow::bail!("Path given but no name. Use projctl add <name> <path>"),
    }
}

fn add_interactive_from_auto() -> Result<()> {
    let mut projects = load_projects()?;

    use std::collections::HashSet;
    let tracked: HashSet<_> = projects
        .projects
        .values()
        .map(|p| canon(Path::new(p)))
        .collect();

    let autos: Vec<(String, PathBuf)> = autodetected_projects()
        .into_iter()
        .filter(|(_n, p)| !tracked.contains(&canon(p)))
        .collect();

    if autos.is_empty() {
        println!(
            "No auto-detected projects to add in `{}`.",
            get_projects_dir().display()
        );
        return Ok(());
    }

    let labels: Vec<String> = autos
        .iter()
        .map(|(n, p)| format!("{n}   ({})", p.display()))
        .collect();

    let idxs = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select projects to add")
        .items(&labels)
        .interact()?;

    if idxs.is_empty() {
        println!("Nothing selected.");
        return Ok(());
    }

    for i in idxs {
        let (name, path) = &autos[i];
        insert_project(&mut projects, name.clone(), path.clone())?;
    }
    save_projects(&projects)?;
    Ok(())
}

fn add_auto_by_name(auto_name: &str) -> Result<()> {
    let found = autodetected_projects()
        .into_iter()
        .find(|(n, _)| n == auto_name);
    let Some((name, path)) = found else {
        anyhow::bail!(
            "Auto-detected project '{}' not found under {}",
            auto_name,
            get_projects_dir().display()
        );
    };
    let mut projects = load_projects()?;
    insert_project(&mut projects, name, path)?;
    save_projects(&projects)?;
    Ok(())
}

fn add_named_path(name: &str, path: &str) -> Result<()> {
    let abs = canon(Path::new(path));
    let mut projects = load_projects()?;

    if projects.projects.contains_key(name) {
        anyhow::bail!("Project '{}' already exists.", name);
    }

    if let Some((existing, _)) = projects
        .projects
        .iter()
        .find(|(_n, p)| same_path(Path::new(p), &abs))
    {
        anyhow::bail!("That path is already tracked as '{}'.", existing);
    }
    projects
        .projects
        .insert(name.to_string(), abs.display().to_string());
    save_projects(&projects)?;
    println!("Added projects {} -> {}", name, abs.display());
    Ok(())
}

fn insert_project(projects: &mut Projects, name: String, path: PathBuf) -> Result<()> {
    if projects.projects.contains_key(&name) {
        if !Confirm::new()
            .with_prompt(format!("Project '{}' exists. Overwrite path?", name))
            .default(false)
            .interact()?
        {
            println!("Skipped '{}'", name);
            return Ok(());
        }
    } else if projects
        .projects
        .values()
        .any(|p| same_path(Path::new(p), &path))
    {
        let existing = projects
            .projects
            .iter()
            .find(|(_n, p)| same_path(Path::new(p), &path))
            .unwrap()
            .0;
        println!(
            "Skipping '{}': path already tracked as '{}'.",
            name, existing
        );
        return Ok(());
    }

    projects
        .projects
        .insert(name.clone(), path.display().to_string());
    println!("Added '{}'", name);
    Ok(())
}

pub fn use_proj(name: Option<String>) -> Result<()> {
    match name {
        Some(n) => use_by_name(&n),
        None => use_interactive(),
    }
}

fn use_by_name(name: &str) -> Result<()> {
    // Enabled
    if let Some(p) = get_projdir(name)? {
        return switch_to(name, Path::new(&p), /*persist_db_current=*/ true);
    }

    // Auto-detected
    if let Some(p) = get_autodetected_projdir(name) {
        let add = Confirm::new()
            .with_prompt(format!(
                "'{}' is auto-detected but not added. Add now?",
                name
            ))
            .default(true)
            .interact()?;
        if add {
            let mut projects = load_projects()?;
            insert_project(&mut projects, name.to_string(), p.clone())?;
            save_projects(&projects)?;
            return switch_to(name, &p, true);
        } else {
            // Just switch don't save
            return switch_to(name, &p, false);
        }
    }
    anyhow::bail!(
        "Project '{}' not found. Hint: run `projctl add` to add it.",
        name
    )
}

fn use_interactive() -> Result<()> {
    let projects = load_projects()?;
    let items: Vec<(String, String)> = projects
        .projects
        .iter()
        .map(|(n, p)| (n.clone(), p.clone()))
        .collect();

    if items.is_empty() {
        println!(
            "No added projects. Hint: run `projctl add` to add from '{}'.",
            get_projects_dir().display()
        );
        return Ok(());
    }

    let current_path = get_current_projdir().ok().map(|p| canon(&p));

    let labels: Vec<String> = items.iter().map(|(n, p)| format!("{n}    {p}")).collect();

    let default_idx = items
        .iter()
        .position(|(_, p)| canon(Path::new(p)) == current_path.clone().unwrap_or_default())
        .unwrap_or(0);

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select project")
        .items(&labels)
        .default(default_idx)
        .interact()?;

    let (name, path_str) = &items[idx];
    switch_to(name, Path::new(path_str), true)
}

fn switch_to(name: &str, path: &Path, persist_db_current: bool) -> Result<()> {
    if persist_db_current {
        let mut projects = load_projects()?;
        projects.current = Some(name.to_string());
        save_projects(&projects)?;
    }
    let state_path = get_state();
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&state_path, path.display().to_string())?;
    println!("Switched to project '{}' ({})", name, path.display());
    Ok(())
}

pub fn list() -> Result<()> {
    let projects = load_projects()?;

    let current_path = get_current_projdir().ok().map(|p| canon(&p));

    // Compute padding for alignment
    let max_name = projects.projects.keys().map(|s| s.len()).max().unwrap_or(0);
    let max_name = cmp::min(max_name, 40);

    if projects.projects.is_empty() {
        println!("{}", style("no projects added yet").dim());
    } else {
        let dot = Emoji("●", "*");
        for (name, path) in &projects.projects {
            let abs = canon(Path::new(path));
            let is_current = current_path.as_ref() == Some(&abs);

            let left = if is_current {
                format!("{} {}", style(dot).green(), style(name).bold().green())
            } else {
                format!("   {}", style(name))
            };

            // Right column: path, dim; if current; tint slightly
            let right = if is_current {
                style(path).green().dim()
            } else {
                style(path).dim()
            };

            println!("{:<width$} {}", left, right, width = max_name + 3);
        }
    }

    println!(
        "\n{} {}",
        style("Hint:").bold().dim(),
        style(format!(
            "run `projctl add` to add auto-detected projects from `{}`.",
            get_projects_dir().display()
        ))
        .dim()
    );

    Ok(())
}

pub fn remove(name: String) -> Result<()> {
    let mut projects = load_projects()?;
    if projects.projects.remove(&name).is_none() {
        anyhow::bail!("Project '{}' not found", name);
    }
    save_projects(&projects)?;
    if projects.current.as_deref() == Some(&name) {
        let _ = fs::remove_file(get_current_projdir()?);
    }
    println!("Removed project '{}'", name);
    Ok(())
}

pub fn path_cmd(name: Option<String>) -> Result<()> {
    if let Some(n) = name {
        let projdir = get_projdir(&n)?.ok_or_else(|| anyhow!("Project '{}' not found", n))?;
        println!("{}", projdir.display());
    } else {
        let projdir = get_current_projdir()?;
        println!("{}", projdir.display());
    }
    Ok(())
}

pub fn run(projdir: PathBuf, cmd: Vec<String>) -> Result<()> {
    env::set_current_dir(&projdir)?;
    let output = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(&projdir)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Command failed: {:?}", cmd);
    }
    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

pub fn edit(projdir: PathBuf, cfg: &ResolvedConfig) -> Result<()> {
    let (bin, args) = parse_cmd(&cfg.editor);
    Command::new(bin)
        .args(&args)
        .arg(".")
        .current_dir(&projdir)
        .status()?;
    Ok(())
}

pub fn git(projdir: PathBuf, cfg: &ResolvedConfig) -> Result<()> {
    let (bin, mut args) = parse_cmd(&cfg.git_ui);
    args.extend(["-p".into(), projdir.display().to_string()]);
    Command::new(bin)
        .args(&args)
        .current_dir(&projdir)
        .status()?;
    Ok(())
}

pub fn logs(path: Option<String>) -> Result<()> {
    let target = if let Some(p) = path {
        env::current_dir()?.join(p).canonicalize()?
    } else {
        get_current_projdir()?
    };
    let logdir = target.join("logs");
    if logdir.exists() && fs::read_dir(&logdir)?.next().is_some() {
        if which::which("lnav").is_ok() {
            Command::new("lnav")
                .arg("*.log")
                .current_dir(&logdir)
                .status()?;
        } else {
            Command::new("tail")
                .args(["-F", "*.log"])
                .current_dir(&logdir)
                .status()?;
        }
    } else {
        println!("No logs in {}.", logdir.display())
    }
    Ok(())
}

pub fn servers(projdir: PathBuf, refresh: bool, reset: bool, kill: bool) -> Result<()> {
    setup_servers(&projdir, refresh, reset, kill)
}

pub fn create_db(name: String) -> Result<()> {
    let user = env::var("PGUSER").unwrap_or_else(|_| "postgres".to_string());
    let pass = env::var("PGPASSWORD").unwrap_or_else(|_| "postgres".to_string());
    let host = env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
    let db_cmd = format!("CREATE DATABASE \"{}\"", name);

    let output = Command::new("psql")
        .arg("-U")
        .arg(&user)
        .arg("-h")
        .arg(&host)
        .arg("-p")
        .arg(&port)
        .arg("-c")
        .arg(&db_cmd)
        .env("PGPASSWORD", pass)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create DB: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("Created database '{}'", name);
    Ok(())
}
