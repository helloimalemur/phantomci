use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Add {
        path: Option<String>,
        branch: Option<String>,
    },
    Configure {
        sub: String,
    },
    Logs {
        /// Backward-compatible positional filter (substring of repo URL/name)
        sub: Option<String>,
        /// Filter by exact repo URL/path
        #[arg(long)]
        repo: Option<String>,
        /// Filter by branch name (best-effort; matches inside message)
        #[arg(long)]
        branch: Option<String>,
        /// Limit number of log rows (0 = no limit)
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    Jobs {
        sub: Option<String>,
    },
    Repo {
        sub: Option<String>,
    },
    Reset,
}
