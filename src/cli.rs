use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "projctl",
    version = "2.0.0",
    about = "Helper to manage project context across a system"
)]
pub struct Cli {
    #[arg(long)]
    pub editor: Option<String>,

    #[arg(long, value_name = "CMD")]
    pub git_ui: Option<String>,

    #[arg(long, value_name = "FILE", default_value_os_t = crate::config::default_config_path())]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a named project
    Add(AddArgs),
    /// Switch to a new project
    Use {
        /// Project name
        name: Option<String>,
    },
    /// List all added projects (including auto-detected)
    List,
    /// Remove a named project
    Remove {
        /// Project name
        name: String,
    },
    /// Print current project path (or named project's path)
    Path {
        /// Optional project name
        name: Option<String>,
    },
    /// Run a command inside the current project
    Run {
        /// Command and args
        cmd: Vec<String>,
    },
    /// Open editor in current project
    Edit,
    /// Open git UI in current project
    Git,
    /// Open logs in current project (or given path)
    Logs {
        /// Optional path
        path: Option<String>,
    },
    /// Setup/attach tmux servers session for current project
    Servers {
        /// Setup flag (creates/reseeds if needed)
        #[arg(long, conflicts_with_all = ["reset, kill"])]
        refresh: bool,

        #[arg(long, conflicts_with_all = ["refresh, kill"])]
        reset: bool,

        #[arg(long, conflicts_with_all = ["refresh, reset"])]
        kill: bool,
    },
    /// Create a Postgres DB
    DbCreate {
        /// Database name
        name: String,
    },
}

#[derive(Args)]
pub struct AddArgs {
    /// Either: <autoName> OR <path>. If omitted, interactive auto-pick list is shown.
    pub name: Option<String>,
    /// Path to project (when adding an arbitrary path)
    pub path: Option<String>,
}
