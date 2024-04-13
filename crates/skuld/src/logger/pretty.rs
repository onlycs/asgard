pub enum Color {
    Red,    // error
    Yellow, // warn
    Blue,   // info
    Purple, // debug
    White,  // trace
}

pub fn colored(text: impl Into<String>, color: Color) -> String {
    // ansi coloring
    let colored_text = match color {
        Color::Red => format!("\x1b[31m{}\x1b[0m", text.into()),
        Color::Yellow => format!("\x1b[33m{}\x1b[0m", text.into()),
        Color::Blue => format!("\x1b[34m{}\x1b[0m", text.into()),
        Color::Purple => format!("\x1b[35m{}\x1b[0m", text.into()),
        Color::White => format!("\x1b[37m{}\x1b[0m", text.into()),
    };

    colored_text
}

pub fn level(level: log::Level) -> String {
    match level {
        log::Level::Error => colored("ERROR", Color::Red),
        log::Level::Warn => colored("WARN", Color::Yellow),
        log::Level::Info => colored("INFO", Color::Blue),
        log::Level::Debug => colored("DEBUG", Color::Purple),
        log::Level::Trace => colored("TRACE", Color::White),
    }
}

pub fn bold(text: impl Into<String>) -> String {
    format!("\x1b[1m{}\x1b[0m", text.into())
}

pub fn light(text: impl Into<String>) -> String {
    format!("\x1b[2m{}\x1b[0m", text.into())
}
