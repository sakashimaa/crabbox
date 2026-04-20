# Configuration

## ContainerConfig

`ContainerConfig` is the central struct that holds everything needed to launch a container. It's created from CLI arguments and validated before any syscalls happen.

```rust
pub struct ContainerConfig {
    pub id: String,         // auto-generated 8-char hex ID
    pub rootfs: PathBuf,    // path to the extracted rootfs
    pub command: String,    // command to run inside the container
    pub args: Vec<String>,  // arguments for the command
}
```

### The `id` field

`ContainerConfig::new()` auto-generates a short hex ID (e.g. `7f3a2b1c`) from the system clock's nanoseconds. It's used for the container hostname (`crabbox-7f3a2b1c`) and will become the handle for future `list`/`stop`/`inspect` commands. No dependency on the `uuid` crate — nanosecond resolution is plenty unique per-machine-per-run.

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
    id: String("7f3a2b1c"),  // auto-generated
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
    id: String("9c1d4e8a"),  // auto-generated
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
  → ContainerConfig::new()           (validate + generate id)
  → container::run()
    → namespaces::unshare_namespaces()  (CLONE_NEWPID | CLONE_NEWNS | CLONE_NEWUTS)
    → fork()
        ├─ parent: waitpid(child)       (block until container exits)
        └─ child (PID 1 in new namespace):
             → namespaces::set_hostname("crabbox-<id>")
             → filesystem::setup_rootfs()   (MS_PRIVATE + bind + pivot_root + cleanup)
             → filesystem::mount_proc()     (mount procfs at /proc)
             → filesystem::mount_tmp()      (mount tmpfs at /tmp)
             → filesystem::exec_command()   (execvpe with clean env)
```

The parent process stays alive for the lifetime of the container, waiting on the child via `waitpid`. When the child exits (e.g. user hits Ctrl+D), `waitpid` returns and the parent cleans up naturally — its mount namespace dies with it, taking all container-only mounts (like `/proc` and `/tmp`) with it.

### Why fork?

`unshare(CLONE_NEWPID)` only affects **future children**, not the caller. So we unshare first, then fork — the child is PID 1 in the fresh PID namespace. The parent stays in the host's PID space, which is what lets it `waitpid` on the child.

## Filesystem setup (pivot_root)

`setup_rootfs()` replaces the process's view of `/` with the rootfs directory. It's stricter than `chroot`: the old root gets detached entirely, so a privileged process can't escape back out.

The sequence:

1. **Remount `/` as `MS_PRIVATE` (recursive)** — the kernel rejects `pivot_root` if the new root's parent mount has shared propagation. Systemd mounts `/` as shared on most distros, and `CLONE_NEWNS` inherits that propagation. Without this step, `pivot_root` fails with `EINVAL`.
2. **Bind-mount the rootfs onto itself** — `pivot_root` requires `new_root` to be a mount point, not just a directory. A self-bind-mount is the cheapest way to satisfy this.
3. **Create `rootfs/oldroot`** — the `put_old` target where the old root gets parked.
4. **`pivot_root(rootfs, rootfs/oldroot)`** — the actual root swap. New root is now `/`, old root is at `/oldroot`.
5. **`chdir("/")`** — move the CWD into the new root (otherwise it still points inside the old root).
6. **`umount2("/oldroot", MNT_DETACH)` + `remove_dir("/oldroot")`** — detach the old root (lazy unmount, safe even if something is still using it) and remove the empty mountpoint. The container no longer has any reference to the host filesystem.

After `setup_rootfs` returns, `/proc` and `/tmp` are then mounted fresh inside the new root.
