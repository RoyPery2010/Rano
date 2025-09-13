use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, Clear, ClearType, size},
};
use std::fs;
use std::io::{self, Write, Result};
use std::process;
use std::time::Duration;
use errno::{errno, set_errno, Errno};

pub struct Editor {
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub buffer: Vec<String>,
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            cursor_x: 0,
            cursor_y: 0,
            buffer: Vec::new(),
        }
    }

    /// Open a file if it exists, else start empty
    pub fn open(&mut self, filename: &str) {
        if let Ok(contents) = fs::read_to_string(filename) {
            self.buffer = contents.lines().map(|l| l.to_string()).collect();
        } else {
            self.buffer = Vec::new(); // empty buffer
        }
    }

    /// Refresh the terminal screen
    pub fn refresh_screen(&self) -> Result<()> {
        clear_screen()?;
        draw_rows(&self.buffer)?;
        execute!(io::stdout(), cursor::MoveTo(self.cursor_x, self.cursor_y), cursor::Show)?;
        Ok(())
    }

    /// Process keypress, returns false if editor should quit
    pub fn process_keypress(&mut self) -> Result<bool> {
        if let Some(ev) = poll()? {
            if let Event::Key(KeyEvent { code, .. }) = ev {
                match code {
                    KeyCode::Char('q') => return Ok(false), // quit on 'q'
                    KeyCode::Up => { if self.cursor_y > 0 { self.cursor_y -= 1; } }
                    KeyCode::Down => {
                        if (self.cursor_y as usize) < self.buffer.len().saturating_sub(1) {
                            self.cursor_y += 1;
                        }
                    }
                    KeyCode::Left => { if self.cursor_x > 0 { self.cursor_x -= 1; } }
                    KeyCode::Right => { self.cursor_x += 1; }
                    _ => {}
                }
            }
        }
        Ok(true)
    }
}

/// Clear the screen
fn clear_screen() -> Result<()> {
    execute!(io::stdout(), cursor::Hide, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    Ok(())
}

/// Draw file lines or '~' for empty rows
fn draw_rows(buffer: &[String]) -> Result<()> {
    let (_, rows) = size()?;
    let mut stdout = io::stdout();

    for y in 0..rows {
        if (y as usize) < buffer.len() {
            writeln!(stdout, "{}", buffer[y as usize])?;
        } else {
            writeln!(stdout, "~")?;
        }
    }
    stdout.flush()?;
    Ok(())
}

/// poll() wrapper with errno
fn poll() -> Result<Option<Event>> {
    match event::poll(Duration::from_millis(500)) {
        Ok(true) => Ok(Some(read()?)),
        Ok(false) => Ok(None),
        Err(e) => {
            set_errno(Errno(e.raw_os_error().unwrap_or(1)));
            println!("poll fail - errno = {}\r", errno().0);
            die("poll failed");
        }
    }
}

/// read() wrapper with errno
fn read() -> Result<Event> {
    match event::read() {
        Ok(ev) => Ok(ev),
        Err(e) => {
            set_errno(Errno(e.raw_os_error().unwrap_or(1)));
            println!("read fail - errno = {}\r", errno().0);
            die("read failed");
        }
    }
}

/// Die and exit program
fn die(msg: &str) -> ! {
    let _ = disable_raw_mode();
    eprintln!("\r\n[FATAL] {}\r", msg);
    process::exit(1);
}
