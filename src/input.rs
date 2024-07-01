use std::io;

use anyhow::{Context, Result};
use crossterm::{terminal::disable_raw_mode, terminal::enable_raw_mode};

use crate::output::output_notice;

pub fn enable_raw_input() -> Result<()> {
    enable_raw_mode().context("Failed to enable raw mode")
}

pub fn disable_raw_input() -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")
}

pub fn read_line(notice: &str) -> Result<String> {
    disable_raw_input()?;
    output_notice(notice)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read line")?;
    Ok(input)
}

pub fn read_line_simple() -> Result<String> {
    disable_raw_input()?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read line")?;
    Ok(input)
}
