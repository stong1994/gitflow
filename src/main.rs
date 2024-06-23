use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::disable_raw_mode;
use crossterm::{event::poll, terminal::enable_raw_mode};
use gitflow::actions::Action;
use gitflow::choose::{Choose, OptionAction};
use gitflow::git::{
    any_changes, check_in_git_repo, get_current_branch, get_remote_names, has_file_added,
    has_uncommitted_changes,
};
use gitflow::input::disable_raw_input;
use gitflow::status::Status;
use std::io::{stdout, BufRead, BufReader};
use std::process::{Command, Output, Stdio};
use std::thread::sleep;
use std::time::Duration;
use std::{env, io, process};

fn main() {
    let mut status = Status::Begin;
    loop {
        status = status.call()
    }
}
