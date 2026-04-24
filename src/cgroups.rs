use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const CRABBOX_PARENT: &str = "crabbox";

pub struct Cgroup {
    path: PathBuf,
}

impl Cgroup {
    pub fn new(container_id: &str) -> Result<Self> {
        let parent = parent_path();
        fs::create_dir_all(&parent).context("failed to create /sys/fs/cgroup/crabbox")?;

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

pub fn ensure_cgroups_v2() -> Result<()> {
    if !Path::new(CGROUP_ROOT).join("cgroup.controllers").exists() {
        bail!("cgroups v2 not found. Is your kernel configured with unified hierarchy?");
    }
    Ok(())
}

pub fn cgroup_name(container_id: &str) -> Result<String> {
    let container_id = container_id.trim();
    let id = container_id
        .strip_prefix("crabbox-")
        .unwrap_or(container_id);

    if id.is_empty() {
        bail!("container id cannot be empty");
    }

    Ok(format!("crabbox-{id}"))
}

pub fn status(container_id: &str) -> Result<()> {
    let name = cgroup_name(container_id)?;
    let path = parent_path().join(&name);

    if !path.exists() {
        bail!("container {name} not found");
    }

    let memory_current = read_cgroup_u64(&path, "memory.current")?;
    let memory_max = read_cgroup_string(&path, "memory.max")?;
    let pids_current = read_cgroup_string(&path, "pids.current")?;
    let pids_max = read_cgroup_string(&path, "pids.max")?;
    let cpu_max = read_cgroup_string(&path, "cpu.max")?;

    println!("Container:  {name}");
    println!(
        "Memory:     {} / {}",
        format_bytes(memory_current),
        format_cgroup_bytes(&memory_max)
    );
    println!("PIDs:       {pids_current} / {pids_max}");
    println!("CPU:        {cpu_max}");

    Ok(())
}

pub fn list_containers() -> Result<()> {
    let parent = parent_path();
    if !parent.exists() {
        println!("No running crabbox containers.");
        return Ok(());
    }

    let mut found = false;
    for entry in fs::read_dir(&parent)
        .with_context(|| format!("failed to read cgroup dir: {}", parent.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("crabbox-") {
            continue;
        }

        let path = entry.path();
        let memory_current = read_cgroup_u64(&path, "memory.current")?;
        let pids_current = read_cgroup_string(&path, "pids.current")?;
        println!(
            "{name}  mem={}  pids={pids_current}",
            format_bytes(memory_current)
        );
        found = true;
    }

    if !found {
        println!("No running crabbox containers.");
    }

    Ok(())
}

fn parent_path() -> PathBuf {
    Path::new(CGROUP_ROOT).join(CRABBOX_PARENT)
}

fn read_cgroup_string(path: &Path, file: &str) -> Result<String> {
    Ok(fs::read_to_string(path.join(file))
        .with_context(|| format!("failed to read {}", path.join(file).display()))?
        .trim()
        .to_string())
}

fn read_cgroup_u64(path: &Path, file: &str) -> Result<u64> {
    read_cgroup_string(path, file)?
        .parse()
        .with_context(|| format!("failed to parse {file}"))
}

fn format_cgroup_bytes(value: &str) -> String {
    match value {
        "max" => "max".to_string(),
        _ => value
            .parse()
            .map(format_bytes)
            .unwrap_or_else(|_| value.to_string()),
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GIB {
        format!("{:.1}G", bytes / GIB)
    } else if bytes >= MIB {
        format!("{:.1}M", bytes / MIB)
    } else if bytes >= KIB {
        format!("{:.1}K", bytes / KIB)
    } else {
        format!("{bytes:.0}B")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgroup_name_accepts_bare_id() {
        assert_eq!(cgroup_name("7f3a2b1c").unwrap(), "crabbox-7f3a2b1c");
    }

    #[test]
    fn cgroup_name_accepts_prefixed_id() {
        assert_eq!(cgroup_name("crabbox-7f3a2b1c").unwrap(), "crabbox-7f3a2b1c");
    }
}
