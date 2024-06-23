use crossterm::event::{read, Event, KeyCode};

use crate::{actions::Action, input::enable_raw_input, output::*, status::Status};
use anyhow::Result;

pub struct ChooseOption<'a> {
    action: OptionAction,
    prompt: &'a str,
}
pub struct Choose<'a> {
    prompt: &'a str,
    options: Vec<ChooseOption<'a>>,
}

impl<'a> Choose<'a> {
    pub fn new(prompt: &'a str) -> Self {
        Choose {
            prompt,
            options: Vec::new(),
        }
    }

    pub fn add_option(&mut self, prompt: &'a str, action: OptionAction) -> &mut Self {
        self.options.push(ChooseOption { action, prompt });
        self
    }
    pub fn add_quit(&mut self) -> &mut Self {
        self.options.push(ChooseOption {
            action: QUIT_ACTION,
            prompt: "Quit.",
        });
        self
    }

    fn prompt(&self) {
        let mut up = UserPrompt::new(self.prompt);
        for option in self.options.iter() {
            up.add_option(option.action.option, option.prompt);
        }
        up.print();
    }
    fn choose(&self) -> Result<Status> {
        loop {
            enable_raw_input();
            if let Ok(Event::Key(event)) = read() {
                if let KeyCode::Char(c) = event.code {
                    for option in self.options.iter() {
                        let key = option.action.option;
                        if c == key || c == key.to_ascii_lowercase() {
                            return option.action.action.call();
                        }
                    }
                    output_invalid_type();
                }
            }
        }
    }

    pub fn prompt_choose(&self) -> Result<Status> {
        self.prompt();
        self.choose()
    }
}

const QUIT_ACTION: OptionAction = OptionAction::new('Q', Action::Quit);

#[derive(Clone)]
pub struct OptionAction {
    option: char,
    action: Action,
}

impl OptionAction {
    pub const fn new(option: char, action: Action) -> Self {
        OptionAction { option, action }
    }
}

struct PromptOption<'a> {
    key: char,
    desc: &'a str,
}

struct UserPrompt<'a> {
    prompt: &'a str,
    options: Vec<PromptOption<'a>>,
}
impl<'a> UserPrompt<'a> {
    fn new(prompt: &'a str) -> Self {
        UserPrompt {
            prompt,
            options: Vec::new(),
        }
    }

    fn add_option(&mut self, key: char, desc: &'a str) -> &mut Self {
        self.options.push(PromptOption { key, desc });
        self
    }

    fn print(&self) {
        colorful_print(
            *PROMPT_BG_COLOR,
            *PROMPT_FG_COLOR,
            format!("\n==> {}\n\nPlease choose an option:\n", self.prompt),
        );

        for option in &self.options {
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                "\n\t- [".to_string(),
            );
            colorful_print_with_bold(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_KEY_FG_COLOR,
                option.key.to_string(),
            );
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                "]: ".to_string(),
            );
            colorful_print(
                *PROMPT_BG_COLOR,
                *PROMPT_OPTIONI_DESC_FG_COLOR,
                format!("{}\n", option.desc),
            );
        }
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
