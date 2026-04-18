# Configuration

## ContainerConfig

`ContainerConfig` is the central struct that holds everything needed to launch a container. It's created from CLI arguments and validated before any syscalls happen.

```rust
pub struct ContainerConfig {
    pub rootfs: PathBuf,    // path to the extracted rootfs
    pub command: String,    // command to run inside the container
    pub args: Vec<String>,  // arguments for the command
}
```

### Validation

`ContainerConfig::new()` checks two things before accepting the config:

1. **rootfs exists** — the directory must be present on disk
2. **rootfs contains /bin/sh** — a basic sanity check that this is a real rootfs (checked via `symlink_metadata` to handle symlinks correctly)

If either check fails, you get a clear error:

```
Error: rootfs path does not exist: /tmp/crabbox/nonexistent
Error: rootfs is missing /bin/sh — is this a valid rootfs? (/tmp/crabbox/empty)
```

### How CLI maps to config

The CLI command:

```bash
crabbox run /tmp/crabbox/alpine /bin/echo hello world
```

Produces:

```rust
ContainerConfig {
    rootfs: PathBuf("/tmp/crabbox/alpine"),
    command: String("/bin/echo"),
    args: vec!["hello", "world"],
}
```

An interactive shell with no extra args:

```bash
crabbox run /tmp/crabbox/alpine /bin/sh
```

```rust
ContainerConfig {
    rootfs: PathBuf("/tmp/crabbox/alpine"),
    command: String("/bin/sh"),
    args: vec![],
}
```

## Container environment

The container process gets a clean environment, not the host's. These variables are set automatically:

| Variable | Value                                          | Why                                    |
|----------|------------------------------------------------|----------------------------------------|
| `PATH`   | `/bin:/usr/bin:/sbin:/usr/sbin:/usr/local/bin`  | So commands like `ls`, `cat` are found |
| `HOME`   | `/root`                                        | Default home directory                 |
| `TERM`   | `xterm-256color`                               | Terminal colors work properly           |

This is done via `execvpe` instead of `execvp` — the `e` variant lets us pass an explicit environment array rather than inheriting from the host.

Without this, the container would inherit the host's `PATH` (e.g. `/home/user/.cargo/bin:/usr/local/sbin:...`) which doesn't exist inside the chroot.

## Flow

```
CLI args
  → ContainerConfig::new() (validate)
  → container::run()
    → filesystem::setup_rootfs()   (chroot + chdir)
    → filesystem::exec_command()   (execvpe with clean env)
```

The process replaces itself via `exec` — there's no parent process waiting. The container *is* the process.
