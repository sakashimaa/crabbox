# crabbox

A mini container runtime in Rust — a Docker clone built from scratch as a learning project.

## Project overview

`crabbox` isolates processes using Linux kernel primitives. The three pillars:
1. **Filesystem isolation** — chroot/pivot_root
2. **Namespace isolation** — PID, mount, UTS, network (via `unshare`)
3. **Resource limits** — cgroups v2 (future)

## Current state

Day 1 complete. Working chroot into Alpine minirootfs.

What works:
- CLI: `crabbox run <rootfs> <command> [args...]`
- chroot into rootfs with clean environment (PATH/HOME/TERM)
- Config validation (rootfs exists, has /bin/sh via symlink_metadata)

What's next (see `docs/plan/1-3.md` for full plan):
- Day 2: namespaces (PID, mount, UTS) via unshare + fork
- Day 3: pivot_root, cleanup, container ID
- Day 4+: cgroups, networking, image management, overlay FS, Crabfile

## Architecture

```
src/
├── main.rs         # CLI parsing (clap derive)
├── config.rs       # ContainerConfig — validates rootfs and command
├── container.rs    # Orchestrates container lifecycle
├── filesystem.rs   # chroot/pivot_root, mounts, execvpe
└── namespaces.rs   # Namespace setup (placeholder, Day 2)
```

Flow: CLI args → ContainerConfig::new() → container::run() → filesystem::setup_rootfs() → filesystem::exec_command()

## Build and test

```bash
cargo build
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/sh
```

Alpine minirootfs must be extracted at `/tmp/crabbox/alpine/` before running. See README.md for setup instructions.

Note: `sudo cargo run` won't work — sudo resets PATH and drops `~/.cargo/bin`. Always build first, then sudo the binary.

## Dependencies

- `clap` (derive) — CLI parsing
- `nix` 0.29 (features: process, mount, sched, hostname, fs) — safe Linux syscall wrappers
- `anyhow` — error handling with bail!/context()

## Conventions

- Use `anyhow::Result` and `bail!` for errors, `.context()` for adding messages to syscall failures
- Use `nix` crate for all syscalls — no raw `unsafe { libc::... }`
- Rootfs symlinks (Alpine uses absolute symlinks like `bin/sh -> /bin/busybox`) — always use `symlink_metadata()` instead of `exists()` when checking paths inside rootfs from the host
- Container gets a clean env via `execvpe`, not host's env via `execvp`
- Requires root to run (chroot, namespaces, mount are privileged operations)
