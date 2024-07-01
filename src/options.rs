use std::{collections::HashMap, process};

use anyhow::Result;
use crossterm::event::{read, Event, KeyCode};

use crate::{commands::quit, input::enable_raw_input, output::*};

pub struct OptionItem<'a, T> {
    pub key: char,
    pub desc: String,
    pub action: Box<dyn Fn() -> Result<T> + 'a>,
}
pub struct Options<'a, T> {
    pub prompt: &'a str,
    pub options: Vec<OptionItem<'a, T>>,
}

impl<'a, T> Options<'a, T> {
    pub fn execute(&self) -> Result<T> {
        self.print_prompt()?;
        self.print_options()?;

        let ops_map: HashMap<_, _> = self
            .options
            .iter()
            .map(|option| (option.key.to_ascii_lowercase(), &option.action))
            .collect();

        loop {
            enable_raw_input().unwrap();
            if let Ok(Event::Key(event)) = read() {
                if let KeyCode::Char(c) = event.code {
                    let c = c.to_ascii_lowercase();
                    if c == 'q' {
                        quit()?;
                        process::exit(0);
                    }
                    let option = ops_map.get(&c);
                    match option {
                        Some(option) => {
                            return option();
                        }
                        None => {
                            output_notice("Invalid option, please try again\n.")?;
                        }
                    }
                }
            }
        }
    }

    fn print_prompt(&self) -> Result<()> {
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_FG_COLOR),
            format!("\n==> {}\n", self.prompt),
        )
    }

    fn print_options(&self) -> Result<()> {
        self.options
            .iter()
            .try_for_each(|option| Self::print_option(option))?;
        Self::print_quit()?;
        Ok(())
    }

    fn print_option(option: &OptionItem<'a, T>) -> Result<()> {
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            "\n\t- [".to_string(),
        )?;
        colorful_print(
            Styles::with_bold(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_KEY_FG_COLOR),
            option.key.to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            "]: ".to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            format!("{}\n", option.desc).to_string(),
        )?;
        Ok(())
    }

    fn print_quit() -> Result<()> {
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            "\n\t- [".to_string(),
        )?;
        colorful_print(
            Styles::with_bold(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_QUITKEY_FG_COLOR),
            "Q".to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            "]: ".to_string(),
        )?;
        colorful_print(
            Styles::new(*PROMPT_BG_COLOR, *PROMPT_OPTIONI_DESC_FG_COLOR),
            format!("{}\n", "Quit").to_string(),
        )?;
        Ok(())
    }
}
