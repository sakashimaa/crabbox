use anyhow::{Context, Result};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::sethostname;

pub fn unshare_namespaces() -> Result<()> {
    unshare(
        CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUTS,
    )
    .context("failed to unshare namespaces — are you running as root?")?;
    Ok(())
}

pub fn set_hostname(hostname: &str) -> Result<()> {
    sethostname(hostname).context("failed to set hostname")?;
    Ok(())
}
