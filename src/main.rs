mod app;
mod background;
mod bar;

use std::io::stdout;

use clap::{Parser, ValueEnum};
use stylix::Stylix;

use crate::app::AppLike;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum AppVariant {
    Bar,
    Background,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum, default_value_t = AppVariant::Bar)]
    variant: AppVariant,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let palette = Stylix::load().unwrap();

    let viewport = match args.variant {
        AppVariant::Bar => ratatui::Viewport::Inline(1),
        AppVariant::Background => ratatui::Viewport::Fullscreen,
    };

    let mut terminal = ratatui::init_with_options(ratatui::TerminalOptions { viewport });

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture)?;

    match args.variant {
        AppVariant::Bar => bar::App::new(palette)?.run(&mut terminal).await?,
        AppVariant::Background => background::App::new(palette)?.run(&mut terminal).await?,
    }

    crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;
    ratatui::restore();

    Ok(())
}
