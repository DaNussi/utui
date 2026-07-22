use crossterm::event::{KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::event::Event;

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub task: JoinHandle<()>,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
}

impl Tui {
    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                              _event_tx.send(Event::Key(key)).unwrap();
                            }
                          },
                          crossterm::event::Event::Resize(width, height) => {
                            _event_tx.send(Event::Resize(width, height)).unwrap();
                          },
                          crossterm::event::Event::Mouse(mouse) => {
                            _event_tx.send(Event::Mouse(mouse)).unwrap();
                          },
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        _event_tx.send(Event::Error).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = tick_delay => {
                      _event_tx.send(Event::Tick).unwrap();
                  },
                  _ = render_delay => {
                      _event_tx.send(Event::Render).unwrap();
                  },
                }
            }
        });
    }
}
