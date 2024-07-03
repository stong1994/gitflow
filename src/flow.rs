use std::process;

use crate::{
    commands::{ai_generate_commit, exec_commit},
    git::{self, check_in_git_repo, fetch},
    input,
    options::{OptionItem, Options},
    output::{output_error, output_notice},
    status::{GitRemoteBranch, GitStatus},
};
use anyhow::Result;

pub fn run() -> Result<()> {
    if !check_in_git_repo()? {
        uninitialized()?;
    }
    let remote_info = get_upstream()?;
    loop {
        output_notice("Checking git status...")?;

        let status = GitStatus::of(remote_info.clone()).unwrap();
        match status {
            GitStatus::Clean => clean()?,
            GitStatus::Unstaged => unstaged()?,
            GitStatus::PartiallyStaged => partially_staged()?,
            GitStatus::FullyStaged => fully_staged()?,
            GitStatus::PartiallyCommited => partially_committed()?,
            GitStatus::MessPartiallyCommited => mess_partially_committed()?,
            GitStatus::MessFullyCommited => mess_fully_committed()?,
            GitStatus::FullyCommited => fully_committed()?,
            GitStatus::Conflicted => conflicted()?,
        }
    }
}

fn confirm_commit(commit_command: String) -> Result<()> {
    Options {
        prompt: "Confirm the commit message.",
        options: vec![
            OptionItem {
                key: 'Y',
                desc: "Yes, execute it!!!".to_string(),
                action: Box::new(|| exec_commit(&commit_command.clone())),
            },
            OptionItem {
                key: 'R',
                desc: "Regenerate commit message.".to_string(),
                action: Box::new(commit),
            },
        ],
    }
    .execute()
}

fn commit() -> Result<()> {
    Options {
        prompt: "Choose a way to commit.",
        options: vec![
            OptionItem {
                key: 'A',
                desc: "AI generate commit message.".to_string(),
                action: Box::new(|| ai_generate_commit().and_then(confirm_commit)),
            },
            OptionItem {
                key: 'I',
                desc: "Input commit message.".to_string(),
                action: Box::new(|| {
                    input::read_line("Inpput commit command").and_then(confirm_commit)
                }),
            },
        ],
    }
    .execute()
}

fn unstaged() -> Result<()> {
    Options {
        prompt: "Files unstaged.",
        options: vec![
            OptionItem {
                key: 'A',
                desc: "Add all files.".to_string(),
                action: Box::new(add),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout a branch.".to_string(),
                action: Box::new(checkout_branch),
            },
            OptionItem {
                key: 'B',
                desc: "Create a branch.".to_string(),
                action: Box::new(create_branch),
            },
        ],
    }
    .execute()
}

fn uninitialized() -> Result<()> {
    Options {
        prompt: "Not in a git repository, do you wanna initialize git repo?",
        options: vec![OptionItem {
            key: 'Y',
            desc: "Yes, initialize.".to_string(),
            action: Box::new(git::init),
        }],
    }
    .execute()
}

fn clean() -> Result<()> {
    Options {
        prompt: "Nothing to commit, working tree clean. Maybe you wanna:",
        options: vec![
            OptionItem {
                key: 'M',
                desc: "Merge another branch".to_string(),
                action: Box::new(merge),
            },
            OptionItem {
                key: 'C',
                desc: "Checkout a branch.".to_string(),
                action: Box::new(checkout_branch),
            },
            OptionItem {
                key: 'B',
                desc: "Create a branch.".to_string(),
                action: Box::new(create_branch),
            },
        ],
    }
    .execute()
}

fn merge() -> Result<()> {
    Options {
        prompt: "Merge local branch or remote branch",
        options: vec![
            OptionItem {
                key: 'L',
                desc: "Merge local branch".to_string(),
                action: Box::new(merge_local_branch),
            },
            OptionItem {
                key: 'R',
                desc: "Merge remote branch".to_string(),
                action: Box::new(merge_remote_branch),
            },
        ],
    }
    .execute()
}

fn pull() -> Result<()> {
    let upstream = git::get_upstream()?;
    if let Some(upstream) = upstream {
        Options {
            prompt: &format!(
                "There is an upstream: {}, do you wanna pull from it?",
                upstream
            ),
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yes, pull from upstream branch".to_string(),
                action: Box::new(|| git::pull(None)),
            }],
        }
        .execute()
    } else {
        select_pull()
    }
}

fn select_pull() -> Result<()> {
    let remotes = git::get_remote_names()?;
    if remotes.is_empty() {
        output_notice("No remote found, need add one first")
    } else if remotes.len() == 1 {
        Options {
            prompt: &format!("Only one remote found: {}", remotes[0]),
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yea, use this".to_string(),
                action: Box::new(|| {
                    let branch =
                        git::get_branches(remotes.first().cloned()).and_then(choose_branch)?;
                    git::pull(Some((&remotes[0], &branch)))
                }),
            }],
        }
        .execute()
    } else {
        choose_remote(remotes).and_then(|remote| {
            let branch = git::get_branches(Some(remote.clone())).and_then(choose_branch)?;
            git::pull(Some((&remote, &branch)))
        })
    }
}

fn merge_local_branch() -> Result<()> {
    git::get_branches(None).and_then(|branches| {
        Options {
            prompt: "Please choose a branch.",
            options: branches
                .into_iter()
                .enumerate()
                .map(|(idx, branch_name)| {
                    let branch_name = branch_name.clone();
                    OptionItem {
                        key: std::char::from_u32(idx as u32).unwrap(),
                        desc: branch_name.clone(),
                        action: Box::new(move || git::merge(&branch_name)),
                    }
                })
                .collect(),
        }
        .execute()
    })
}

fn merge_remote_branch() -> Result<()> {
    select_remote_branch()
        .and_then(|(remote, branch)| git::merge(&format!("{}/{}", remote, branch)))
}

fn add() -> Result<()> {
    Options {
        prompt: "Confirm to add all?",
        options: vec![OptionItem {
            key: 'Y',
            desc: "Yes, add all!".to_string(),
            action: Box::new(git::add_all),
        }],
    }
    .execute()
}

fn checkout_branch() -> Result<()> {
    git::get_branches(None)
        .and_then(choose_branch)
        .and_then(|branch| git::checkout(&branch))
}

fn choose_branch(branches: Vec<String>) -> Result<String> {
    if branches.len() == 1 {
        Options {
            prompt: &format!("Only one branch found: {}", branches[0]),
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yes, use this branch.".to_string(),
                action: Box::new(|| Ok(branches[0].clone())),
            }],
        }
        .execute()
    } else {
        Options {
            prompt: "Please choose a branch.",
            options: branches
                .into_iter()
                .enumerate()
                .map(|(idx, branch_name)| {
                    let branch_name = branch_name.clone();
                    OptionItem {
                        key: index_to_char(idx),
                        desc: branch_name.clone(),
                        action: Box::new(move || Ok(branch_name.clone())),
                    }
                })
                .collect(),
        }
        .execute()
    }
}

fn choose_remote(remotes: Vec<String>) -> Result<String> {
    Options {
        prompt: "Select a remote.",
        options: remotes
            .into_iter()
            .enumerate()
            .map(|(idx, remote_name)| OptionItem {
                key: index_to_char(idx),
                desc: remote_name.clone(),
                action: Box::new(move || Ok(remote_name.clone())),
            })
            .collect(),
    }
    .execute()
}
fn create_branch() -> Result<()> {
    input::read_line("Please input the branch name:")
        .and_then(|branch_name| git::create_checkout(&branch_name))
}

fn get_upstream() -> Result<Option<GitRemoteBranch>> {
    let upstream = git::get_upstream()?;
    if let Some(upstream) = upstream {
        Options {
            prompt: &format!("There is an upstream: {}, do you wanna use it?", upstream),
            options: vec![
                OptionItem {
                    key: 'Y',
                    desc: "Yes, use upstream.".to_string(),
                    action: Box::new(|| {
                        let sp: Vec<_> = upstream.splitn(2, "/").collect();
                        Ok(Some(GitRemoteBranch {
                            remote: sp[0].to_string(),
                            branch: sp[1].to_string(),
                        }))
                    }),
                },
                OptionItem {
                    key: 'N',
                    desc: "No, use another remote.".to_string(),
                    action: Box::new(set_upstream),
                },
            ],
        }
        .execute()
    } else {
        set_upstream()
    }
}

fn set_upstream() -> Result<Option<GitRemoteBranch>> {
    let local_branch = git::get_current_branch()?;
    let remotes = git::get_remote_names()?;
    let remote = match remotes.len() {
        0 => add_remote().and_then(|remote| {
            fetch(&remote)?;
            Ok(remote)
        })?,
        1 => remotes[0].clone(),
        _ => choose_remote(remotes.clone())?,
    };

    let branches = git::get_branches(Some(remote.clone()))?;
    if branches.contains(&local_branch) {
        Options {
                    prompt: &format!(
                        "There is a remote branch that has the same name with local branch, do you wanna set upstream to it: {}/{}?",
                        remote.clone(), &local_branch
                    ),
                    options: vec![
                        OptionItem {
                            key: 'Y',
                            desc: "Yes, set upstream.".to_string(),
                            action: Box::new(|| git::set_upstream(&remotes[0], &local_branch).map(|()| Some(GitRemoteBranch{
                                remote: remotes[0].clone(),
                                branch: local_branch.clone(),
                            }))),
                        },

                        OptionItem {
                            key: 'N',
                            desc: "No, use another branch.".to_string(),
                            action: Box::new(||choose_branch(branches.clone()).map(|branch| Some(GitRemoteBranch{
                                remote: remote.clone(),
                                branch: branch.clone(),
                            }))),
                        },
                    ],
                }.execute()
    } else {
        Ok(None)
    }
}

fn select_remote_branch() -> Result<(String, String)> {
    let remotes = git::get_remote_names()?;
    if remotes.is_empty() {
        Options {
            prompt: "No remote found. Do you wanna add a new remote?",
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yea, add a new remote.".to_string(),
                action: Box::new(|| {
                    add_remote().and_then(|remote| {
                        git::fetch(&remote)?;
                        let branch =
                            git::get_branches(Some(remote.clone())).and_then(choose_branch)?;
                        Ok((remote, branch))
                    })
                }),
            }],
        }
        .execute()
    } else if remotes.len() == 1 {
        Options {
            prompt: &format!("Only one remote found: {}", remotes[0]),
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yea, use this".to_string(),
                action: Box::new(|| {
                    let branch =
                        git::get_branches(remotes.first().cloned()).and_then(choose_branch)?;
                    Ok((remotes[0].clone(), branch))
                }),
            }],
        }
        .execute()
    } else {
        choose_remote(remotes).and_then(|remote| {
            let branch = git::get_branches(Some(remote.clone())).and_then(choose_branch)?;
            Ok((remote, branch))
        })
    }
}

fn input_remote() -> Result<(String, String)> {
    input::read_line("Input remote name.").and_then(|name| {
        let url = input::read_line("Input remote url.")?;
        Ok((name, url))
    })
}

fn add_remote() -> Result<String> {
    input_remote().and_then(|(name, url)| {
        git::add_remote(&name, &url)?;
        Ok(name)
    })
}

fn partially_staged() -> Result<()> {
    Options {
        prompt: "Files are partially staged, you can choose:",
        options: vec![
            OptionItem {
                key: 'A',
                desc: "Add files.".to_string(),
                action: Box::new(add),
            },
            OptionItem {
                key: 'C',
                desc: "Commit files".to_string(),
                action: Box::new(commit),
            },
        ],
    }
    .execute()
}

fn fully_staged() -> Result<()> {
    Options {
        prompt: "Files are fully staged, you can choose:",
        options: vec![
            OptionItem {
                key: 'C',
                desc: "Commit files".to_string(),
                action: Box::new(commit),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout".to_string(),
                action: Box::new(checkout_branch),
            },
            OptionItem {
                key: 'M',
                desc: "Merge".to_string(),
                action: Box::new(merge),
            },
            OptionItem {
                key: 'B',
                desc: "Create a branch.".to_string(),
                action: Box::new(create_branch),
            },
        ],
    }
    .execute()
}

fn partially_committed() -> Result<()> {
    Options {
        prompt: "Files are partially committed, you can choose:",
        options: vec![
            OptionItem {
                key: 'A',
                desc: "Add files.".to_string(),
                action: Box::new(add),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout".to_string(),
                action: Box::new(checkout_branch),
            },
        ],
    }
    .execute()
}

fn mess_partially_committed() -> Result<()> {
    Options {
        prompt: "Files are partially committed, you can choose:",
        options: vec![
            OptionItem {
                key: 'C',
                desc: "Commit files.".to_string(),
                action: Box::new(commit),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout".to_string(),
                action: Box::new(checkout_branch),
            },
        ],
    }
    .execute()
}

fn mess_fully_committed() -> Result<()> {
    Options {
        prompt: "Files are partially committed and paritially added, you can choose:",
        options: vec![
            OptionItem {
                key: 'C',
                desc: "Commit files.".to_string(),
                action: Box::new(commit),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout".to_string(),
                action: Box::new(checkout_branch),
            },
        ],
    }
    .execute()
}

fn fully_committed() -> Result<()> {
    Options {
        prompt: "Files are all committed, you can chose:",
        options: vec![
            OptionItem {
                key: 'M',
                desc: "Merge.".to_string(),
                action: Box::new(merge),
            },
            OptionItem {
                key: 'L',
                desc: "Pull.".to_string(),
                action: Box::new(pull),
            },
            OptionItem {
                key: 'O',
                desc: "Checkout".to_string(),
                action: Box::new(checkout_branch),
            },
            OptionItem {
                key: 'P',
                desc: "Push to remote".to_string(),
                action: Box::new(push),
            },
        ],
    }
    .execute()
}

fn push() -> Result<()> {
    let upstream = git::get_upstream()?;
    let (remote, branch) = if let Some(upstream) = upstream {
        let branch_info: Vec<_> = upstream.splitn(2, '/').collect();
        Options {
            prompt: &format!(
                "There is an upstream: {}, do you wanna push to it?",
                upstream
            ),
            options: vec![OptionItem {
                key: 'Y',
                desc: "Yes, push to upstream branch".to_string(),
                action: Box::new(move || {
                    Ok((branch_info[0].to_string(), branch_info[1].to_string()))
                }),
            }],
        }
        .execute()
    } else {
        select_remote_branch()
    }?;
    output_notice("\nPushing, please wait a moment...\n")?;
    git::push(Some(GitRemoteBranch { remote, branch }))
}

fn conflicted() -> Result<()> {
    Options {
        prompt: "Files are identified conflicted, confirm you have resolved:",
        options: vec![
            OptionItem {
                key: 'Y',
                desc: "Yes, I have resolved.".to_string(),
                action: Box::new(git::add_all),
            },
            OptionItem {
                key: 'N',
                desc: "No, I haven't resolved.".to_string(),
                action: Box::new(|| {
                    output_error("Please resolve the conflict first.")?;
                    process::exit(1);
                }),
            },
        ],
    }
    .execute()
}

fn index_to_char(n: usize) -> char {
    std::char::from_u32((n + 1) as u32 + '0' as u32).unwrap()
}
