// standard library
use std::io::{self, Write};

// internal crates
use crate::errors::{DialoguerErr, InstallerErr};
use config_agent::trace;

// external crates
use dialoguer::console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm, Password};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

pub enum Colors {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

pub fn clear_terminal() {
    // ANSI escape code to clear the screen
    print!("{}[2J", 27_u8 as char);
    // ANSI escape code to move the cursor to the top left
    print!("{}[H", 27_u8 as char);
    io::stdout().flush().unwrap_or_default(); // Ensure the clear command is executed
}

pub fn input_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: style("".to_string()),
        ..Default::default()
    }
}

pub fn confirm_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: style("".to_string()),
        ..Default::default()
    }
}

pub fn select_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: style("".to_string()),
        prompt_style: Style::new().for_stderr(),
        ..Default::default()
    }
}

pub fn wait(prompt: &str) -> Result<(), InstallerErr> {
    let _: String = Password::with_theme(&input_theme())
        .with_prompt(prompt)
        .allow_empty_password(true)
        .interact()
        .map_err(|e| {
            InstallerErr::DialoguerErr(DialoguerErr {
                source: e,
                trace: trace!(),
            })
        })?;
    Ok(())
}

pub fn confirm(prompt: &str) -> Result<bool, InstallerErr> {
    let continue_: bool = Confirm::with_theme(&confirm_theme())
        .with_prompt(prompt)
        .interact()
        .map_err(|e| {
            InstallerErr::DialoguerErr(DialoguerErr {
                source: e,
                trace: trace!(),
            })
        })?;
    Ok(continue_)
}

pub fn color_text(text: &str, color: Colors) -> String {
    let color_code = match color {
        Colors::Red => "31",
        Colors::Green => "32",
        Colors::Yellow => "33",
        Colors::Blue => "34",
        Colors::Magenta => "35",
        Colors::Cyan => "36",
        Colors::White => "37",
    };
    format!("\x1b[{}m{}\x1b[0m", color_code, text)
}

pub fn bold_text(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}

pub fn color_text_rgb(text: &str, r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

pub fn print_title(title: &str) {
    print_boxed_title(title);
}

fn print_boxed_title(title: &str) {
    let width = title.len() + 4;
    let border = "═".repeat(width);

    println!("╔{}╗", border);
    println!("║  {}  ║", title);
    println!("╚{}╝", border);
}

pub fn print_err_msg(err: Option<String>) {
    println!("Our apologies, but an error occurred during your installation :/ Please send us an email at ben@miruml.com for immediate support.\n");

    if let Some(e) = err {
        println!("Error: {}\n", e);
    }
}

pub fn format_url(url: &str, display_text: &str) -> String {
    format!(
        "\x1b]8;{}\x1b\\{}\x1b]8;\x1b\\",
        url,
        bold_text(&color_text(display_text, Colors::Green))
    )
}
