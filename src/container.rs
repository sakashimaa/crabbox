use anyhow::{Context, Result};
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, fork};

use crate::cgroups::Cgroup;
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
            let cgroup = Cgroup::new(&config.id)?;

            if let Some(mem) = config.memory_limit {
                cgroup.set_memory_limit(mem)?;
            }
            if let Some(cpus) = config.cpu_limit {
                cgroup.set_cpu_limit(cpus)?;
            }
            if let Some(pids) = config.pids_limit {
                cgroup.set_pids_limit(pids)?;
            }

            cgroup.add_pid(child.as_raw() as u32)?;
            waitpid(child, None).context("waitpid failed")?;
        }
        ForkResult::Child => {
            namespaces::unshare_mount()?;
            namespaces::set_hostname(&hostname)?;
            filesystem::setup_rootfs(&config.rootfs)?;
            filesystem::mount_proc()?;
            filesystem::mount_tmp()?;
            filesystem::exec_command(&config.command, &config.args)?;
        }
    }

    Ok(())
}
