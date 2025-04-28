use std::result::Result::Ok;
use std::{str::FromStr, sync::Arc};

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
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

#[derive(Debug)]
enum SensorData {
    Temperature(f32),
    Power(f32),
}

pub struct App {
    /// Is the application running?
    running: bool,
    // Event stream.
    event_stream: EventStream,
    progress1: u8,
    progress2: u8,
    messages: Vec<String>,

    termometer: Arc<Termometer>,
    socket: Arc<Socket>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<SensorData>(32);

    let termometer = Arc::new(Termometer::new(Temperature::new(0.0)));
    let socket = Arc::new(Socket::new(Power::new(0.0)));

    let terminal = ratatui::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut app = App::new(termometer, socket).await;
    app.run(terminal, rx).await;

    loop {
        if !app.is_running() {
            break;
        }

        let (tcp, _) = listener.accept().await?;

        let tx_clone = tx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(tcp, tx_clone).await {
                println!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    mut socket: TcpStream,
    tx: mpsc::Sender<SensorData>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0; 32];

    // loop {
    //     let n = match socket.read(&mut buf).await {
    //         //Ok(0) => return Ok(()),
    //         Ok(n) => n,
    //         Err(e) => return Err(e.into()),
    //     };

    //     let recieved = String::from_utf8_lossy(&buf[..n]);

    //     if let Ok(t) = Termometer::from_str(&recieved) {
    //         println!("Temperature set to {}", t);
    //         //app.termometer.temperature().set(t.temperature().get());
    //         //app.messages.push(format!("Temperature set to {}", t));
    //     }

    //     if let Ok(s) = Socket::from_str(&recieved) {
    //         println!("Power set to {}", s);
    //     }
    // }

    let n = socket.read(&mut buf).await?;
    let recieved = String::from_utf8_lossy(&buf[..n]);

    if let Ok(t) = Termometer::from_str(&recieved) {
        tx.send(SensorData::Temperature(t.temperature().get()))
            .await?;
    }

    if let Ok(s) = Socket::from_str(&recieved) {
        tx.send(SensorData::Power(s.power().get())).await?;
    }

    Ok(())
}

impl App {
    pub async fn new(t: Arc<Termometer>, s: Arc<Socket>) -> Self {
        Self {
            running: bool::default(),
            event_stream: EventStream::default(),
            progress1: 0,
            progress2: 0,
            messages: vec!["40 градусов".to_string(), "50 ВТ".to_string()],
            termometer: t,
            socket: s,
        }
    }

    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
        mut rx: mpsc::Receiver<SensorData>,
    ) -> Result<()> {
        self.running = true;
        while self.is_running() {
            if let Ok(data) = rx.try_recv() {
                println!("The data: {:?}", data);
            }

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
            .ratio(self.progress1 as f64 / 100.0);
        f.render_widget(gauge1, chunks[0]);

        // Отображение второй шкалы
        let gauge2 = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Розетка"))
            .ratio(self.progress2 as f64 / 100.0);
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
}
