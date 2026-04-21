use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ContainerConfig {
    pub id: String,
    pub rootfs: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub memory_limit: Option<u64>,
    pub cpu_limit: Option<f64>,
    pub pids_limit: Option<u64>,
}

impl ContainerConfig {
    pub fn new(
        rootfs: PathBuf,
        command: String,
        args: Vec<String>,
        memory_limit: Option<u64>,
        cpu_limit: Option<f64>,
        pids_limit: Option<u64>,
    ) -> Result<Self> {
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
            memory_limit,
            cpu_limit,
            pids_limit,
        })
    }
}

pub fn parse_memory(input: &str) -> Result<u64> {
    let input = input.trim();
    if input.is_empty() {
        bail!("memory limit cannot be empty");
    }

    let (num_str, unit) = input.split_at(input.len() - 1);
    let num: u64 = num_str
        .parse()
        .context(format!("invalid memory value: {input}"))?;

    match unit.to_ascii_uppercase().as_str() {
        "K" => Ok(num * 1024),
        "M" => Ok(num * 1024 * 1024),
        "G" => Ok(num * 1024 * 1024 * 1024),
        _ => bail!("unknown memory unit '{unit}' in '{input}' — use K, M, or G"),
    }
}

fn generate_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:08x}", ts.subsec_nanos())
}
