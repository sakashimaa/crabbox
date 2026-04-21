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
```

| Argument    | Description                                |
|-------------|--------------------------------------------|
| `rootfs`    | Path to an extracted rootfs directory       |
| `command`   | Command to execute inside the container     |
| `args`      | Optional arguments passed to the command    |

| Option       | Description                                     |
|--------------|-------------------------------------------------|
| `--memory`   | Memory limit (e.g. `64M`, `1G`, `512K`)         |
| `--cpus`     | CPU limit as fractional cores (e.g. `0.5`, `2.0`) |
| `--pids`     | Maximum number of processes                     |

### Examples

```bash
# Interactive shell
sudo crabbox run /tmp/crabbox/alpine /bin/sh

# With resource limits
sudo crabbox run --memory 64M --cpus 0.5 --pids 32 /tmp/crabbox/alpine /bin/sh

# Run a single command
sudo crabbox run /tmp/crabbox/alpine /bin/echo "hello from the box"

# Check the container's OS
sudo crabbox run /tmp/crabbox/alpine /bin/cat /etc/os-release
```

## Project structure

```
src/
├── main.rs         # CLI parsing (clap)
├── config.rs       # ContainerConfig validation + parse_memory
├── container.rs    # Container lifecycle orchestration (unshare, fork, cgroups, waitpid)
├── cgroups.rs      # cgroups v2 resource limits (memory, CPU, PIDs)
├── filesystem.rs   # pivot_root, mounts (/proc, /tmp), exec
└── namespaces.rs   # Namespace setup (unshare, sethostname)
```

## Roadmap

- [x] CLI + chroot isolation
- [x] PID, mount, UTS namespaces
- [x] Container ID generation
- [x] pivot_root (replaces chroot) + tmpfs `/tmp`
- [x] cgroups v2 (memory/CPU/PID limits)
- [ ] Networking (veth, bridge, NAT)
- [ ] Image management (download rootfs by name)
- [ ] Overlay FS (layers)

## Docs

- [Configuration](docs/config.md) — how `ContainerConfig` works and what gets validated

## Dependencies

| Crate   | Purpose                              |
|---------|--------------------------------------|
| `clap`  | CLI argument parsing (derive macros) |
| `nix`   | Safe Rust wrappers over Linux syscalls |
| `anyhow`| Ergonomic error handling             |
