# crabbox Agent Notes

This file is for Codex and other coding agents working in this repository.

## Project overview

`crabbox` is a mini container runtime in Rust. It isolates processes with Linux
kernel primitives:

1. Filesystem isolation with `pivot_root`
2. PID, mount, and UTS namespaces through `nix`
3. Resource limits and live inspection through cgroups v2

## Current state

Days 1-5 are complete:

- `crabbox run [--memory <LIMIT>] [--cpus <FLOAT>] [--pids <COUNT>] <rootfs> <command> [args...]`
- `crabbox run --config container.toml`
- `crabbox status <id|crabbox-id>`
- `crabbox ps`
- Container IDs are 8-char hex strings; cgroups are named `crabbox-<id>`.
- Default hostname is `crabbox-<id>`; TOML config can override it.
- Cgroups live under `/sys/fs/cgroup/crabbox/crabbox-<id>`.

## Architecture

```
src/
|-- main.rs         # clap CLI and preflight checks
|-- config.rs       # runtime config, TOML config, validation, parse_memory
|-- container.rs    # container lifecycle orchestration
|-- cgroups.rs      # cgroups v2 limits, status, ps
|-- filesystem.rs   # pivot_root, mounts, execvpe
`-- namespaces.rs   # namespace and hostname setup
```

Flow: CLI args or TOML config become a runtime `ContainerConfig`, then
`container::run()` unshares namespaces, forks, sets up cgroups in the parent,
and configures hostname/rootfs/mounts/exec in the child.

## Build and test

```bash
cargo test
cargo build
sudo ./target/debug/crabbox run /tmp/crabbox/alpine /bin/sh
```

An Alpine rootfs is expected at `/tmp/crabbox/alpine/` for manual runs. Build
first, then run the binary with `sudo`; `sudo cargo run` can fail when sudo
resets PATH.

## Conventions

- Use `anyhow::Result`, `bail!`, and `.context()` for errors.
- Use `nix` wrappers for syscalls instead of raw libc where available.
- Use `symlink_metadata()` for rootfs checks because Alpine uses absolute
  symlinks such as `bin/sh -> /bin/busybox`.
- Keep CLI flags and TOML config converging into `ContainerConfig`.
- Do not mix `--config` with CLI overrides until explicit precedence rules are
  added.
- Keep status/ps as live cgroup views only; there is no persistent container
  metadata store yet.
