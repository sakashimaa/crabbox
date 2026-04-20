use anyhow::{Result, bail};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ContainerConfig {
    pub id: String,
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
            id: generate_id(),
            rootfs,
            command,
            args,
        })
    }
}

fn generate_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:08x}", ts.subsec_nanos())
}
