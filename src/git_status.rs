use anyhow::{bail, Result};

use crate::git::{check_in_git_repo, diff_remote_stat, git_status_short};

struct GitRemoteBranch {
    remote: String,
    branch: String,
}

enum GitStatus {
    Uninitialized,
    // Initialized,
    Clean,
    Unstaged,
    PartiallyStaged,
    FullyStaged,
    CleanParitiallyCommited,
    DirtyPartiallyCommited,
    DirtyFullCommited,
    FullyCommited,

    Conflicted,
}

impl GitStatus {
    pub fn of(remote_branch: Option<GitRemoteBranch>) -> Result<Self> {
        if !check_in_git_repo()? {
            return Ok(Self::Uninitialized);
        }
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
                let status_code = &line[0..2];
                match status_code {
                    "A " | "M " | "D " | "R " | "C " | "T " => staged = true,
                    " M" | " D" | "??" | " T" | " R" | " C" | "!!" | "DD" | "AU" | "UD" | "UA"
                    | "DU" => unstaged = true,
                    "UU" | "AA" => need_resolve = true,
                    _ => bail!("Unrecognized git status: {}", status_code),
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
                    (true, false, true) => Ok(Self::CleanParitiallyCommited),
                    (true, true, false) => Ok(Self::DirtyFullCommited),
                    (true, true, true) => Ok(Self::DirtyPartiallyCommited),
                }
            }
        }
    }
}
