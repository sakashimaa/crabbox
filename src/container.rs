use anyhow::{Context, Result};
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, fork};

use crate::config::ContainerConfig;
use crate::{filesystem, namespaces};

pub fn run(config: ContainerConfig) -> Result<()> {
    println!(
        "[crabbox] starting container {} with rootfs: {}",
        config.id,
        config.rootfs.display()
    );
    println!("[crabbox] command: {} {}", config.command, config.args.join(" "));

    namespaces::unshare_namespaces()?;

    let hostname = format!("crabbox-{}", config.id);

    match unsafe { fork() }.context("fork failed")? {
        ForkResult::Parent { child } => {
            waitpid(child, None).context("waitpid failed")?;
        }
        ForkResult::Child => {
            namespaces::set_hostname(&hostname)?;
            filesystem::setup_rootfs(&config.rootfs)?;
            filesystem::mount_proc()?;
            filesystem::exec_command(&config.command, &config.args)?;
        }
    }

    Ok(())
}
