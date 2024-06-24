use std::io::stdout;
use std::process::Output;

use crate::commands::exec;
use crate::input::disable_raw_input;
use anyhow::Result;
use crossterm::style::{
    Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::{execute, ExecutableCommand};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROMPT_BG_COLOR: Color = hex_to_color("#222831");
    pub static ref CODE_BG_COLOR: Color = hex_to_color("#F9E8C9");
    pub static ref CODE_BORDER_FG_COLOR: Color = hex_to_color("#898121");
    pub static ref CODE_FG_COLOR: Color = hex_to_color("#0A6847");
    pub static ref COMMAND_FG_COLOR: Color = hex_to_color("#ACD793");
    pub static ref PROMPT_FG_COLOR: Color = hex_to_color("#ECB159");
    pub static ref PROMPT_OPTIONI_KEY_FG_COLOR: Color = hex_to_color("#CBFFA9");
    pub static ref PROMPT_OPTIONI_QUITKEY_FG_COLOR: Color = hex_to_color("#FF6868");
    pub static ref PROMPT_OPTIONI_DESC_FG_COLOR: Color = hex_to_color("#5BBCFF");
    pub static ref PROMPT_ERR_FG_COLOR: Color = hex_to_color("#FF0000");
    pub static ref PROMPT_SUCCESS_FG_COLOR: Color = hex_to_color("#CDE990");
    pub static ref PROMPT_NOTICE_FG_COLOR: Color = hex_to_color("#C780FA");
    pub static ref OUTTER_OUTPUT_FG_COLOR: Color = hex_to_color("#5356FF");
}

pub fn output_success_result(result: &str) -> Result<()> {
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_SUCCESS_FG_COLOR),
        result.to_string(),
    )
}

pub struct Styles {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub bold: Option<bool>,
}

impl Styles {
    pub fn new(bg: Color, fg: Color) -> Styles {
        Styles {
            bg: Some(bg),
            fg: Some(fg),
            bold: None,
        }
    }

    pub fn with_bold(bg: Color, fg: Color) -> Styles {
        Styles {
            bg: Some(bg),
            fg: Some(fg),
            bold: Some(true),
        }
    }
}

pub fn colorful_print(colors: Styles, content: String) -> Result<()> {
    disable_raw_input();

    let mut o = stdout();
    if let Some(bg) = colors.bg {
        o.execute(SetBackgroundColor(bg))?;
    }
    if let Some(fg) = colors.fg {
        o.execute(SetBackgroundColor(fg))?;
    }
    if colors.bold.is_some() {
        o.execute(SetAttribute(Attribute::Bold))?;
    }
    o.execute(Print(content))?;
    o.execute(ResetColor)?;
    Ok(())
}

// TODO: refacor
pub fn colorful_print_with_bold(bg: Color, fg: Color, content: String) {
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

pub fn output_invalid_type() -> Result<()> {
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_ERR_FG_COLOR),
        "Invalid input. Please try again.\n".to_string(),
    )
}

pub fn report_error(msg: &str) {
    disable_raw_input();
    println!("{}", msg);
}

pub fn report_success(msg: &str) {
    disable_raw_input();
    println!("{}", msg);
}

fn execute_command(command: &str) -> Result<Output> {
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *PROMPT_NOTICE_FG_COLOR),
        "\nExecuting:".to_string(),
    )?;
    colorful_print(
        Styles::new(*PROMPT_BG_COLOR, *COMMAND_FG_COLOR),
        format!("\t{}", command).to_string(),
    )?;
    exec(command)
}
