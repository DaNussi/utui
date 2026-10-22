use std::{io, time::Duration};

use chrono::Local;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use rand::RngExt;
use ratatui::{
    buffer::Buffer,
    prelude::{Alignment, Rect},
    style::{Color as RatatuiColor, Style},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    DefaultTerminal, Frame, Terminal,
};
use stylix::palette::StylixPalette;

use crate::app::{AppError, AppLike};

pub struct App {
    palette: StylixPalette,
    exit: bool,
    values: Vec<(u16, u16, char, stylix::color::Color)>,
}

impl AppLike for App {
    fn new(palette: StylixPalette) -> Result<Self, AppError> {
        Ok(Self {
            palette,
            exit: false,
            values: Vec::new(),
        })
    }

    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        Terminal::clear(terminal)?;
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if self.handle_events()? {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }

        Ok(())
    }
}

impl App {
    fn handle_events(&mut self) -> io::Result<bool> {
        if crossterm::event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.exit = true;
                    return Ok(true);
                }
            }
        }

        Ok(self.exit)
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buffer = frame.buffer_mut();

        let padding = 1;

        let rows: Vec<u16> = (padding..area.height - padding).step_by(2).collect();
        let cols: Vec<u16> = (padding..area.width - padding).step_by(2).collect();

        let init = rows.len() * cols.len() != self.values.len();

        if init {
            self.values.clear();
            for row in rows.iter() {
                for col in cols.iter() {
                    let x = *col.min(&area.width.saturating_sub(1));
                    let y = *row.min(&area.height.saturating_sub(1));
                    let color = self.random_color();
                    let glyph = self.random_char();
                    self.values.push((x, y, glyph, color));
                }
            }
        }

        for _ in 0..10 {
            let index = rand::random_range(0..self.values.len());
            let mut value = self.values[index];
            value.3 = self.random_color();
            self.values[index] = value;
        }

        for _ in 0..5 {
            let index = rand::random_range(0..self.values.len());
            let mut value = self.values[index];
            value.2 = self.random_char();
            self.values[index] = value;
        }

        for (x, y, glyph, color) in self.values.iter() {
            if let Some(cell) = buffer.cell_mut((*x, *y)) {
                cell.set_char(*glyph);
                cell.set_style(Style::default().fg(RatatuiColor::Rgb(color.r, color.g, color.b)));
            }
        }

        // let clock_text = Local::now().format("%H:%M:%S").to_string();
        // let clock_width = clock_text.chars().count() as u16 + 4;
        // let clock_height = 3;
        // let clock_rect = Rect {
        //     x: area.width.saturating_sub(clock_width) / 2,
        //     y: area.height.saturating_sub(clock_height) / 2,
        //     width: clock_width.min(area.width),
        //     height: clock_height.min(area.height),
        // };

        // let clock_widget = Paragraph::new(clock_text)
        //     .alignment(Alignment::Center)
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .border_style(Style::new())
        //             .style(Style::default().fg(RatatuiColor::Rgb(
        //                 self.palette.base05.r,
        //                 self.palette.base05.g,
        //                 self.palette.base05.b,
        //             ))),
        //     );

        // frame.render_widget(Clear, clock_rect);
        // frame.render_widget(clock_widget, clock_rect);
    }

    fn random_char(&mut self) -> char {
        let glyphs: Vec<char> =
            "0123456789ABCDEF!@#$%^&*()_+-=[]{}|;:,.<>/?~abcdefghijklmnopqrstuvwxyz"
                .chars()
                .collect();
        let index = rand::rng().random_range(0..glyphs.len());
        glyphs[index]
    }

    fn random_color(&mut self) -> stylix::color::Color {
        let colors = [
            self.palette.base08,
            self.palette.base09,
            self.palette.base0A,
            self.palette.base0B,
            self.palette.base0C,
            self.palette.base0D,
            self.palette.base0E,
            self.palette.base0F,
        ];
        let index = rand::rng().random_range(0..colors.len());
        colors[index]
    }
}
