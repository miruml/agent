// standard library
use std::io::{self, Write};

// external crates
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
    format!("\x1b[{color_code}m{text}\x1b[0m")
}

pub fn bold_text(text: &str) -> String {
    format!("\x1b[1m{text}\x1b[0m")
}

pub fn color_text_rgb(text: &str, r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

pub fn print_title(title: &str) {
    print_boxed_title(title);
}

fn print_boxed_title(title: &str) {
    let width = title.len() + 4;
    let border = "═".repeat(width);

    println!("╔{border}╗");
    println!("║  {title}  ║");
    println!("╚{border}╝");
}

pub fn print_err_msg(err: Option<String>) {
    println!("An error occurred during your installation. Contact us at ben@miruml.com or 832-461-3504 for immediate support.\n");

    if let Some(e) = err {
        println!("Error: {e}\n");
    }
}

pub fn format_url(url: &str, display_text: &str) -> String {
    format!(
        "\x1b]8;{}\x1b\\{}\x1b]8;\x1b\\",
        url,
        bold_text(&color_text(display_text, Colors::Green))
    )
}
