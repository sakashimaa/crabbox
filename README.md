# crabbox

A mini container runtime written in Rust. Think Docker, but built from scratch as a learning project.

`crabbox` isolates processes using Linux kernel primitives: `pivot_root`, namespaces, and cgroups v2.

## Install

```bash
curl -sSf https://raw.githubusercontent.com/sakashimaa/crabbox/main/install.sh | sh
```

Or manually:

```bash
cargo install --git https://github.com/sakashimaa/crabbox.git
```

### Prerequisites

- Linux (x86_64)
- Rust 1.94+
- Root privileges (containers need kernel access)

## Quick start

### Prepare a rootfs

Download and extract Alpine Linux minirootfs:

```bash
mkdir -p /tmp/crabbox/alpine
cd /tmp/crabbox/alpine
curl -O https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-minirootfs-3.21.3-x86_64.tar.gz
tar xzf alpine-minirootfs-3.21.3-x86_64.tar.gz
```

### Run

```bash
sudo crabbox run /tmp/crabbox/alpine /bin/sh
```

You're now inside an isolated Alpine shell.

## Usage

```
crabbox run [OPTIONS] <rootfs> <command> [args...]
crabbox run --config <container.toml>
crabbox status <id|crabbox-id>
crabbox ps
```

| Argument  | Description                              |
| --------- | ---------------------------------------- |
| `rootfs`  | Path to an extracted rootfs directory    |
| `command` | Command to execute inside the container  |
| `args`    | Optional arguments passed to the command |

| Option     | Description                                       |
| ---------- | ------------------------------------------------- |
| `--config` | Run from a TOML config file                       |
| `--memory` | Memory limit (e.g. `64M`, `1G`, `512K`)           |
| `--cpus`   | CPU limit as fractional cores (e.g. `0.5`, `2.0`) |
| `--pids`   | Maximum number of processes                       |

### TOML config

```toml
[container]
rootfs = "/tmp/crabbox/alpine"
command = "/bin/echo"
args = ["hello", "from", "config"]
hostname = "mycontainer"

[limits]
memory = "64M"
cpus = 0.5
pids = 32
```

### Examples

```bash
# Interactive shell
sudo crabbox run /tmp/crabbox/alpine /bin/sh

# With resource limits
sudo crabbox run --memory 64M --cpus 0.5 --pids 32 /tmp/crabbox/alpine /bin/sh

# From a config file
sudo crabbox run --config container.toml

# Inspect live containers from another terminal
sudo crabbox ps
sudo crabbox status crabbox-7f3a2b1c

# Run a single command
sudo crabbox run /tmp/crabbox/alpine /bin/echo "hello from the box"

# Check the container's OS
sudo crabbox run /tmp/crabbox/alpine /bin/cat /etc/os-release
```

## Project structure

```
src/
├── main.rs         # CLI parsing (clap)
├── config.rs       # ContainerConfig validation, TOML config, parse_memory
├── container.rs    # Container lifecycle orchestration (unshare, fork, cgroups, waitpid)
├── cgroups.rs      # cgroups v2 resource limits + live status/ps
├── filesystem.rs   # pivot_root, mounts (/proc, /tmp), exec
└── namespaces.rs   # Namespace setup (unshare, sethostname)
```

## Roadmap

- [x] CLI + chroot isolation
- [x] PID, mount, UTS namespaces
- [x] Container ID generation
- [x] pivot_root (replaces chroot) + tmpfs `/tmp`
- [x] cgroups v2 (memory/CPU/PID limits)
- [x] TOML config + `status` / `ps`
- [ ] Networking (veth, bridge, NAT)
- [ ] Image management (download rootfs by name)
- [ ] Overlay FS (layers)

## Docs

- [Configuration](docs/config.md) — how `ContainerConfig` works and what gets validated

## Dependencies

| Crate    | Purpose                                |
| -------- | -------------------------------------- |
| `clap`   | CLI argument parsing (derive macros)   |
| `nix`    | Safe Rust wrappers over Linux syscalls |
| `anyhow` | Ergonomic error handling               |
| `serde`  | TOML config deserialization            |
| `toml`   | TOML parser                            |
