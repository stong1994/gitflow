use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

use crate::output::command_output;

pub fn init() -> Result<()> {
    let output = Command::new("git")
        .arg("init")
        .output()
        .context("git init failed")?;
    command_output(Some("git init"), output)?;
    Ok(())
}

pub fn add_all() -> Result<()> {
    let output = Command::new("git")
        .arg("add")
        .arg("--all")
        .output()
        .context("git add failed")?;
    command_output(Some("git add --all"), output)?;
    Ok(())
}

pub fn has_file_added() -> Result<bool> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .context("git diff failed")?;

    command_output(Some("git diff --cached"), output.clone())?;
    Ok(!output.stdout.is_empty())
}

pub fn any_changes() -> Result<bool> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .context("git status failed")?;
    command_output(Some("git status --porcelain"), output.clone())?;
    Ok(!output.stdout.is_empty())
}

pub fn git_status_short() -> Result<String> {
    let output = Command::new("git")
        .arg("status")
        .arg("-s")
        .output()
        .context("git status failed")?;
    command_output(Some("git status -s"), output.clone())?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn check_in_git_repo() -> Result<bool> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .context("Failed to check in git repo")?;
    command_output(Some("git rev-parse --is-inside-work-tree"), output.clone())?;
    Ok(output.stdout == b"true\n")
}

pub fn get_remote_names() -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("remote")
        .output()
        .context("Failed to execute git command")?;

    command_output(Some("git remote"), output.clone())?;
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
    command_output(Some("git rev-parse --abbrev-ref HEAD"), output.clone())?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get current branch: {}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim().to_string())
}

pub fn get_branches(remote_name: Option<String>) -> Result<Vec<String>> {
    match remote_name {
        Some(remote_name) => {
            let output = Command::new("git")
                .arg("branch")
                .arg("-r")
                .arg("--list")
                .arg("--format=%(refname:short)")
                .output()
                .context("Failed to execute git command")?;
            command_output(
                Some("git branch -r --list --format=%(refname:short)"),
                output.clone(),
            )?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("Failed to get branches: {}", stderr);
            }
            let branches = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| line.starts_with(&remote_name))
                .filter_map(|line| line.split('/').nth(1))
                .map(|line| line.to_string())
                .collect();
            Ok(branches)
        }
        None => {
            let output = Command::new("git")
                .arg("branch")
                .arg("--list")
                .arg("--format=%(refname:short)")
                .output()
                .context("Failed to execute git command")?;
            command_output(
                Some("git branch --list --format=%(refname:short)"),
                output.clone(),
            )?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("Failed to get branches: {}", stderr);
            }
            let branches = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| line.to_string())
                .collect();
            Ok(branches)
        }
    }
}

pub fn fetch(remote: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("fetch")
        .arg(remote)
        .output()
        .context("Failed to execute git fetch")?;

    command_output(Some(&format!("git fetch {}", remote)), output.clone())?;
    if !output.status.success() {
        bail!("Failed to checkout branch");
    }
    Ok(())
}

pub fn merge(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("merge")
        .arg("--no-edit")
        .arg(branch)
        .output()
        .context("Failed to execute git merge")?;
    command_output(
        Some(&format!("git merge --no-edit {}", branch)),
        output.clone(),
    )?;
    if !output.status.success() {
        bail!("Failed to merge branch");
    }
    Ok(())
}

pub fn push(remote: &str, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("push")
        .arg(remote)
        .arg(branch)
        .output()
        .context("Failed to execute git push")?;
    command_output(
        Some(&format!("git push {} {}", remote, branch)),
        output.clone(),
    )?;
    if !output.status.success() {
        bail!("Failed to checkout branch");
    }
    Ok(())
}

pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--exit-code")
        .output() // can't use status() directly as it will output the git repsponse
        .context("Failed to execute git command")?;
    command_output(Some("git diff --cached --exit-code"), output.clone())?;

    Ok(!output.status.success())
}

pub fn checkout(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("checkout")
        .arg(branch)
        .output() // can't use status() directly as it will output the git repsponse
        .context("Failed to execute git command")?;

    command_output(Some(&format!("git checkout {}", branch)), output.clone())?;
    if !output.status.success() {
        bail!("Failed to checkout branch");
    }
    Ok(())
}

pub fn create_checkout(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(branch)
        .output() // can't use status() directly as it will output the git repsponse
        .context("Failed to execute git command")?;

    command_output(Some(&format!("git checkout -b {}", branch)), output.clone())?;
    if !output.status.success() {
        bail!("Failed to checkout branch");
    }
    Ok(())
}

pub fn add_remote(name: &str, url: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("remote")
        .arg("add")
        .arg(name)
        .arg(url)
        .output()
        .context("Failed to execute git push")?;
    command_output(
        Some(&format!("git remote add {} {}", name, url)),
        output.clone(),
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to add remote: {}", stderr);
    }
    Ok(())
}

pub fn diff_remote_stat(remote: String, branch: String) -> Result<String> {
    let output = Command::new("git")
        .arg("diff")
        .arg("--stat")
        .arg(format!("{}/{}", remote, branch))
        .output()
        .context("Failed to execute git diff")?;
    command_output(
        Some(&format!("git diff --stat {}/{}", remote, branch)),
        output.clone(),
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to execute git diff: {}", stderr);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
