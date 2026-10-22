use std::{
    io::{self},
    rc::Rc,
    time::Duration,
};

use chrono::Utc;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind};
use futures::StreamExt;
use hyprland::shared::{HyprData, HyprDataActive};
use hyprland::{
    data::{Workspace, Workspaces},
    dispatch::DispatchType::Custom,
};
use nf_icons::nf;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    prelude::Widget,
    style::Stylize,
    widgets::Block,
    DefaultTerminal, Frame, Terminal,
};
use ratatui_interact::traits::ClickRegionRegistry;
use stylix::palette::StylixPalette;
use tracing::{error, info, warn};

use crate::app::{AppError, AppLike};

enum ClickableElement {
    Power,
    Wifi,
    Bluetooth,
    Volume,
    Workspace(i32),
    Clock,
}

pub struct App {
    frame: u128,
    pallet: StylixPalette,
    workspace: Workspace,
    workspaces: Workspaces,
    exit: bool,
    registry: ClickRegionRegistry<Rc<ClickableElement>>,
}

impl App {
    pub fn new(pallet: StylixPalette) -> Result<Self, AppError> {
        Ok(App {
            frame: 0,
            pallet,
            workspace: Workspace::get_active().unwrap(),
            workspaces: Workspaces::get().unwrap(),
            exit: false,
            registry: ClickRegionRegistry::new(),
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        Terminal::clear(terminal)?;

        while !self.exit {
            if let Ok(workspace) = Workspace::get_active() {
                self.workspace = workspace;
            } else {
                error!("Failed to get active workspace!");
            }

            if let Ok(workspaces) = Workspaces::get() {
                self.workspaces = workspaces;
            } else {
                error!("Failed to get workspaces!");
            }

            if let Ok(_) = terminal.draw(|frame| self.draw(frame)) {
                self.frame += 1;
            } else {
                error!("Failed to draw frame!");
            }

            if let Ok(_) = self.handle_events().await {
                // events handled successfully
            } else {
                error!("Failed to handle events!");
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.render(frame.area(), frame.buffer_mut());
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        let timeout = tokio::time::sleep(Duration::from_millis(10));
        let mut crossterm_event_stream = crossterm::event::EventStream::new();
        let mut hyprland_event_stream = hyprland::event_listener::EventStream::new();

        tokio::select! {
            _ = timeout => {},
            crossterm_event = crossterm_event_stream.next() => {
                if let Some(Ok(event)) = crossterm_event {
                    match event {
                        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                            self.handle_key_event(key_event);
                        }
                        Event::Mouse(mouse_event) => {
                            match mouse_event.kind {
                                MouseEventKind::Down(_) => {
                                    if let Some(element) = self.registry.handle_click(mouse_event.column, mouse_event.row) {
                                        match element.as_ref() {
                                            ClickableElement::Power => {
                                                info!("Power button clicked!");
                                            }
                                            ClickableElement::Wifi => {
                                                info!("Wifi button clicked!");
                                            }
                                            ClickableElement::Bluetooth => {
                                                info!("Bluetooth button clicked!");
                                            }
                                            ClickableElement::Volume => {
                                                info!("Volume button clicked!");
                                            }
                                            ClickableElement::Workspace(workspace_id) => {
                                                let dispatch_command = format!("hl.dsp.focus({{ workspace = '{}' }})", workspace_id);
                                                let result = hyprland::dispatch::Dispatch::call(Custom(&dispatch_command, ""));
                                                if let Err(e) = result {
                                                    error!("{:?}", e)
                                                }
                                            }
                                            ClickableElement::Clock => {
                                                info!("Clock clicked!");
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            },
            hyprland_event = hyprland_event_stream.next() => {
                let _ = hyprland_event;
            },
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default();
        let block_area = block.inner(area);

        let [left_area, center_area, right_area] = Layout::horizontal([
            Constraint::Length(self.left_width()),
            Constraint::Length(self.center_width()),
            Constraint::Length(self.right_width()),
        ])
        .horizontal_margin(1)
        .flex(Flex::SpaceBetween)
        .areas(block_area);

        self.registry.clear();
        self.registry
            .register(left_area, Rc::new(ClickableElement::Power));

        block.render(area, buf);
        self.left_render(left_area, buf);
        self.center_render(center_area, buf);
        self.right_render(right_area, buf);
    }

    fn center_width(&self) -> u16 {
        self.workspaces.iter().len() as u16 * 3 - 2
    }

    fn center_render(&mut self, area: Rect, buf: &mut Buffer) {
        let mut workspace_refs: Vec<&Workspace> = self.workspaces.iter().collect();
        workspace_refs.sort_by_key(|workspace| workspace.id);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![Constraint::Length(1); workspace_refs.len()])
            .spacing(2)
            .flex(Flex::SpaceBetween)
            .split(area);

        for (index, workspace) in workspace_refs.iter().enumerate() {
            let workspace_area = if let Some(area) = layout.get(index) {
                *area
            } else {
                warn!(
                    "Failed to get workspace area for workspace {}!",
                    workspace.id
                );
                continue;
            };

            let workspace_label = workspace.id.to_string();
            if workspace.id == self.workspace.id {
                workspace_label
                    .black()
                    .on_yellow()
                    .bold()
                    .render(workspace_area, buf);
            } else {
                workspace_label.yellow().render(workspace_area, buf);
            }

            self.registry.register(
                workspace_area,
                Rc::new(ClickableElement::Workspace(workspace.id)),
            );
        }
    }

    fn left_width(&self) -> u16 {
        4 * 2 - 1
    }

    fn left_render(&mut self, area: Rect, buf: &mut Buffer) {
        let [power_area, wifi_area, bluetooth_area, volume_area] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .spacing(1)
        .flex(Flex::SpaceBetween)
        .areas(area);

        nf!("nf-md-power_standby").yellow().render(power_area, buf);
        nf!("nf-md-wifi").yellow().render(wifi_area, buf);
        nf!("nf-md-bluetooth").yellow().render(bluetooth_area, buf);
        nf!("nf-md-volume_high").yellow().render(volume_area, buf);

        self.registry
            .register(power_area, Rc::new(ClickableElement::Power));
        self.registry
            .register(wifi_area, Rc::new(ClickableElement::Wifi));
        self.registry
            .register(bluetooth_area, Rc::new(ClickableElement::Bluetooth));
        self.registry
            .register(volume_area, Rc::new(ClickableElement::Volume));
    }

    fn clock_text(&self) -> String {
        Utc::now().format("%H:%M:%S").to_string()
    }

    fn right_width(&self) -> u16 {
        self.clock_text().len() as u16
    }

    fn right_render(&mut self, area: Rect, buf: &mut Buffer) {
        let [clock_area] = Layout::horizontal([Constraint::Length(self.clock_text().len() as u16)])
            .flex(Flex::SpaceBetween)
            .areas(area);

        self.clock_text().yellow().render(clock_area, buf);
        self.registry
            .register(clock_area, Rc::new(ClickableElement::Clock));
    }
}

impl AppLike for App {
    fn new(pallet: StylixPalette) -> Result<Self, AppError> {
        Self::new(pallet)
    }

    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.run(terminal).await
    }
}
