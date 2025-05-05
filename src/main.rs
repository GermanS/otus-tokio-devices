use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::{result::Result::Ok, sync::Arc};

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use otus_tokio_devices::power::Power;
use otus_tokio_devices::socket::Socket;
use otus_tokio_devices::temperature::Temperature;
use otus_tokio_devices::termometer::Termometer;

use color_eyre::Result;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem},
};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

#[derive(Debug)]
pub enum SensorData {
    Temperature(f32),
    Power(f32),
    Unknown,
}

pub struct App {
    /// Is the application running?
    running: bool,
    // Event stream.
    event_stream: EventStream,
    messages: Vec<String>,

    termometer: Termometer,
    socket: Socket,
    rx: tokio::sync::mpsc::Receiver<Arc<SensorData>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel::<Arc<SensorData>>(32);

    let listener = TcpListener::bind("localhost:8080").await?;

    let t = Arc::new(tx);

    tokio::spawn(async move {
        loop {
            let (tcp, _) = listener.accept().await.unwrap();

            let tx_clone = Arc::clone(&t);
            tokio::spawn(async move {
                match handle_connection(tcp).await {
                    Ok(data) => {
                        if let Err(send_err) = tx_clone.send(Arc::new(data)).await {
                            eprintln!("Failed to send data through channel: {:?}", send_err);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error handling connection: {:?}", e);
                    }
                }
            });
        }
    });

    let termometer = Termometer::new(Temperature::new(0.0));
    let socket = Socket::new(Power::new(0.0));

    let terminal = ratatui::init();

    let mut app = App::new(termometer, socket, rx).await;
    let _r = app.run(terminal).await;

    Ok(())
}

async fn handle_connection(mut socket: TcpStream) -> anyhow::Result<SensorData> {
    let mut buf = [0; 128];

    let n = socket.read(&mut buf).await?;
    let recieved = String::from_utf8_lossy(&buf[..n]);

    //println!("{:?}", recieved);

    if let Ok(t) = Termometer::from_str(&recieved) {
        return Ok(SensorData::Temperature(t.temperature().get()));
    }

    if let Ok(s) = Socket::from_str(&recieved) {
        return Ok(SensorData::Power(s.power().get()));
    }

    // Отправляем ответ клиенту
    let response = format!("Ok: {}\n", recieved);
    socket.write_all(response.as_bytes()).await?;

    return Ok(SensorData::Unknown);
}

impl App {
    pub async fn new(
        t: Termometer,
        s: Socket,
        rx: tokio::sync::mpsc::Receiver<Arc<SensorData>>,
    ) -> Self {
        Self {
            running: true,
            event_stream: EventStream::default(),
            messages: vec!["40 градусов".to_string(), "50 ВТ".to_string()],
            termometer: t,
            socket: s,
            rx: rx,
        }
    }

    pub async fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.is_running() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }

        ratatui::restore();
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>

    fn draw(&mut self, f: &mut Frame) {
        if let Ok(data) = &self.rx.try_recv() {
            self.process_sensor_data(data);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3), // Первая шкала
                    Constraint::Length(3), // Вторая шкала
                    Constraint::Min(5),    // Список сообщений
                ]
                .as_ref(),
            )
            .split(f.area());

        // Отображение первой шкалы
        let gauge1 = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Термометер"))
            .label(format!(
                "Температура: {:.2} C из {} С",
                self.termometer.temperature().get(),
                Temperature::MAX_TEMPERATURE
            ))
            .ratio(Temperature::ratio(self.termometer.temperature().get()).into());
        f.render_widget(gauge1, chunks[0]);

        // Отображение второй шкалы
        let gauge2 = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Розетка"))
            .label(format!(
                "Мощность {:.1} W из {} W",
                self.socket.power().get(),
                Power::MAX_POWER
            ))
            .ratio(Power::ratio(self.socket.power().get()).into());
        f.render_widget(gauge2, chunks[1]);

        // Отображение списка сообщений
        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .map(|msg| ListItem::new(msg.as_str()))
            .collect();
        let messages_list =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Сообщения"));
        //            .start_corner(Corner::BottomLeft);
        f.render_widget(messages_list, chunks[2]);
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
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    pub fn process_sensor_data(&mut self, data: &SensorData) {
        match *data {
            SensorData::Temperature(temp) => {
                self.termometer.temperature_mut().set(temp); // Устанавливаем температуру в Termometer
                self.messages.push(format!("Temperature set to {}", temp));
            }
            SensorData::Power(power) => {
                self.socket.power_mut().set(power); // Устанавливаем мощность в Socket
                self.messages.push(format!("Power set to {}", power));
            }
            SensorData::Unknown => {
                self.messages.push("Unknown data received.".to_string());
            }
        }
    }
}
