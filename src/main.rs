// Copyright 2024 Jelly Terra
// Use of this source code form is governed under the MIT license.

use std::{env, fs};
use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;

const fn default_false() -> bool {
    false
}

#[derive(Deserialize)]
pub struct Config {
    pub bind: Option<Vec<(String, String)>>,

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

    pub uid: Option<u16>,
    pub gid: Option<u16>,

    pub hostname: Option<String>,
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

    let profile: Config = toml::from_str(fs::read_to_string(profile_path).unwrap().as_str()).unwrap();

    let mut cmd = Command::new("/bin/bwrap");

    cmd.arg("--die-with-parent");
    cmd.arg("--dev").arg("/dev");
    cmd.arg("--proc").arg("/proc");
    cmd.arg("--tmpfs").arg("/tmp");

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
        Some(bind) => {
            for (src, dst) in bind {
                cmd.arg("--bind").arg(src).arg(dst);
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

    cmd.arg(executable).args(args).spawn().unwrap().wait().unwrap();
}
