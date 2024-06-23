use std::io;

use anyhow::{Context, Result};
use crossterm::{terminal::disable_raw_mode, terminal::enable_raw_mode};

pub fn enable_raw_input() {
    enable_raw_mode().expect("Failed to enable raw mode");
}

pub fn disable_raw_input() {
    disable_raw_mode().expect("Failed to disable raw mode");
}

pub fn input_commit_message() -> Result<String> {
    disable_raw_input();
    println!("Please input commit message:"); //TODO:

    let mut commit_message = String::new();
    io::stdin()
        .read_line(&mut commit_message)
        .context("Failed to read line")?;
    Ok(commit_message)
}

pub fn input_branch_name() -> Result<String> {
    disable_raw_input();
    println!("Please input branch name:"); // TODO:

    let mut commit_message = String::new();
    io::stdin()
        .read_line(&mut commit_message)
        .context("Failed to read line")?;

    Ok(commit_message)
}

pub fn input_remote_name() -> Result<String> {
    disable_raw_input();
    println!("Please input the remote name"); // TODO:
    let mut remote = String::new();
    io::stdin()
        .read_line(&mut remote)
        .context("Failed to read line")?;
    Ok(remote)
}

pub fn input_remote_url(remote: String) -> Result<String> {
    disable_raw_input();
    println!("Please input the url of {}.", remote);
    let mut url = String::new();
    io::stdin()
        .read_line(&mut url)
        .context("Failed to read line")?;
    Ok(url)
}
