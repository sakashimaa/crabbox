use anyhow::Result;

use crate::config::ContainerConfig;
use crate::filesystem;

pub fn run(config: ContainerConfig) -> Result<()> {
    println!("[crabbox] starting container with rootfs: {}", config.rootfs.display());
    println!("[crabbox] command: {} {}", config.command, config.args.join(" "));

    filesystem::setup_rootfs(&config.rootfs)?;
    filesystem::exec_command(&config.command, &config.args)?;

    Ok(())
}
