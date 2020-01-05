pub mod commit;
pub mod packages;
pub mod update;

use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Select};

/// Selects one choice from the list.
pub fn select_from_list<T: ToString>(prompt: &str, list: &[T]) -> std::io::Result<Option<usize>> {
    Select::with_theme(&default_theme())
        .with_prompt(prompt)
        .items(list)
        .interact_opt()
}

/// Asks for a confirmation.
pub fn confirm(text: &str) -> std::io::Result<bool> {
    Confirmation::with_theme(&default_theme())
        .with_text(text)
        .interact()
}

pub(crate) fn default_theme() -> ColorfulTheme {
    ColorfulTheme {
        values_style: Style::new().yellow().dim(),
        indicator_style: Style::new().yellow().bold(),
        yes_style: Style::new().yellow().dim(),
        no_style: Style::new().yellow().dim(),
        ..ColorfulTheme::default()
    }
}
