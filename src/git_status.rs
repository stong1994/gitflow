use anyhow::{bail, Result};

use crate::git::{diff_remote_stat, git_status_short};

pub struct GitRemoteBranch {
    remote: String,
    branch: String,
}

pub enum GitStatus {
    // Uninitialized,
    // Initialized,
    Clean,
    Unstaged,
    PartiallyStaged,
    FullyStaged,
    PartiallyCommited,
    MessPartiallyCommited,
    MessFullyCommited,
    FullyCommited,

    Conflicted,
}

impl GitStatus {
    pub fn of(remote_branch: Option<GitRemoteBranch>) -> Result<Self> {
        let output = git_status_short()?; // Assume this function runs `git status -s` and returns the output

        if output.is_empty() {
            Ok(Self::Clean)
        } else {
            let lines: Vec<&str> = output.lines().collect();

            let mut staged = false;
            let mut unstaged = false;
            let mut need_resolve = false;
            let mut has_commit_to_push = false;

            for line in lines {
                let (first, second) = (&line[0..1], &line[1..2]);
                match first {
                    "M" | "A" | "D" | "R" | "C" | "T" => staged = true,
                    _ => {}
                }
                match second {
                    "M" | "D" | "?" | "T" | "R" | "C" | "!" => unstaged = true,
                    _ => {}
                }
                match (first, second) {
                    ("U", "U") | ("A", "A") => need_resolve = true,
                    _ => {}
                }
            }

            if let Some(remote_branch) = remote_branch {
                let diff = diff_remote_stat(remote_branch.remote, remote_branch.branch)?;
                has_commit_to_push = !diff.is_empty();
            }

            if need_resolve {
                Ok(Self::Conflicted)
            } else {
                match (has_commit_to_push, staged, unstaged) {
                    (false, false, false) => Ok(Self::Clean),
                    (false, false, true) => Ok(Self::Unstaged),
                    (false, true, false) => Ok(Self::FullyStaged),
                    (false, true, true) => Ok(Self::PartiallyStaged),
                    (true, false, false) => Ok(Self::FullyCommited),
                    (true, false, true) => Ok(Self::PartiallyCommited),
                    (true, true, false) => Ok(Self::MessFullyCommited),
                    (true, true, true) => Ok(Self::MessPartiallyCommited),
                }
            }
        }
    }
}
