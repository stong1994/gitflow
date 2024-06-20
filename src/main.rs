use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::disable_raw_mode;
use crossterm::{event::poll, terminal::enable_raw_mode};
use std::process::{Command, Output};
use std::{env, io, process};

fn main() {
    check_git_installed();
    check_in_git_repo();

    if !has_file_added() {
        add_files();
    }
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

fn add_files() {
    if !any_changes() {
        report_error("There is nothing need to push.");
    }

    println!("==> There are files ready to be added.");
    println!("Please choose an option:\n\t - [Y]: Add all files\n\t- [Q]: Quit");
    enable_raw_input();

    loop {
        if let Ok(Event::Key(event)) = read() {
            match event.code {
                KeyCode::Char('y') => {
                    git_add(true);
                    break;
                }
                KeyCode::Char('q') => quit(),
                _ => {
                    report_ok("Invalid input. Please press 'y' to add all files or 'q' to quit.");
                }
            }
        }
    }
}

static COMMIT_PROMPT: &str = "==> There are uncommitted changes. Please choose an option:\n  - [Y]: Use AICommit to commit the files.\n  - [M]: Enter commit message manually.\n  - [Q]: Quit.\n";

fn commit() {
    println!("{}", COMMIT_PROMPT);
    enable_raw_input();

    loop {
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
                _ => {
                    println!("Invalid input. Please press 'y' to commit changes or 'q' to quit.",);
                }
            }
        }
    }
}
fn push() {
    let remotes = get_remote_names();
    if remotes.is_empty() {
        report_error("No remote repository found.");
    } else if remotes.len() == 1 {
        git_push(&remotes[0]);
    } else {
        loop {
            println!("Please choose a remote to push to:");
            for (i, remote) in remotes.iter().enumerate() {
                println!("{}: {}", i + 1, remote);
            }
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            match input.trim().parse::<usize>() {
                Ok(n) if n > 0 && n <= remotes.len() => {
                    git_push(&remotes[n - 1]);
                    break;
                }
                _ => {
                    println!("Invalid input. Please try again.");
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
    println!("{}", msg);
    process::exit(1);
}

fn quit() {
    println!("quiting...");
    process::exit(0);
}

fn report_ok(msg: &str) {
    println!("{}", msg);
    process::exit(0);
}

fn git_add(all: bool) {
    let command = &mut Command::new("git");
    let mut c = command.arg("add");
    if all {
        c = c.arg("--all")
    }
    c.output().expect("git add failed");
    println!("All files have been added.\n");
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
    let output = Command::new("aicommit")
        .output()
        .expect("Failed to execute aicommit");
    let command = String::from_utf8_lossy(&output.stdout);

    println!(
    "==> AICommit generated the following command:\n\n{}\n\nPlease choose an option:\n  - [Y]: Execute the command\n  - [R]: Regenerate command\n  - [M]: Enter commit message manually\n  - [Q]: Quit",
    command
);
    enable_raw_input();
    loop {
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
                    KeyCode::Char('q') => report_ok("Quiting..."),
                    _ => {
                        println!(
                            "Invalid input. Please press 'y' to commit changes or 'q' to quit.",
                        );
                    }
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

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.split_whitespace().map(String::from).collect()
    } else {
        eprintln!("Failed to get remote names.");
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error:\n{}", stderr);
        vec![]
    }
}

fn git_push(remote: &str) {
    let output = Command::new("git")
        .arg("push")
        .arg(remote)
        .output()
        .expect("Failed to execute git push");
    if output.status.success() {
        println!("Pushed to {} successfully.", remote);
    } else {
        eprintln!("Failed to push to {}.", remote);
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error:\n{}", stderr);
    }
}
fn execute_commit_command(command: &str) {
    println!("Executing: \n\t {}", command);
    let output = execute_command(command);

    if output.status.success() {
        println!("Command executed successfully.");
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output:\n{}", stdout);
    } else {
        eprintln!("Command execution failed.");
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error:\n{}", stderr);
    }
}

fn execute_command(command: &str) -> Output {
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
