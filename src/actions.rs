use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::commands::check_aicommit_installed;
use crate::git::*;
use crate::input::read_line;
use crate::output::colorful_print;
use crate::output::output_success_result;
use crate::output::Styles;
use crate::output::CODE_BG_COLOR;
use crate::output::CODE_BORDER_FG_COLOR;
use crate::output::CODE_FG_COLOR;
use crate::output::PROMPT_BG_COLOR;
use crate::output::PROMPT_NOTICE_FG_COLOR;
use crate::status::Status;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;

#[derive(Clone)]
pub enum Action {
    NotAddFiles,
    AddAll,
    ExecuteCommitCommand(String),
    GenerateCommitCommand,
    EnterBranchNameManually(String),
    AddRemote,
    UseRemote(String),
    UseBranch { remote: String, branch: String },
    EnterCommitMessageManually,
    Quit,
}

impl Action {
    pub fn call(&self) -> Result<Status> {
        match self {
            Action::NotAddFiles => Ok(Status::AddFinished),
            Action::EnterBranchNameManually(remote_name) => input_branch(remote_name.to_string()),
            Action::AddAll => {
                add_all()?;
                output_success_result("\nAll files have been added.")?;
                Ok(Status::AddFinished)
            }
            Action::GenerateCommitCommand => ai_generae_commit(),
            Action::ExecuteCommitCommand(commit_message) => {
                commit_files(commit_message)?;
                output_success_result("\nFiles have been committed.")?;
                Ok(Status::CommitFinished)
            }
            Self::AddRemote => git_add_remote(),
            Self::UseRemote(remote) => Ok(Status::RemoteSelected(remote.to_string())),
            Self::UseBranch { remote, branch } => Ok(Status::BranchSelected {
                remote_name: remote.to_string(),
                branch_name: branch.to_string(),
            }),
            Self::EnterCommitMessageManually => input_commit_manually(),
            Action::Quit => Ok(Status::QuitPressed),
        }
    }
}

fn ai_generae_commit() -> Result<Status> {
    if !check_aicommit_installed()? {
        bail!("AICommit is not installed.")
    }
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
        "\n==> generating command by aicommit, please wait a moment ....\n".to_string(),
    )?;
    let command = execute_aicommit()?;

    Ok(Status::CommitMessageGenerated(command))
}

fn execute_aicommit() -> Result<String> {
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *CODE_BORDER_FG_COLOR),
        format!("{:-^50}\n", "AICOMMIT BEGIN".to_string()),
    )?;
    let mut child = Command::new("aicommit")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to execute aicommit")?;

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();

    for word in reader.split(b' ') {
        let mut word = word.context("Failed to read word")?;
        word.push(b' ');

        let content = String::from_utf8(word).context("Failed to convert word to string")?;
        full_output.push_str(&content);

        colorful_print(Styles::with_bold(*CODE_BG_COLOR, *CODE_FG_COLOR), content)?;
        sleep(Duration::from_millis(300));
    }
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *CODE_BORDER_FG_COLOR),
        "\n".to_string(),
    )?;

    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *CODE_BORDER_FG_COLOR),
        format!("{:-^50}", "AICOMMIT END".to_string()),
    )?;
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *CODE_BORDER_FG_COLOR),
        "\n".to_string(),
    )?;
    let output = child.wait().context("Failed to wait on child")?;

    if !output.success() {
        bail!("aicommit execution failed.");
    }
    Ok(full_output)
}

fn input_commit_manually() -> Result<Status> {
    let commit_message = read_line("Please input commit message:")?;
    Ok(Status::CommitMessageGenerated(commit_message))
}

fn input_branch(remote_name: String) -> Result<Status> {
    let branch_name = read_line("Please input branch name:")?;
    Ok(Status::BranchSelected {
        branch_name,
        remote_name,
    })
}

fn git_add_remote() -> Result<Status> {
    let name = read_line("Please input the remote name")?;
    let url = read_line(&format!("Please input the url of {}.", name))?;
    add_remote(name.clone(), url)?;
    output_success_result("\nRemote has been added.")?;
    Ok(Status::RemoteSelected(name))
}
