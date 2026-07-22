use std::os::unix::thread;

use color_eyre::eyre::{bail, Result, WrapErr};
use futures_lite::StreamExt;
use hyprland::{
    event_listener::{
        self, AsyncEventListener, Event::ActiveWindowChanged, EventListener, EventStream,
    },
    prelude::async_closure,
    shared::{HyprDataActive, HyprDataActiveOptional},
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
    TerminalOptions,
};
use tracing::{debug, info, instrument, Level};

use tracing_subscriber::util::SubscriberInitExt;

mod app;
// mod tui;
use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish()
        .init();

    info!("Initialized tracing");

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .wrap_err("Couldn't create runtime!")?;

    let _ = tokio_runtime.block_on(main_thread());

    Ok(())
}

#[instrument]
async fn main_thread() -> Result<()> {
    let mut terminal = ratatui::init_with_options(TerminalOptions {
        viewport: ratatui::Viewport::Inline(8),
    });

    let mut app = App::new();

    let mut hyprland_event_stream = hyprland::event_listener::EventStream::new();

    tokio::spawn(async move {
        let _ = app.run(&mut terminal).await;
    });

    while let Some(Ok(event)) = hyprland_event_stream.next().await {
        match event {
            ActiveWindowChanged(Some(window_event_data)) => info!("{window_event_data:?}"),
            _ => {}
        }
    }

    Ok(())
}
