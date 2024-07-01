use std::{
    env,
    io::{BufRead, BufReader},
    process::{Command, Output, Stdio},
    thread::sleep,
    time::Duration,
};

use anyhow::{bail, Context, Result};

use crate::{input::disable_raw_input, output::*};

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

pub fn quit() -> Result<()> {
    disable_raw_input()?;
    output_notice("quiting...")?;
    Ok(())
}

pub fn exec(command: &str) -> Result<Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .context("Failed to execute command")
}

pub fn exec_commit(command: &str) -> Result<()> {
    let output = exec(command)?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
            "\nCommand executed successfully. Output:\n".to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *OUTTER_OUTPUT_FG_COLOR),
            stdout.to_string(),
        )
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Command execution failed: {}.", stderr);
    }
}

pub fn ai_generate_commit() -> Result<String> {
    if !check_aicommit_installed()? {
        bail!("AICommit is not installed.")
    }
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
        "\n==> generating command by aicommit, please wait a moment ....\n".to_string(),
    )?;
    let command = execute_aicommit()?;

    Ok(command)
}

pub fn execute_aicommit() -> Result<String> {
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
