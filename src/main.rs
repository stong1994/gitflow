use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::disable_raw_mode;
use crossterm::{event::poll, terminal::enable_raw_mode};
use lazy_static::lazy_static;
use std::io::{stdout, BufRead, BufReader};
use std::process::{Command, Output, Stdio};
use std::thread::sleep;
use std::time::Duration;
use std::{env, io, process};

fn main() {
    check_git_installed();
    check_in_git_repo();

    add();
    if has_uncommitted_changes() {
        commit();
    }
    push();
}

fn has_file_added() -> bool {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .expect("git diff failed");

    !output.stdout.is_empty()
}

fn add() {
    if has_file_added() {
        UserPrompt::new("\n==> There are files have been added.".to_string())
            .add_option("Y".to_string(), "Don't add other files".to_string())
            .add_option("A".to_string(), "Add all files".to_string())
            .print();

        loop {
            enable_raw_input();
            if let Ok(Event::Key(event)) = read() {
                match event.code {
                    KeyCode::Char('y') => {
                        break;
                    }
                    KeyCode::Char('a') => {
                        git_add(true);
                        break;
                    }
                    KeyCode::Char('q') => quit(),
                    _ => {
                        output_invalid_type();
                    }
                }
            }
        }
        return;
    }
    if !any_changes() {
        return;
    }

    UserPrompt::new("\n==> There are files ready to be added.".to_string())
        .add_option("Y".to_string(), "Add all files".to_string())
        .print();

    loop {
        enable_raw_input();
        if let Ok(Event::Key(event)) = read() {
            match event.code {
                KeyCode::Char('y') => {
                    git_add(true);
                    break;
                }
                KeyCode::Char('q') => quit(),
                _ => {
                    output_invalid_type();
                }
            }
        }
    }
}

fn commit() {
    UserPrompt::new("\n==> There are uncommitted changes.".to_string())
        .add_option("Y".to_string(), "Commit with AICommit".to_string())
        .add_option("M".to_string(), "Enter commit message manually".to_string())
        .print();

    loop {
        enable_raw_input();
        if let Ok(Event::Key(event)) = read() {
            match event.code {
                KeyCode::Char('y') => {
                    aicommit();
                    break;
                }
                KeyCode::Char('m') => {
                    disable_raw_input();
                    println!("Please input commit message:");

                    let mut commit_message = String::new();
                    io::stdin()
                        .read_line(&mut commit_message)
                        .expect("Failed to read line");

                    commit_files(&commit_message);

                    break;
                }
                KeyCode::Char('q') => quit(),
                _ => output_invalid_type(),
            }
        }
    }
}

fn push() {
    let remote = get_remote_name();
    let branch = get_branch_name();
    git_push(&remote, &branch);
}

fn get_remote_name() -> String {
    let remotes = get_remote_names();
    if remotes.is_empty() {
        disable_raw_input();
        println!("Please input the remote name");
        let mut remote = String::new();
        io::stdin()
            .read_line(&mut remote)
            .expect("Failed to read line");
        println!("Please input the url of {}.", remote);
        let mut url = String::new();
        io::stdin()
            .read_line(&mut url)
            .expect("Failed to read line");

        git_set_remote(&remote, &url);
        remote
    } else if remotes.len() == 1 {
        UserPrompt::new("==> Confirm Remote. There is only one remote repository.".to_string())
            .add_option(
                "Y".to_string(),
                format!("Push to the remote: {}", remotes[0]),
            )
            .print();
        loop {
            enable_raw_input();
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        return remotes[0].clone();
                    }
                    KeyCode::Char('q') => quit(),
                    _ => output_invalid_type(),
                }
            }
        }
    } else {
        let mut prompt = UserPrompt::new("==> There are multiple remote:".to_string());
        remotes.iter().enumerate().for_each(|(i, remote)| {
            prompt.add_option(i.to_string(), remote.to_string());
        });

        loop {
            enable_raw_input();
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char(c) => {
                        if c == 'q' {
                            quit();
                        }
                        let i = (c as u8 - b'0') as usize;
                        if i > 0 && i <= remotes.len() {
                            return remotes[i - 1].clone();
                        } else {
                            output_invalid_type();
                        }
                    }
                    _ => output_invalid_type(),
                }
            }
        }
    }
}

fn check_git_installed() {
    if !check_command_installed("git") {
        report_error("Git is not installed. Please install git first.");
    }
}

fn check_aicommit_installed() {
    if !check_command_installed("aicommit") {
        report_error("AICommit is not installed. Please install aicommit first. see: https://github.com/stong1994/aicommit");
    }
}

fn check_command_installed(command: &str) -> bool {
    let os = env::consts::OS;
    let exec = if os == "windows" { "where" } else { "which" };
    let output = Command::new(exec)
        .arg(command)
        .output()
        .expect("failed to execute process");
    !output.stdout.is_empty()
}

fn check_in_git_repo() {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("git rev-parse failed");
    if output.stdout != b"true\n" {
        report_error("Not in a git repository.");
    }
}

fn report_error(msg: &str) {
    disable_raw_input();
    println!("{}", msg);
    process::exit(1);
}

fn quit() {
    disable_raw_input();
    println!("quiting...");
    process::exit(0);
}

fn git_add(all: bool) {
    let command = &mut Command::new("git");
    let mut c = command.arg("add");
    if all {
        c = c.arg("--all")
    }
    c.output().expect("git add failed");
    output_success_result("\nAll files have been added.");
}

fn commit_files(msg: &str) {
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(msg)
        .spawn()
        .expect("Failed to commit");
}

fn aicommit() {
    check_aicommit_installed();

    colorful_print(
        *PROMPT_BG_COLOR,
        *PROMPT_NOTICE_FG_COLOR,
        "\n==> generating command by aicommit, please wait a moment ....\n".to_string(),
    );
    let command = execute_aicommit();
    UserPrompt::new("==> AICommit generated command".to_string())
        .add_option("Y".to_string(), "Execute the command".to_string())
        .add_option("R".to_string(), "Regenerate command".to_string())
        .add_option("M".to_string(), "Enter commit message manually".to_string())
        .print();

    loop {
        enable_raw_input();
        if poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        execute_commit_command(&command);
                        break;
                    }
                    KeyCode::Char('r') => {
                        aicommit();
                        break;
                    }
                    KeyCode::Char('m') => {
                        disable_raw_input();
                        println!("Please input commit message:");

                        let mut commit_message = String::new();
                        io::stdin()
                            .read_line(&mut commit_message)
                            .expect("Failed to read line");

                        commit_files(&commit_message);

                        break;
                    }
                    KeyCode::Char('q') => quit(),
                    _ => output_invalid_type(),
                }
            }
        }
    }
}
fn any_changes() -> bool {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("git status failed");
    !output.stdout.is_empty()
}
fn has_uncommitted_changes() -> bool {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--exit-code")
        .output() // can't use status() directly as it will output the git repsponse
        .expect("Failed to execute git command");

    !output.status.success()
}

fn get_remote_names() -> Vec<String> {
    let output = Command::new("git")
        .arg("remote")
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to get remote names: {}", stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.split_whitespace().map(String::from).collect()
}

fn get_branch_name() -> String {
    let local_branch = get_current_branch();

    UserPrompt::new("==> Confirm Branch".to_string())
        .add_option(
            "Y".to_string(),
            format!("Push to the remote branch: {}", local_branch.clone()),
        )
        .add_option("M".to_string(), "Input branch manually.".to_string())
        .print();

    enable_raw_input();
    loop {
        if let Ok(Event::Key(event)) = read() {
            match event.code {
                KeyCode::Char('y') => {
                    return local_branch;
                }
                KeyCode::Char('m') => {
                    disable_raw_input();

                    let mut branch = String::new();
                    io::stdin()
                        .read_line(&mut branch)
                        .expect("Failed to read line");
                    return branch;
                }

                KeyCode::Char('q') => quit(),
                _ => output_invalid_type(),
            }
        }
    }
}
fn get_current_branch() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .expect("Failed to execute git command");
    if !output.status.success() {
        disable_raw_input();
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to get current branch: {}", stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().to_string()
}

fn git_set_remote(name: &str, url: &str) {
    disable_raw_input();
    let output = Command::new("git")
        .arg("remote")
        .arg("add")
        .arg(name)
        .arg(url)
        .output()
        .expect("Failed to execute git push");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to add remote: {}", stderr));
    }
    println!("Set remote {} successfully.", name);
}

fn git_push(remote: &str, branch: &str) {
    colorful_print(
        *PROMPT_BG_COLOR,
        *PROMPT_NOTICE_FG_COLOR,
        "Pushing code, please wait a moment...\n".to_string(),
    );
    let output = Command::new("git")
        .arg("push")
        .arg(remote)
        .arg(branch)
        .output()
        .expect("Failed to execute git push");
    if output.status.success() {
        disable_raw_input();
        output_success_result(&format!("Pushed to {} successfully.", remote))
    } else {
        disable_raw_input();
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to push: {}", stderr));
    }
}

fn execute_aicommit() -> String {
    colorful_print(
        *PROMPT_BG_COLOR,
        *CODE_BORDER_FG_COLOR,
        format!("{:-^50}\n", "AICOMMIT BEGIN".to_string()),
    );
    let mut child = Command::new("aicommit")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute aicommit");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut full_output = String::new();

    for word in reader.split(b' ') {
        let mut word = word.expect("Failed to read word");
        word.push(b' ');

        let content = String::from_utf8(word).expect("Failed to convert word to string");
        full_output.push_str(&content);
        colorful_print_with_bold(*CODE_BG_COLOR, *CODE_FG_COLOR, content);
        sleep(Duration::from_millis(300));
    }
    colorful_print(*CODE_BG_COLOR, *CODE_FG_COLOR, "\n".to_string());

    colorful_print(
        *PROMPT_BG_COLOR,
        *CODE_BORDER_FG_COLOR,
        format!("{:-^50}", "AICOMMIT END".to_string()),
    );
    colorful_print(*PROMPT_BG_COLOR, *CODE_BORDER_FG_COLOR, "\n".to_string());
    let output = child.wait().expect("Failed to wait on child");

    if !output.success() {
        report_error("aicommit execution failed.");
    }
    full_output
}

fn execute_commit_command(command: &str) {
    disable_raw_input();
    let output = execute_command(command);
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_NOTICE_FG_COLOR,
            "\nCommand executed successfully. Output:\n".to_string(),
        );
        colorful_print(
            *PROMPT_BG_COLOR,
            *OUTTER_OUTPUT_FG_COLOR,
            stdout.to_string(),
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Command execution failed: {}.", stderr));
    }
}

fn execute_command(command: &str) -> Output {
    colorful_print(
        *PROMPT_BG_COLOR,
        *PROMPT_NOTICE_FG_COLOR,
        "\nExecuting:".to_string(),
    );
    colorful_print(
        *PROMPT_BG_COLOR,
        *COMMAND_FG_COLOR,
        format!("\t{}", command).to_string(),
    );
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command")
}
fn enable_raw_input() {
    enable_raw_mode().expect("Failed to enable raw mode");
}

fn disable_raw_input() {
    disable_raw_mode().expect("Failed to disable raw mode");
}

fn output_success_result(result: &str) {
    colorful_print(
        *PROMPT_BG_COLOR,
        *PROMPT_SUCCESS_FG_COLOR,
        result.to_string(),
    );
}

struct UserPrompt {
    prompt: String,
    options: Vec<[String; 2]>,
}
lazy_static! {
    static ref PROMPT_BG_COLOR: crossterm::style::Color = hex_to_color("#222831");
    static ref CODE_BG_COLOR: crossterm::style::Color = hex_to_color("#F9E8C9");
    static ref CODE_BORDER_FG_COLOR: crossterm::style::Color = hex_to_color("#898121");
    static ref CODE_FG_COLOR: crossterm::style::Color = hex_to_color("#0A6847");
    static ref COMMAND_FG_COLOR: crossterm::style::Color = hex_to_color("#ACD793");
    static ref PROMPT_FG_COLOR: crossterm::style::Color = hex_to_color("#ECB159");
    static ref PROMPT_OPTIONI_KEY_FG_COLOR: crossterm::style::Color = hex_to_color("#CBFFA9");
    static ref PROMPT_OPTIONI_QUITKEY_FG_COLOR: crossterm::style::Color = hex_to_color("#FF6868");
    static ref PROMPT_OPTIONI_DESC_FG_COLOR: crossterm::style::Color = hex_to_color("#5BBCFF");
    static ref PROMPT_ERR_FG_COLOR: crossterm::style::Color = hex_to_color("#FF0000");
    static ref PROMPT_SUCCESS_FG_COLOR: crossterm::style::Color = hex_to_color("#CDE990");
    static ref PROMPT_NOTICE_FG_COLOR: crossterm::style::Color = hex_to_color("#C780FA");
    static ref OUTTER_OUTPUT_FG_COLOR: crossterm::style::Color = hex_to_color("#5356FF");
}
impl UserPrompt {
    fn new(prompt: String) -> Self {
        UserPrompt {
            prompt,
            options: Vec::new(),
        }
    }

    fn add_option(&mut self, key: String, desc: String) -> &mut UserPrompt {
        self.options.push([key, desc]);
        self
    }

    fn print(&self) {
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_FG_COLOR,
            format!("\n{}\n\nPlease choose an option:\n", self.prompt.clone()),
        );

        self.options.clone().into_iter().for_each(|option| {
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                "\n\t- [".to_string(),
            );
            colorful_print_with_bold(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_KEY_FG_COLOR,
                option[0].to_string(),
            );
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                "]: ".to_string(),
            );
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                option[1].clone() + "\n",
            );
        });
        // print quit option
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_OPTIONI_DESC_FG_COLOR,
            "\n\t- [".to_string(),
        );
        colorful_print_with_bold(
            *PROMPT_BG_COLOR,
            *PROMPT_OPTIONI_QUITKEY_FG_COLOR,
            "Q".to_string(),
        );
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_OPTIONI_DESC_FG_COLOR,
            "]: ".to_string(),
        );
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_OPTIONI_DESC_FG_COLOR,
            "Quit\n".to_string(),
        );
    }
}

fn output_invalid_type() {
    colorful_print(
        *PROMPT_BG_COLOR,
        *PROMPT_ERR_FG_COLOR,
        "Invalid input. Please try again.\n".to_string(),
    );
}

fn colorful_print(bg: Color, fg: Color, content: String) {
    disable_raw_input();
    execute!(
        stdout(),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(content),
        ResetColor
    )
    .expect("Failed to colorful print");
}

// TODO: refacor
fn colorful_print_with_bold(bg: Color, fg: Color, content: String) {
    disable_raw_input();
    execute!(
        stdout(),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        SetAttribute(Attribute::Bold),
        Print(content),
        ResetColor
    )
    .expect("Failed to colorful print");
}
fn hex_to_color(hex: &str) -> crossterm::style::Color {
    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();

    crossterm::style::Color::Rgb { r, g, b }
}
