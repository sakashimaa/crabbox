use std::ffi::CString;
use std::path::Path;

use anyhow::{Context, Result};
use nix::unistd::{chdir, chroot, execvpe};

pub fn setup_rootfs(rootfs: &Path) -> Result<()> {
    let rootfs_str = rootfs
        .to_str()
        .context("rootfs path is not valid UTF-8")?;

    chroot(rootfs_str).context("failed to chroot — are you running as root?")?;
    chdir("/").context("failed to chdir into new root")?;

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
