# crabbox

A mini container runtime in Rust — a Docker clone built from scratch as a learning project.

## Project overview

`crabbox` isolates processes using Linux kernel primitives. The three pillars:
1. **Filesystem isolation** — pivot_root
2. **Namespace isolation** — PID, mount, UTS, network (via `unshare`)
3. **Resource limits** — cgroups v2

## Current state

Days 1–5 complete. Fully isolated container with resource limits, TOML config input, and live cgroup inspection.

What works:
- CLI: `crabbox run [--memory <LIMIT>] [--cpus <FLOAT>] [--pids <COUNT>] <rootfs> <command> [args...]`
- TOML config: `crabbox run --config container.toml`
- Live inspection: `crabbox status <id|crabbox-id>` and `crabbox ps`
- Container ID (8-char hex) + hostname `crabbox-<id>`
- Optional TOML hostname override
- Namespaces: `unshare(CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS)` + `fork` so child is PID 1
- `pivot_root` filesystem swap (MS_PRIVATE remount → self-bind → pivot → detach `/oldroot`)
- Mounts inside container: `/proc` (procfs), `/tmp` (tmpfs)
- Clean environment (`execvpe` with explicit PATH/HOME/TERM)
- Config validation (rootfs exists, has `/bin/sh` via symlink_metadata; hostname non-empty/max 63 bytes)
- cgroups v2: memory limit (`--memory 64M`), CPU limit (`--cpus 0.5`), PID limit (`--pids 32`)
- Cgroup lifecycle: create → set limits → add PID → cleanup on Drop

What's next (Day 6+):
- Networking (veth, bridge, NAT)
- Image management (download rootfs by name)
- Overlay FS (layers)
- Crabfile / crab-compose

## Architecture

```
src/
├── main.rs         # CLI parsing (clap derive)
├── config.rs       # ContainerConfig + TOML config + parse_memory
├── container.rs    # Orchestrates container lifecycle
├── cgroups.rs      # cgroups v2 limits + status/ps
├── filesystem.rs   # pivot_root, mounts (/proc, /tmp), execvpe
└── namespaces.rs   # unshare_namespaces, set_hostname
```

Flow: CLI args or TOML config → `ContainerConfig` runtime → `container::run()` → `namespaces::unshare_namespaces()` → `fork()` → parent: `Cgroup::new` → set limits → `add_pid` → `waitpid` → Drop cleanup; child: `set_hostname(config.hostname)` → `filesystem::setup_rootfs` (pivot_root) → `mount_proc` → `mount_tmp` → `exec_command`.

## Build and test

```bash
cargo build
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/sh
```

Alpine minirootfs must be extracted at `/tmp/crabbox/alpine/` before running. See README.md for setup instructions.

Note: `sudo cargo run` won't work — sudo resets PATH and drops `~/.cargo/bin`. Always build first, then sudo the binary.

## Dependencies

- `clap` (derive) — CLI parsing
- `nix` 0.29 (features: process, mount, sched, hostname, fs, user) — safe Linux syscall wrappers
- `anyhow` — error handling with bail!/context()
- `serde` (derive) + `toml` — config file parsing

## Conventions

- Use `anyhow::Result` and `bail!` for errors, `.context()` for adding messages to syscall failures
- Use `nix` crate for all syscalls — no raw `unsafe { libc::... }`
- Rootfs symlinks (Alpine uses absolute symlinks like `bin/sh -> /bin/busybox`) — always use `symlink_metadata()` instead of `exists()` when checking paths inside rootfs from the host
- Container gets a clean env via `execvpe`, not host's env via `execvp`
- Requires root to run (pivot_root, namespaces, mount are privileged operations)
- Before `pivot_root`, always remount `/` as `MS_PRIVATE | MS_REC` — otherwise the kernel returns `EINVAL` (mount propagation inherited from systemd via `CLONE_NEWNS`)
- Cgroups live under `/sys/fs/cgroup/crabbox/crabbox-<id>`; status/ps must use that parent path, not the cgroup root.
