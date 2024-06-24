use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

use crate::output::output_success_result;

pub fn add_all() -> Result<()> {
    Command::new("git")
        .arg("add")
        .arg("--all")
        .output()
        .context("git add failed")?;
    Ok(())
}

pub fn has_file_added() -> Result<bool> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .context("git diff failed")?;

    Ok(!output.stdout.is_empty())
}

pub fn any_changes() -> Result<bool> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("git status failed")?;
    Ok(!output.stdout.is_empty())
}

pub fn check_in_git_repo() -> Result<bool> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .context("Failed to check in git repo")?;
    Ok(output.stdout == b"true\n")
}

pub fn commit_files(msg: &str) -> Result<Output> {
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .output()
        .context("Failed to commit")?;

    Ok(output)
}

pub fn get_remote_names() -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("remote")
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get remote names: {}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.split_whitespace().map(String::from).collect())
}

pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .context("Failed to execute git command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get current branch: {}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

pub fn push(remote: &str, branch: &str) -> Result<Output> {
    Command::new("git")
        .arg("push")
        .arg(remote)
        .arg(branch)
        .output()
        .context("Failed to execute git push")
}

pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--exit-code")
        .output() // can't use status() directly as it will output the git repsponse
        .context("Failed to execute git command")?;

    Ok(!output.status.success())
}

pub fn add_remote(name: String, url: String) -> Result<()> {
    let output = Command::new("git")
        .arg("remote")
        .arg("add")
        .arg(name)
        .arg(url)
        .output()
        .context("Failed to execute git push")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add remote: {}", stderr);
    }
    Ok(())
}
