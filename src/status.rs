use anyhow::{bail, Context, Result};
use std::process;

use crate::git::commit_files;
use crate::output::{Styles, OUTTER_OUTPUT_FG_COLOR, PROMPT_BG_COLOR, PROMPT_NOTICE_FG_COLOR};
use crate::{
    actions::Action,
    choose::{Choose, OptionAction},
    commands::{check_git_installed, quit},
    git::{
        any_changes, check_in_git_repo, get_current_branch, get_remote_names, has_file_added,
        has_uncommitted_changes, push,
    },
    output::{colorful_print, output_success_result, report_error, report_success},
};

#[derive(Clone)]
pub enum Status {
    Begin,
    PreCheckPast,
    FileHasAdded,
    AddFinished,
    CommitMessageGenerated(String),
    CommitMessageCofnfirmed(String),
    CommitFinished,

    RemoteSelected(String),
    BranchSelected {
        remote_name: String,
        branch_name: String,
    },
    Success,
    Failure(String),
    QuitPressed,
}

impl Status {
    pub fn call(&self) -> Status {
        match self {
            Status::Begin => match pre_check() {
                Ok(true) => Status::PreCheckPast,
                Ok(false) => Status::Failure("Pre check not past.".to_string()),
                Err(err) => Status::Failure(err.to_string()),
            },

            Status::PreCheckPast => match has_file_added() {
                Ok(true) => Status::FileHasAdded,
                Ok(false) => match try_add_all() {
                    Ok(status) => status,
                    Err(err) => Status::Failure(err.to_string()),
                },
                Err(err) => Status::Failure(err.to_string()),
            },
            Status::FileHasAdded => file_added(),
            Self::AddFinished => match commit() {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },
            Self::CommitMessageGenerated(command) => match confirm_message(command.clone()) {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },
            Self::CommitMessageCofnfirmed(message) => match commit_message(message.clone()) {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },
            Self::CommitFinished => match select_remote() {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },
            Self::RemoteSelected(remote) => match select_branch(remote.to_string()) {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },

            Status::BranchSelected {
                remote_name,
                branch_name,
            } => match git_push(remote_name, branch_name) {
                Ok(status) => status,
                Err(err) => Status::Failure(err.to_string()),
            },
            Status::Success => {
                report_success("Success!");
                process::exit(0)
            }
            Status::Failure(msg) => {
                report_error(msg);
                process::exit(1)
            }
            Self::QuitPressed => {
                quit();
                process::exit(0)
            }
        }
    }
}

fn pre_check() -> Result<bool> {
    match check_git_installed() {
        Ok(true) => (),
        Ok(false) => {
            bail!("Git is not installed. Please install git first.")
        }
        Err(err) => {
            return Err(err);
        }
    }

    match check_in_git_repo()? {
        true => Ok(true),
        false => {
            bail!("Not in a git repository.")
        }
    }
}

fn git_push(remote: &str, branch: &str) -> Result<Status> {
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
        "Pushing code, please wait a moment...\n".to_string(),
    )?;

    let output = push(remote, branch)?;
    if output.status.success() {
        output_success_result(&format!("Pushed to {} successfully.\n", remote))?;
        Ok(Status::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(Status::Failure(format!("Failed to push: {}", stderr)))
    }
}

fn file_added() -> Status {
    match Choose::new("There are files have been added.")
        .add_option(
            "Don't add other files",
            OptionAction::new('Y', Action::NotAddFiles),
        )
        .add_option("Add all files", OptionAction::new('A', Action::AddAll)) // TODO:
        // check if there are filed need to add
        .add_quit()
        .prompt_choose()
    {
        Ok(status) => status,
        Err(err) => Status::Failure(err.to_string()),
    }
}

fn commit() -> Result<Status> {
    match has_uncommitted_changes()? {
        true => Choose::new("There are uncommitted changes.")
            .add_option(
                "Commit with AICommit",
                OptionAction::new('Y', Action::GenerateCommitCommand),
            )
            .add_option(
                "Enter commit message manually",
                OptionAction::new('M', Action::EnterCommitMessageManually),
            )
            .add_quit()
            .prompt_choose(),
        false => Ok(Status::CommitFinished),
    }
}

fn select_remote() -> Result<Status> {
    let remotes = get_remote_names()?;
    match remotes.len() {
        0 => Choose::new("There is no remote repository.")
            .add_option("Add Remote.", OptionAction::new('Y', Action::AddRemote))
            .add_quit()
            .prompt_choose(),
        1 => Choose::new("Confirm Remote. There is only one remote repository.")
            .add_option(
                format!("Push to the remote: {}", remotes[0]).as_str(),
                OptionAction::new('Y', Action::UseRemote(remotes[0].clone())),
            )
            .add_quit()
            .prompt_choose(),
        _ => {
            let mut choose = Choose::new("Confirm Remote. There are multiple remote.");
            for (i, remote) in remotes.iter().enumerate() {
                let key = to_char(i)?;
                choose.add_option(
                    remote,
                    OptionAction::new(key, Action::UseRemote(remote.to_string())),
                );
            }
            choose.add_quit();
            choose.prompt_choose()
        }
    }
}
fn select_branch(remote: String) -> Result<Status> {
    let local_branch = get_current_branch()?;

    Choose::new("Confirm Branch")
        .add_option(
            format!("Push to the remote branch: {}", local_branch.clone()).as_str(),
            OptionAction::new(
                'Y',
                Action::UseBranch {
                    remote: remote.clone(),
                    branch: local_branch,
                },
            ),
        )
        .add_option(
            "Input branch manually.",
            OptionAction::new('M', Action::EnterBranchNameManually(remote.clone())),
        )
        .add_quit()
        .prompt_choose()
}
fn to_char(n: usize) -> Result<char> {
    let c = std::char::from_u32(n as u32).context("Failed to conver to char")?;
    Ok(c)
}

fn try_add_all() -> Result<Status> {
    if !any_changes()? {
        return Ok(Status::FileHasAdded);
    }

    Choose::new("There are files ready to be added.")
        .add_option("Add all files", OptionAction::new('Y', Action::AddAll))
        .add_quit()
        .prompt_choose()
}

fn commit_message(command: String) -> Result<Status> {
    let output = commit_files(&command)?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
            "\nCommand executed successfully. Output:\n".to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *OUTTER_OUTPUT_FG_COLOR),
            stdout.to_string(),
        )?;
        Ok(Status::CommitFinished)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Command execution failed: {}.", stderr);
    }
}

fn confirm_message(command: String) -> Result<Status> {
    Choose::new("Confirm Commit command")
        .add_option(
            &command,
            OptionAction::new('Y', Action::ExecuteCommitCommand(command.clone())),
        )
        .add_option(
            "Regenerate commit command",
            OptionAction::new('R', Action::GenerateCommitCommand),
        )
        .add_option(
            "Enter commit command manually",
            OptionAction::new('M', Action::EnterCommitMessageManually),
        )
        .add_quit()
        .prompt_choose()
}
