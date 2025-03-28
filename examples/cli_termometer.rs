use std::{io::Write, net::TcpStream};

use otus_tokio_devices::temperature::Temperature;
use otus_tokio_devices::termometer::Termometer;

use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    widgets::{Block, Borders, Gauge},
};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    // Event stream.
    event_stream: EventStream,
    level: f32,

    tcp_stream: TcpStream,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: bool::default(),
            event_stream: EventStream::default(),
            level: f32::default(),
            tcp_stream: TcpStream::connect("localhost:8080").expect("Unable to connect"),
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        // Создаем элемент Gauge (шкала)
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(
                "Управление термометром. Нажмите [\"+\"/\"-\"] для изменения значений. Esc - выход",
            ))
            .label(format!("Температура: {:.2} C из {} С", self.level, Temperature::MAX_TEMPERATURE))
            .ratio(Temperature::ratio(self.level).into());

        let area = Rect {
            height: frame.area().height.saturating_sub(2),
            ..frame.area()
        };
        frame.render_widget(gauge, area)
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                match event {
                    Some(Ok(evt)) => {
                        match evt {
                            Event::Key(key)
                                if key.kind == KeyEventKind::Press
                                    => self.on_key_event(key),
                            Event::Mouse(_) => {}
                            Event::Resize(_, _) => {}
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(20)) => {
                // Sleep for a short duration to avoid busy waiting.
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Char('+')) => self.increase_level(),
            (_, KeyCode::Char('-')) => self.decrease_level(),
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn increase_level(&mut self) {
        if self.level < Temperature::MAX_TEMPERATURE {
            self.level += Temperature::GRADUATION;
        }

        self.notify()
    }

    fn decrease_level(&mut self) {
        if self.level > Temperature::MIN_TEMPERATURE {
            self.level -= Temperature::GRADUATION;
        }

        self.notify()
    }

    fn notify(&mut self) {
        let termometer = Termometer::new(Temperature::new(self.level));

        self.tcp_stream
            .write_all(termometer.to_string().as_bytes())
            .unwrap();
    }
}
