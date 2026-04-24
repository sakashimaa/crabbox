use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ContainerConfig {
    pub id: String,
    pub rootfs: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub hostname: String,
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
        Self::new_with_hostname(
            rootfs,
            command,
            args,
            None,
            memory_limit,
            cpu_limit,
            pids_limit,
        )
    }

    pub fn new_with_hostname(
        rootfs: PathBuf,
        command: String,
        args: Vec<String>,
        hostname: Option<String>,
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

        let id = generate_id();
        let hostname = resolve_hostname(hostname, &id)?;

        Ok(Self {
            id,
            rootfs,
            command,
            args,
            hostname,
            memory_limit,
            cpu_limit,
            pids_limit,
        })
    }

    pub fn from_toml_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let config: TomlConfig = toml::from_str(&contents)
            .with_context(|| format!("failed to parse TOML config: {}", path.display()))?;
        config.into_runtime()
    }
}

#[derive(Deserialize)]
pub struct TomlConfig {
    pub container: TomlContainer,
    pub limits: Option<TomlLimits>,
}

#[derive(Deserialize)]
pub struct TomlContainer {
    pub rootfs: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub hostname: Option<String>,
}

#[derive(Deserialize)]
pub struct TomlLimits {
    pub memory: Option<String>,
    pub cpus: Option<f64>,
    pub pids: Option<u64>,
}

impl TomlConfig {
    pub fn into_runtime(self) -> Result<ContainerConfig> {
        let memory_limit = self
            .limits
            .as_ref()
            .and_then(|limits| limits.memory.as_deref())
            .map(parse_memory)
            .transpose()?;
        let cpu_limit = self.limits.as_ref().and_then(|limits| limits.cpus);
        let pids_limit = self.limits.as_ref().and_then(|limits| limits.pids);

        ContainerConfig::new_with_hostname(
            PathBuf::from(self.container.rootfs),
            self.container.command,
            self.container.args.unwrap_or_default(),
            self.container.hostname,
            memory_limit,
            cpu_limit,
            pids_limit,
        )
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

fn resolve_hostname(hostname: Option<String>, id: &str) -> Result<String> {
    match hostname {
        Some(hostname) => {
            let hostname = hostname.trim();
            if hostname.is_empty() {
                bail!("hostname cannot be empty");
            }
            if hostname.len() > 63 {
                bail!("hostname cannot exceed 63 bytes");
            }
            Ok(hostname.to_string())
        }
        None => Ok(format!("crabbox-{id}")),
    }
}

fn generate_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:08x}", ts.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rootfs() -> PathBuf {
        let path = std::env::temp_dir().join(format!("crabbox-test-{}", generate_id()));
        fs::create_dir_all(path.join("bin")).unwrap();
        fs::write(path.join("bin/sh"), "").unwrap();
        path
    }

    #[test]
    fn toml_config_parses_args_hostname_and_limits() {
        let rootfs = test_rootfs();
        let toml = format!(
            r#"
[container]
rootfs = "{}"
command = "/bin/echo"
args = ["hello", "world"]
hostname = "mycontainer"

[limits]
memory = "64M"
cpus = 0.5
pids = 32
"#,
            rootfs.display()
        );

        let config: TomlConfig = toml::from_str(&toml).unwrap();
        let runtime = config.into_runtime().unwrap();

        assert_eq!(runtime.rootfs, rootfs);
        assert_eq!(runtime.command, "/bin/echo");
        assert_eq!(runtime.args, vec!["hello".to_string(), "world".to_string()]);
        assert_eq!(runtime.hostname, "mycontainer");
        assert_eq!(runtime.memory_limit, Some(64 * 1024 * 1024));
        assert_eq!(runtime.cpu_limit, Some(0.5));
        assert_eq!(runtime.pids_limit, Some(32));
    }

    #[test]
    fn toml_config_defaults_optional_values() {
        let rootfs = test_rootfs();
        let toml = format!(
            r#"
[container]
rootfs = "{}"
command = "/bin/sh"
"#,
            rootfs.display()
        );

        let config: TomlConfig = toml::from_str(&toml).unwrap();
        let runtime = config.into_runtime().unwrap();

        assert_eq!(runtime.args, Vec::<String>::new());
        assert!(runtime.hostname.starts_with("crabbox-"));
        assert_eq!(runtime.memory_limit, None);
        assert_eq!(runtime.cpu_limit, None);
        assert_eq!(runtime.pids_limit, None);
    }
}
