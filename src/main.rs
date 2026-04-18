mod config;
mod container;
mod filesystem;
mod namespaces;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::ContainerConfig;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "crabbox", about = "A mini container runtime")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a command in a new container
    Run {
        /// Path to the extracted rootfs
        rootfs: PathBuf,

        /// Command to execute inside the container
        command: String,

        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rootfs,
            command,
            args,
        } => {
            let config = ContainerConfig::new(rootfs, command, args)?;
            container::run(config)?;
        }
    }

    Ok(())
}
