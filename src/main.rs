use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::disable_raw_mode;
use crossterm::{event::poll, terminal::enable_raw_mode};
use std::process::{Command, Output};
use std::{env, io, process};

fn main() {
    check_git_installed();
    check_in_git_repo();

    if !has_added() {
        add();
    }
    if has_uncommitted_changes() {
        commit();
    }
}

fn has_added() -> bool {
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .expect("git diff failed");

    !output.stdout.is_empty()
}

fn add() {
    let output_status = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("git status failed");
    if output_status.stdout.is_empty() {
        report_error("There is nothing need to push.");
    }
    println!("There are files ready to be added.");
    println!("Type y to add all files or type q to quit:");
    enable_raw_input();
    loop {
        if poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        add_files(true);
                        break;
                    }
                    KeyCode::Char('q') => {
                        println!("Operation cancelled.");
                        process::exit(0);
                    }
                    _ => {
                        report_ok(
                            "Invalid input. Please press 'y' to add all files or 'q' to quit.",
                        );
                    }
                }
            }
        }
    }
}

static COMMIT_PROMPT: &str = r#"There are uncommitted changes. Type:
  - y: use aicommit to commit the files.
  - n: put commit message manually.
  - q: quit.
"#;

fn commit() {
    println!("{}", COMMIT_PROMPT);
    enable_raw_input();
    loop {
        if poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        aicommit();
                        break;
                    }
                    KeyCode::Char('n') => {
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

fn report_ok(msg: &str) {
    println!("{}", msg);
    process::exit(0);
}

fn add_files(all: bool) {
    let command = &mut Command::new("git");
    let mut c = command.arg("add");
    if all {
        c = c.arg("--all")
    }
    c.output().expect("git add failed");
    println!("All files have been added.");
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
        r#"command is :
    ```
    {}
    ```
    You can type: 
    - y : execute .
    - r : re-generate command.
    - m : manually input commit message.
    - q : quit.
    "#,
        command
    );

    enable_raw_input();
    loop {
        if poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        execute_command(&command);
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
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn has_uncommitted_changes() -> bool {
    let status = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--exit-code")
        .status()
        .expect("Failed to execute git command");

    !status.success()
}

fn execute_command(command: &str) {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute command");
}
fn enable_raw_input() {
    enable_raw_mode().expect("Failed to enable raw mode");
}

fn disable_raw_input() {
    disable_raw_mode().expect("Failed to disable raw mode");
}
