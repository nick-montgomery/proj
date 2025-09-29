use anyhow::Result;
use clap::Parser;
use projctl::cli::{Cli, Commands};
use projctl::config::{ResolvedConfig, load_config};
use projctl::models::ensure_projects_db;
use projctl::utils::get_current_projdir;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let file_cfg = load_config(&cli.config)?;
    let cfg = ResolvedConfig::resolve(&cli, file_cfg);

    for (label, cmd) in [("editor", &cfg.editor), ("git_ui", &cfg.git_ui)] {
        if cmd
            .split_whitespace()
            .next()
            .is_some_and(|bin| which::which(bin).is_err())
        {
            eprintln!("warning: {label} command '{cmd}' not found in PATH");
        }
    }

    // Ensure db exists for relevant commands
    match &cli.command {
        Commands::Add { .. }
        | Commands::Use { .. }
        | Commands::List
        | Commands::Remove { .. }
        | Commands::Path { .. }
        | Commands::Run { .. }
        | Commands::Edit
        | Commands::Git
        | Commands::Logs { .. }
        | Commands::Servers { .. } => ensure_projects_db()?,
        _ => {}
    }

    match cli.command {
        Commands::Add(args) => projctl::commands::add_args(args),
        Commands::Use { name } => projctl::commands::use_proj(name),
        Commands::List => projctl::commands::list(),
        Commands::Remove { name } => projctl::commands::remove(name),
        Commands::Path { name } => projctl::commands::path_cmd(name),
        Commands::Run { cmd } => {
            let projdir = get_current_projdir()?;
            projctl::commands::run(projdir, cmd)
        }
        Commands::Edit => {
            let projdir = get_current_projdir()?;
            projctl::commands::edit(projdir, &cfg)
        }
        Commands::Git => {
            let projdir = get_current_projdir()?;
            projctl::commands::git(projdir, &cfg)
        }
        Commands::Logs { path } => projctl::commands::logs(path),
        Commands::Servers {
            refresh,
            reset,
            kill,
        } => {
            let proj_dir = get_current_projdir()?;
            projctl::commands::servers(proj_dir, refresh, reset, kill)
        }
        Commands::DbCreate { name } => projctl::commands::create_db(name),
    }
}
