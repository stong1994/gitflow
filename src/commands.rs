use std::{
    env,
    process::{self, Command, Output},
};

use anyhow::{Context, Result};

use crate::input::disable_raw_input;

pub fn check_git_installed() -> Result<bool> {
    check_command_installed("git").context("Failed to check git installed")
}

pub fn check_aicommit_installed() -> Result<bool> {
    check_command_installed("aicommit")
}

pub fn check_command_installed(command: &str) -> Result<bool> {
    let os = env::consts::OS;
    let exec = if os == "windows" { "where" } else { "which" };
    let output = Command::new(exec).arg(command).output()?;

    Ok(!output.stdout.is_empty())
}

pub fn quit() {
    disable_raw_input();
    println!("quiting...");
    process::exit(0);
}

pub fn exec(command: &str) -> Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .context("Failed to execute command")
}
