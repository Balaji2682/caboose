use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "caboose")]
#[command(about = "Rails development tool - process manager and monitoring", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start all processes or a specific process
    Dev {
        /// Optional process name to start
        process: Option<String>,
    },
    /// Stop all processes
    Stop,
    /// Restart a process
    Restart {
        /// Process name to restart
        process: String,
    },
    /// Show logs for a process
    Logs {
        /// Process name
        process: String,
    },
    /// List all processes
    Ps,
}
