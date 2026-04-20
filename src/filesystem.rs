use std::ffi::CString;
use std::path::Path;

use anyhow::{Context, Result};
use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::unistd::{chdir, execvpe, pivot_root};

pub fn setup_rootfs(rootfs: &Path) -> Result<()> {
    // Make / private: pivot_root fails on MS_SHARED parents, and CLONE_NEWNS
    // inherits propagation from the host (systemd mounts / shared).
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
    .context("failed to remount / as private")?;

    // pivot_root requires new_root to be a mount point, so bind it onto itself.
    mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )
    .context("failed to bind-mount rootfs onto itself")?;

    let put_old = rootfs.join("oldroot");
    std::fs::create_dir_all(&put_old).context("failed to create oldroot directory")?;

    pivot_root(rootfs, &put_old).context("pivot_root failed")?;
    chdir("/").context("failed to chdir into new root")?;

    umount2("/oldroot", MntFlags::MNT_DETACH).context("failed to unmount old root")?;
    std::fs::remove_dir("/oldroot").context("failed to remove /oldroot")?;

    Ok(())
}

pub fn exec_command(cmd: &str, args: &[String]) -> Result<()> {
    let cmd_cstr = CString::new(cmd).context("invalid command string")?;

    let mut argv: Vec<CString> = vec![cmd_cstr.clone()];
    for arg in args {
        argv.push(CString::new(arg.as_str()).context("invalid argument string")?);
    }

    let env = [
        CString::new("PATH=/bin:/usr/bin:/sbin:/usr/sbin:/usr/local/bin").unwrap(),
        CString::new("HOME=/root").unwrap(),
        CString::new("TERM=xterm-256color").unwrap(),
    ];

    execvpe(&cmd_cstr, &argv, &env).context("failed to exec command")?;

    unreachable!()
}

pub fn mount_proc() -> Result<()> {
    mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )
    .context("failed to mount /proc")?;
    Ok(())
}

pub fn mount_tmp() -> Result<()> {
    mount(
        Some("tmpfs"),
        "/tmp",
        Some("tmpfs"),
        MsFlags::empty(),
        None::<&str>,
    )
    .context("failed to mount /tmp")?;
    Ok(())
}
