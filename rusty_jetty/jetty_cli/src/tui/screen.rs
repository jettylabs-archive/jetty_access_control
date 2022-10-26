use std::io::stdout;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

pub(crate) struct AltScreenContext;

impl AltScreenContext {
    pub(crate) fn start() -> Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }

    pub(crate) fn end(&self) {
        execute!(stdout(), LeaveAlternateScreen).unwrap()
    }
}
