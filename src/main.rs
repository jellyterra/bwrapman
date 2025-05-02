// Copyright 2025 Jelly Terra <jellyterra@symboltics.com>
// Use of this source code form is governed under the MIT license.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::remove_file;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, process};

const fn default_false() -> bool {
    false
}

#[derive(Deserialize)]
pub struct Config {
    pub bind: Option<Vec<BindConfig>>,
    pub dev_bind: Option<Vec<BindConfig>>,

    pub symlink: Option<Vec<(String, String)>>,

    pub env: Option<HashMap<String, String>>,
    pub unset: Option<Vec<String>>,

    #[serde(default = "default_false")]
    pub share_user: bool,
    #[serde(default = "default_false")]
    pub share_ipc: bool,
    #[serde(default = "default_false")]
    pub share_pid: bool,
    #[serde(default = "default_false")]
    pub share_net: bool,
    #[serde(default = "default_false")]
    pub share_uts: bool,
    #[serde(default = "default_false")]
    pub share_cgroup: bool,

    #[serde(default = "default_false")]
    pub share_wayland: bool,
    #[serde(default = "default_false")]
    pub share_x11: bool,

    #[serde(default = "default_false")]
    pub share_env: bool,

    #[serde(default = "default_false")]
    pub keep_alive: bool,

    pub procfs: Option<String>,
    pub tmpfs: Option<Vec<String>>,

    pub uid: Option<u16>,
    pub gid: Option<u16>,

    pub hostname: Option<String>,

    pub dbus_proxy: Option<DBusProxyConfig>,
}

#[derive(Deserialize)]
pub struct BindConfig {
    pub src: String,
    pub dest: String,

    #[serde(default = "default_false")]
    pub no_fail: bool,
    #[serde(default = "default_false")]
    pub rw: bool,
}

#[derive(Deserialize)]
pub struct DBusProxyConfig {
    pub own: Vec<String>,
    pub talk: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: {} (profile) (executable) [args...]", args[0]);
        return;
    }

    let profile_path = &args[1];
    let executable = &args[2];
    let args = &args[3..];

    let profile: Config =
        toml::from_str(fs::read_to_string(profile_path).unwrap().as_str()).unwrap();

    let mut cmd = Command::new("bwrap");

    if !profile.share_ipc {
        cmd.arg("--unshare-ipc");
    }
    if !profile.share_pid {
        cmd.arg("--unshare-pid");
    }
    if !profile.share_user {
        cmd.arg("--unshare-user");
    }
    if !profile.share_net {
        cmd.arg("--unshare-net");
    }
    if !profile.share_cgroup {
        cmd.arg("--unshare-cgroup");
    }
    if !profile.share_uts {
        cmd.arg("--unshare-uts");
    }
    if !profile.share_env {
        cmd.arg("--clearenv");
    }
    if !profile.keep_alive {
        cmd.arg("--die-with-parent");
    }

    if profile.share_wayland {
        let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
        let wl_display = env::var("WAYLAND_DISPLAY").unwrap();
        let wl_socket = format!("{}/{}", &xdg_runtime_dir, &wl_display);
        cmd.arg("--ro-bind")
            .arg(&wl_socket)
            .arg(&wl_socket)
            .env("WAYLAND_DISPLAY", &wl_display)
            .env("XDG_RUNTIME_DIR", &xdg_runtime_dir);
    }

    if profile.share_x11 {
        cmd.arg("--ro-bind")
            .arg("/tmp/.X11-unix")
            .arg("/tmp/.X11-unix");

        let x11_auth = env::var("XAUTHORITY");
        match x11_auth {
            Err(_) => {}
            Ok(x11_auth) => {
                cmd.arg("--ro-bind")
                    .arg(&x11_auth)
                    .arg(&x11_auth)
                    .env("XAUTHORITY", x11_auth);
            }
        }
    }

    match profile.procfs {
        None => {}
        Some(path) => {
            cmd.arg("--proc").arg(path);
        }
    }

    match profile.tmpfs {
        None => {}
        Some(paths) => {
            for path in paths {
                cmd.arg("--tmpfs").arg(path);
            }
        }
    }

    match profile.uid {
        None => {}
        Some(uid) => {
            cmd.arg("--uid").arg(uid.to_string());
        }
    }

    match profile.gid {
        None => {}
        Some(gid) => {
            cmd.arg("--gid").arg(gid.to_string());
        }
    }

    match profile.hostname {
        None => {}
        Some(hostname) => {
            cmd.arg("--hostname").arg(hostname);
        }
    }

    match profile.env {
        None => {}
        Some(map) => {
            cmd.envs(map);
        }
    }

    match profile.bind {
        None => {}
        Some(bind_configs) => {
            for bind in bind_configs {
                match bind.rw {
                    true => match bind.no_fail {
                        true => {
                            cmd.arg("--bind-try").arg(bind.src).arg(bind.dest);
                        }
                        false => {
                            cmd.arg("--bind").arg(bind.src).arg(bind.dest);
                        }
                    },
                    false => match bind.no_fail {
                        true => {
                            cmd.arg("--ro-bind-try").arg(bind.src).arg(bind.dest);
                        }
                        false => {
                            cmd.arg("--ro-bind").arg(bind.src).arg(bind.dest);
                        }
                    },
                }
            }
        }
    }

    match profile.dev_bind {
        None => {}
        Some(dev_bind_configs) => {
            for bind in dev_bind_configs {
                match bind.no_fail {
                    true => {
                        cmd.arg("--dev-bind-try").arg(bind.src).arg(bind.dest);
                    }
                    false => {
                        cmd.arg("--dev-bind").arg(bind.src).arg(bind.dest);
                    }
                }
            }
        }
    }

    match profile.unset {
        None => {}
        Some(keys) => {
            for key in keys {
                cmd.arg("--unsetenv").arg(key);
            }
        }
    }

    match profile.symlink {
        None => {}
        Some(symlinks) => {
            for (src, dst) in symlinks {
                cmd.arg("--symlink").arg(src).arg(dst);
            }
        }
    }

    match profile.dbus_proxy {
        None => {
            cmd.arg(executable)
                .args(args)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
        Some(config) => {
            let pid = process::id();

            let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
            let dbus_socket = env::var("DBUS_SESSION_BUS_ADDRESS").unwrap();
            let dbus_proxy_socket = format!("{}/dbus-proxy-{}", xdg_runtime_dir, pid);

            let wrapper_dbus_socket = format!("{}/bus", xdg_runtime_dir);

            let mut dbus_cmd = Command::new("xdg-dbus-proxy");
            dbus_cmd
                .arg(&dbus_socket)
                .arg(&dbus_proxy_socket)
                .arg("--filter");
            for bus in config.own {
                dbus_cmd.arg(format!("--own={}", bus));
            }
            for bus in config.talk {
                dbus_cmd.arg(format!("--talk={}", bus));
            }

            let mut dbus_proc = dbus_cmd.spawn().unwrap();

            loop {
                match dbus_proc.try_wait() {
                    Ok(None) => {
                        if Path::new(&dbus_proxy_socket).exists() {
                            break;
                        }
                        sleep(Duration::from_millis(100));
                    }
                    Ok(Some(status)) => {
                        eprintln!(
                            "dbus-proxy exited too early: exit status code {}",
                            status.code().unwrap()
                        );
                        return;
                    }
                    Err(err) => {
                        eprintln!("launching dbus-proxy failed: {}", err);
                        return;
                    }
                }
            }

            cmd.arg("--bind")
                .arg(&dbus_proxy_socket)
                .arg(&wrapper_dbus_socket)
                .env(
                    "DBUS_SESSION_BUS_ADDRESS",
                    format!("unix:path={}", &wrapper_dbus_socket),
                )
                .arg(executable)
                .args(args)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            dbus_proc.kill().unwrap();

            match remove_file(&dbus_proxy_socket) {
                _ => {}
            }
        }
    }
}
