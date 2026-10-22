use std::{
    error::Error as StdError,
    fmt::{self, Display},
    io,
};

use ratatui::DefaultTerminal;
use stylix::palette::StylixPalette;

#[derive(Debug)]
pub enum AppError {
    ConfigLoadError(Box<dyn StdError + Send + Sync>),
    ConfigParseError(Box<dyn StdError + Send + Sync>),
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ConfigLoadError(err) => write!(f, "config load failed: {err}"),
            AppError::ConfigParseError(err) => write!(f, "config parse failed: {err}"),
        }
    }
}

impl StdError for AppError {}

impl From<Box<dyn StdError + Send + Sync>> for AppError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        Self::ConfigLoadError(err)
    }
}

pub trait AppLike: Sized {
    fn new(palette: StylixPalette) -> Result<Self, AppError>;
    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()>;
}
