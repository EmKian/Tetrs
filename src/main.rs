use std::{
    error::Error,
    io,
    sync::{mpsc, Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use tui::{
    backend::{self, Backend, CrosstermBackend},
    buffer,
    layout::{Alignment::Center, Rect},
    style::Color,
    widgets::{Block, Borders, Widget},
    Terminal,
};

use crossterm::{
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::game::{Direction, ShiftError, TetrominosBag};

mod game;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    const PLAYFIELD_ROWS: u16 = 20;
    const PLAYFIELD_COLS: u16 = 10;
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let terminal_size = terminal.size()?;

    let mut playfield = ui::Playfield::new(
        terminal_size.width,
        terminal_size.height,
        PLAYFIELD_COLS,
        PLAYFIELD_ROWS,
        2,
        1,
    );
    let mut bag = TetrominosBag::new();
    bag.shuffle();
    let mut tetromino = bag.get();
    tetromino.spawn(&mut playfield);
    terminal.show_cursor()?;
    playfield.draw(&mut terminal);

    let (tx_input, rx_input) = mpsc::channel();
    let (tx_timer, rx_timer) = mpsc::channel();
    let accept_input = Arc::new(Mutex::new(true));
    let input_thread = input_thread(tx_input, accept_input.clone());
    let timer_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000));
            tx_timer.send(true).unwrap();
        }
    });
    loop {
        let mut went_down = false;
        let mut result = Ok(());
        playfield.draw(&mut terminal);
        if let Ok(key) = rx_input.recv_timeout(Duration::from_millis(500)) {
            result = match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') | KeyCode::Down => {
                    went_down = true;
                    tetromino.shift(&mut playfield, Direction::Down)
                }
                KeyCode::Char('l') | KeyCode::Left => {
                    tetromino.shift(&mut playfield, Direction::Left)
                }
                KeyCode::Char('h') | KeyCode::Right => {
                    tetromino.shift(&mut playfield, Direction::Right)
                }
                KeyCode::Char('r') => {
                    tetromino.rotate(&mut playfield, true);
                    Ok(())
                }
                KeyCode::Char('R') | KeyCode::Char('e') => {
                    tetromino.rotate(&mut playfield, false);
                    Ok(())
                }
                KeyCode::Char(' ') | KeyCode::Char('J') => {
                    Err(tetromino.hard_drop(&mut playfield))
                }
                _ => Ok(()),
            };
        }
        if rx_timer.recv_timeout(Duration::from_millis(1)) == Ok(true) && !went_down {
            result = tetromino.shift(&mut playfield, Direction::Down);
        } 
        if let Err(ShiftError::BottomCollision) = result {
            playfield.draw(&mut terminal);
            tetromino.place_in_playfield(&mut playfield);
            playfield.clear_lines();
            *accept_input.lock().unwrap() = false;
            thread::sleep(Duration::from_millis(100));
            tetromino = bag.get();
            tetromino.spawn(&mut playfield);
            *accept_input.lock().unwrap() = true;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn input_thread(
    sender: std::sync::mpsc::Sender<crossterm::event::KeyEvent>,
    accept_input: Arc<Mutex<bool>>,
) -> std::thread::JoinHandle<()> {
    thread::spawn(move || loop {
        if let Ok(poll) = poll(Duration::from_millis(5)) {
            if poll && *accept_input.lock().unwrap() {
                if let Ok(Event::Key(key)) = read() {
                    sender.send(key).unwrap();
                }
            }
        }
    })
}
