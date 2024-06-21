use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::disable_raw_mode;
use crossterm::{event::poll, terminal::enable_raw_mode};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Output, Stdio};
use std::thread::sleep;
use std::time::Duration;
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
        return;
    }

    disable_raw_input();
    println!("==> There are files ready to be added.");
    println!("Please choose an option:\n\t- [Y]: Add all files\n\t- [Q]: Quit");

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
                    report_ok("Invalid input. Please try again.");
                }
            }
        }
    }
}

static COMMIT_PROMPT: &str = "==> There are uncommitted changes. Please choose an option:\n\t- [Y]: Use AICommit to commit the files.\n\t- [M]: Enter commit message manually.\n\t- [Q]: Quit.";

fn commit() {
    disable_raw_input();
    println!("{}", COMMIT_PROMPT);

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
                _ => {
                    disable_raw_input();
                    println!("Invalid input. Please try again.",);
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
        disable_raw_input();
        println!("==> There is only one remote repository.\nnPlease choose an option:\n\t- [Y]: Push to the remote: {}.\n\t- [Q]: Quit.", remotes[0]);
        loop {
            enable_raw_input();
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        let branch = get_branch_name();
                        git_push(&remotes[0], &branch);
                        break;
                    }
                    KeyCode::Char('q') => quit(),
                    _ => {
                        println!("Invalid input. Please try again.");
                    }
                }
            }
        }
        let branch = get_branch_name();
        git_push(&remotes[0], &branch);
    } else {
        loop {
            disable_raw_input();
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
                    let branch = get_branch_name();
                    git_push(&remotes[n - 1], &branch);
                    break;
                }
                _ => {
                    println!("Invalid input. Please try again.");
                }
            }
        }
    }
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
        return remote;
    } else if remotes.len() == 1 {
        disable_raw_input();
        println!("==> There is only one remote repository.\nnPlease choose an option:\n\t- [Y]: Push to the remote: {}.\n\t- [Q]: Quit.", remotes[0]);
        loop {
            enable_raw_input();
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char('y') => {
                        let branch = get_branch_name();
                        return branch;
                    }
                    KeyCode::Char('q') => quit(),
                    _ => {
                        println!("Invalid input. Please try again.");
                    }
                }
            }
        }
    } else {
        disable_raw_input();
        println!("Please choose a remote to push to:");
        for (i, remote) in remotes.iter().enumerate() {
            println!("{}: {}", i + 1, remote);
        }

        loop {
            enable_raw_input();
            if let Event::Key(event) = read().unwrap() {
                match event.code {
                    KeyCode::Char(c) => {
                        let i = (c as u8 - b'0') as usize;
                        if i > 0 && i <= remotes.len() {
                            return remotes[i - 1].clone();
                        } else {
                            disable_raw_input();
                            println!("Invalid input. Please try again.");
                        }
                    }
                    _ => {
                        disable_raw_input();
                        println!("Invalid input. Please try again.");
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
    disable_raw_input();
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

static COMMIT_EXEC_PROMPT: &str = "==> AICommit generated command:\nPlease choose an option:\n\t- [Y]: Execute the command\n\t- [R]: Regenerate command\n\t- [M]: Enter commit message manually\n\t- [Q]: Quit";
fn aicommit() {
    check_aicommit_installed();
    disable_raw_input(); // must disable raw mode, otherwise follow output will mess up the terminal
    println!("\n==> generating command by aicommit:\nwaiting....");
    let command = execute_aicommit();
    println!("{}", COMMIT_EXEC_PROMPT);
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
                    KeyCode::Char('q') => report_ok("Quiting..."),
                    _ => {
                        disable_raw_input();
                        println!("Invalid input. Please try again.",);
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

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to get remote names: {}", stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.split_whitespace().map(String::from).collect()
}

fn get_branch_name() -> String {
    let local_branch = get_current_branch();

    disable_raw_input();
    println!("==> There are unpushed commits. Please choose an branch:\n\t- [Y]: {}.\n\t- [M]: Input branch manually.\n\t- [Q]: Quit.\n", local_branch);

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
                _ => {
                    println!("Invalid input. Please try again.");
                    return get_branch_name();
                }
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
    let output = Command::new("git")
        .arg("push")
        .arg(remote)
        .arg(branch)
        .output()
        .expect("Failed to execute git push");
    if output.status.success() {
        disable_raw_input();
        println!("Pushed to {} successfully.", remote);
    } else {
        disable_raw_input();
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Failed to push: {}", stderr));
    }
}

fn execute_aicommit() -> String {
    disable_raw_input();
    println!("{:-^30}", "AICOMMIT BEGIN");
    let mut child = Command::new("aicommit")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute aicommit");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    let mut console = io::stdout();
    let mut full_output = String::new();

    for word in reader.split(b' ') {
        let mut word = word.expect("Failed to read word");
        word.push(b' ');
        console
            .write_all(&word)
            .expect("Failed to write to console");
        console.flush().expect("Failed to flush console");
        full_output.push_str(&String::from_utf8(word).expect("Failed to convert word to string"));
        sleep(Duration::from_millis(300));
    }
    console
        .write_all(b"\n")
        .expect("Failed to write to console");
    console.flush().expect("Failed to flush console");

    println!("{:-^30}", "AICOMMIT END");
    let output = child.wait().expect("Failed to wait on child");

    if !output.success() {
        report_error("aicommit execution failed.");
    }
    full_output
}

fn execute_commit_command(command: &str) {
    disable_raw_input();
    println!("Executing: \n\t {}", command);
    let output = execute_command(command);

    if output.status.success() {
        println!("Command executed successfully.");
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output:\n{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        report_error(&format!("Command execution failed: {}.", stderr));
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
