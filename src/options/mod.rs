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
        sub: Option<String>,
    },
    Jobs {
        sub: Option<String>,
    },
    Repo {
        sub: Option<String>,
    },
    Reset,
}
