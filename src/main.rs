mod cgroups;
mod config;
mod container;
mod filesystem;
mod namespaces;

use anyhow::Result;
use anyhow::bail;
use clap::{Parser, Subcommand};
use config::{ContainerConfig, parse_memory};
use nix::unistd::getuid;
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
        /// Path to a TOML container config
        #[arg(long)]
        config: Option<PathBuf>,

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
        rootfs: Option<PathBuf>,

        /// Command to execute inside the container
        command: Option<String>,

        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Show live resource usage for a running container
    Status {
        /// Container ID, with or without the crabbox- prefix
        container_id: String,
    },

    /// List running crabbox containers
    Ps,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            config,
            memory,
            cpus,
            pids,
            rootfs,
            command,
            args,
        } => {
            preflight()?;
            let config = run_config(config, memory, cpus, pids, rootfs, command, args)?;
            container::run(config)?;
        }
        Commands::Status { container_id } => {
            preflight()?;
            cgroups::status(&container_id)?;
        }
        Commands::Ps => {
            preflight()?;
            cgroups::list_containers()?;
        }
    }

    Ok(())
}

fn run_config(
    config: Option<PathBuf>,
    memory: Option<String>,
    cpus: Option<f64>,
    pids: Option<u64>,
    rootfs: Option<PathBuf>,
    command: Option<String>,
    args: Vec<String>,
) -> Result<ContainerConfig> {
    if let Some(config) = config {
        if memory.is_some()
            || cpus.is_some()
            || pids.is_some()
            || rootfs.is_some()
            || command.is_some()
            || !args.is_empty()
        {
            bail!("--config cannot be combined with resource flags, rootfs, command, or args");
        }

        return ContainerConfig::from_toml_file(&config);
    }

    let rootfs = rootfs.ok_or_else(|| {
        anyhow::anyhow!("run requires <rootfs> <command> unless --config is used")
    })?;
    let command = command.ok_or_else(|| {
        anyhow::anyhow!("run requires <rootfs> <command> unless --config is used")
    })?;
    let memory_limit = memory.as_deref().map(parse_memory).transpose()?;

    ContainerConfig::new(rootfs, command, args, memory_limit, cpus, pids)
}

fn preflight() -> Result<()> {
    if getuid().as_raw() != 0 {
        bail!("crabbox requires root. Run with sudo.");
    }
    cgroups::ensure_cgroups_v2()
}
