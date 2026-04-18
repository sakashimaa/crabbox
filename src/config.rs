use anyhow::{Result, bail};
use std::path::PathBuf;

pub struct ContainerConfig {
    pub rootfs: PathBuf,
    pub command: String,
    pub args: Vec<String>,
}

impl ContainerConfig {
    pub fn new(rootfs: PathBuf, command: String, args: Vec<String>) -> Result<Self> {
        if !rootfs.exists() {
            bail!("rootfs path does not exist: {}", rootfs.display());
        }

        if rootfs.join("bin/sh").symlink_metadata().is_err() {
            bail!(
                "rootfs is missing /bin/sh — is this a valid rootfs? ({})",
                rootfs.display()
            );
        }

        Ok(Self {
            rootfs,
            command,
            args,
        })
    }
}
