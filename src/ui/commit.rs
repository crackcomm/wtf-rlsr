use console::Term;
use dialoguer::Input;

use super::default_theme;

/// Prompts for a multi-line message.
pub fn prompt(prompt: &str) -> std::io::Result<Option<Vec<String>>> {
    println!("{}:", prompt);
    let term = Term::stdout();
    let mut lines = Vec::new();
    let mut last_empty = false;
    loop {
        let line = term.read_line()?;
        if line.len() == 0 {
            if last_empty {
                break;
            }
            last_empty = true;
        }
        lines.push(line.trim().to_owned());
    }
    term.clear_last_lines(lines.len() + 2)?;
    if lines.len() <= 1 && last_empty {
        Ok(None)
    } else {
        Ok(Some(lines))
    }
}

/// Prompts for a commit header.
pub fn prompt_header(prompt: &'static str) -> std::io::Result<String> {
    Input::with_theme(&default_theme())
        .with_prompt(prompt)
        .allow_empty(false)
        .validate_with(move |text: &str| {
            if text.trim().len() < 3 {
                Err(format!("{} is too short.", prompt))
            } else if text.trim().len() > 22 {
                Err(format!("{} is too long.", prompt))
            } else {
                Ok(())
            }
        })
        .interact()
}

/// Logs a commit message.
pub fn log_message(commit: &Vec<String>) {
    for line in commit {
        trace!("Commit message: {}", line);
    }
}
