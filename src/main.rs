mod cgroups;
mod config;
mod container;
mod filesystem;
mod namespaces;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{ContainerConfig, parse_memory};
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
        /// Memory limit (e.g. 64M, 1G, 512K)
        #[arg(long)]
        memory: Option<String>,

        /// CPU limit as fractional cores (e.g. 0.5, 2.0)
        #[arg(long)]
        cpus: Option<f64>,

        /// Maximum number of processes
        #[arg(long)]
        pids: Option<u64>,

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
            memory,
            cpus,
            pids,
            rootfs,
            command,
            args,
        } => {
            let memory_limit = memory.as_deref().map(parse_memory).transpose()?;
            let config = ContainerConfig::new(rootfs, command, args, memory_limit, cpus, pids)?;
            container::run(config)?;
        }
    }

    Ok(())
}
