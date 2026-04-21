use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

pub struct Cgroup {
    path: PathBuf,
}

impl Cgroup {
    pub fn new(container_id: &str) -> Result<Self> {
        let parent = Path::new(CGROUP_ROOT).join("crabbox");
        fs::create_dir_all(&parent)
            .context("failed to create /sys/fs/cgroup/crabbox")?;

        fs::write(parent.join("cgroup.subtree_control"), "+cpu +memory +pids")
            .context("failed to enable cgroup controllers")?;

        let path = parent.join(format!("crabbox-{container_id}"));
        fs::create_dir_all(&path)
            .context(format!("failed to create cgroup dir: {}", path.display()))?;

        Ok(Self { path })
    }

    pub fn set_memory_limit(&self, bytes: u64) -> Result<()> {
        fs::write(self.path.join("memory.max"), bytes.to_string())
            .context("failed to write memory.max")?;
        Ok(())
    }

    pub fn set_cpu_limit(&self, cpus: f64) -> Result<()> {
        let period: u64 = 100_000;
        let quota = (cpus * period as f64) as u64;
        fs::write(self.path.join("cpu.max"), format!("{quota} {period}"))
            .context("failed to write cpu.max")?;
        Ok(())
    }

    pub fn set_pids_limit(&self, max: u64) -> Result<()> {
        fs::write(self.path.join("pids.max"), max.to_string())
            .context("failed to write pids.max")?;
        Ok(())
    }

    pub fn add_pid(&self, pid: u32) -> Result<()> {
        fs::write(self.path.join("cgroup.procs"), pid.to_string())
            .context("failed to add pid to cgroup")?;
        Ok(())
    }
}

impl Drop for Cgroup {
    fn drop(&mut self) {
        let _ = fs::write(self.path.join("cgroup.kill"), "1");

        for _ in 0..100 {
            match fs::read_to_string(self.path.join("cgroup.procs")) {
                Ok(procs) if procs.trim().is_empty() => break,
                _ => std::thread::sleep(std::time::Duration::from_millis(20)),
            }
        }

        let _ = fs::remove_dir(&self.path);
    }
}
