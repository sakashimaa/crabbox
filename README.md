# crabbox

A mini container runtime written in Rust. Think Docker, but built from scratch as a learning project.

`crabbox` isolates processes using Linux kernel primitives: `chroot`/`pivot_root`, namespaces, and (soon) cgroups.

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

### Build and run

```bash
cargo build
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/sh
```

You're now inside an isolated Alpine shell.

## Usage

```
crabbox run <rootfs> <command> [args...]
```

| Argument    | Description                                |
|-------------|--------------------------------------------|
| `rootfs`    | Path to an extracted rootfs directory       |
| `command`   | Command to execute inside the container     |
| `args`      | Optional arguments passed to the command    |

### Examples

```bash
# Interactive shell
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/sh

# Run a single command
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/echo "hello from the box"

# Check the container's OS
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/cat /etc/os-release
```

## Project structure

```
src/
├── main.rs         # CLI parsing (clap)
├── config.rs       # ContainerConfig validation
├── container.rs    # Container lifecycle orchestration
├── filesystem.rs   # chroot/pivot_root, mounts, exec
└── namespaces.rs   # Namespace setup (unshare, fork)
```

## Roadmap

- [x] CLI + chroot isolation
- [ ] PID, mount, UTS namespaces
- [ ] pivot_root (replace chroot)
- [ ] cgroups v2 (memory/CPU limits)
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
