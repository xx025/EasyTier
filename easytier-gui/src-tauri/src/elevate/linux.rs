/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Luis Liu. All rights reserved.
 *  Licensed under the MIT License. See License in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use super::Command;
use anyhow::{Result, anyhow};
use std::env;
use std::ffi::OsStr;
use std::process::{Command as StdCommand, Output};

/// The implementation of state check and elevated executing varies on each platform
impl Command {
    /// Check the state the current program running
    ///
    /// Return `true` if the program is running as root, otherwise false
    pub fn is_elevated() -> bool {
        let uid = unsafe { libc::getuid() };
        uid == 0
    }

    /// Prompting the user with a graphical OS dialog for the root password,
    /// excuting the command with escalated privileges, and return the output
    pub fn output(&self) -> Result<Output> {
        let mut command = StdCommand::new("pkexec");
        let inherited_env = [
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "XAUTHORITY",
            "HOME",
            "XDG_RUNTIME_DIR",
            "DBUS_SESSION_BUS_ADDRESS",
        ]
        .into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key, value)))
        .collect::<Vec<_>>();

        command.arg("--disable-internal-agent");
        if !inherited_env.is_empty() {
            command.arg("env");
            for (key, value) in inherited_env {
                command.arg(format!("{key}={value}"));
            }
        } else if self.cmd.get_envs().any(|(_, v)| v.is_some()) {
            command.arg("env");
        }
        for (k, v) in self.cmd.get_envs() {
            if let Some(value) = v {
                command.arg(format!(
                    "{}={}",
                    k.to_str().ok_or(anyhow!("invalid key"))?,
                    value.to_str().ok_or(anyhow!("invalid value"))?
                ));
            }
        }

        command.arg(self.cmd.get_program());
        let args: Vec<&OsStr> = self.cmd.get_args().collect();
        if !args.is_empty() {
            command.args(args);
        }

        let output = command.output()?;
        Ok(output)
    }
}
