# bwrapman

Bubblewrap profile launcher.

Aims to be an alternative to Flatpak sandbox.

It generates sandbox options for Bubblewrap, which Flatpak does as well.

If DBus is enabled in profile, it launches a process for proxying D-Bus messages.

## Install

### Download [binary releases](https://github.com/jellyterra/bwrapman/releases)

### Build from source

```shell
cargo install bwrapman
```

## Usage

```
bwrapman (profile) (executable) [args ...]
```

```shell
bwrapman dev.toml bash -c "echo Okay"
``` 

### Requirement

- bwrap
- [xdg-dbus-proxy](https://github.com/flatpak/xdg-dbus-proxy)

## Profile

### Profile schema

```rust
pub struct Config {
    pub share_user: bool,
    pub share_ipc: bool,
    pub share_pid: bool,
    pub share_net: bool,
    pub share_uts: bool,
    pub share_cgroup: bool,
    pub share_wayland: bool,
    pub share_x11: bool,
    pub share_env: bool,
    pub keep_alive: bool,

    pub bind: Option<Vec<BindConfig>>,
    pub dev_bind: Option<Vec<BindConfig>>,
    pub symlink: Option<Vec<(String, String)>>,

    pub env: Option<HashMap<String, String>>,
    pub unset: Option<Vec<String>>,

    pub procfs: Option<String>,
    pub tmpfs: Option<Vec<String>>,

    pub uid: Option<u16>,
    pub gid: Option<u16>,

    pub hostname: Option<String>,
    pub dbus_proxy: Option<DBusProxyConfig>,
}

pub struct BindConfig {
    pub src: String,
    pub dest: String,
    pub no_fail: bool,
    pub rw: bool,
}

pub struct DBusProxyConfig {
    pub own: Vec<String>,
    pub talk: Vec<String>,
}
```

### Example

```toml
# Enable internet.
share_net = true
# Enable IPC.
share_ipc = true
# Pass all environment variables to the sandbox.
share_env = true
# Share Wayland socket.
share_wayland = true
# Share X11 socket and authority file.
share_x11 = true

procfs = "/proc"

# Mount tmpfs on /tmp
# Which will override /tmp/.X11-unix but may not affect X11 connection.
tmpfs = ["/tmp"]

[[bind]]
src = "/sandbox"
dest = "/sandbox"
# Read-write permission.
rw = true

[[bind]]
src = "/etc"
dest = "/etc"

[[bind]]
src = "/opt"
dest = "/opt"

[[bind]]
src = "/sys"
dest = "/sys"

[[bind]]
src = "/usr"
dest = "/usr"

[[bind]]
src = "/usr/bin"
dest = "/bin"

[[bind]]
src = "/usr/lib"
dest = "/lib"

[[bind]]
src = "/usr/lib64"
dest = "/lib64"

[[bind]]
src = "/nix"
dest = "/nix"
# Try binding mount or ignore on failure.
no_fail = true

[[dev_bind]]
src = "/dev"
dest = "/dev"

[dbus_proxy]
own = []
talk = ["com.canonical.AppMenu.Registrar.*"]
```
