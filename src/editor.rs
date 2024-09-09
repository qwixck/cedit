use crate::buffer::Buffer;
use std::io::{self, Result, Write};

use crossterm::style::Stylize;
use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Modes {
    Insert,
    Normal,
    Commanding,
    _Visual,
}

#[derive(Debug)]
pub struct Cursor {
    pub normal: (u16, u16),
    pub command: u16,
    pub r#virtual: u16,
    pub _visual: (u16, u16),
}

#[derive(Debug)]
pub struct Editor {
    pub buffer: Buffer,
    pub mode: Modes,
    pub stdout: io::Stdout,
    pub cursor: Cursor,
    pub size: (u16, u16),
    pub screen: u16,
    pub window: bool,
    pub has_edited: bool,
}

impl std::fmt::Display for Modes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modes::Insert => write!(f, "INSERT"),
            Modes::Normal => write!(f, "NORMAL"),
            Modes::Commanding => write!(f, "COMMAND"),
            Modes::_Visual => write!(f, "VISUAL"),
        }
    }
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Self> {
        Ok(Self {
            buffer: buffer,
            mode: Modes::Normal,
            stdout: io::stdout(),
            cursor: Cursor {
                normal: (0, 0),
                command: 0,
                r#virtual: 0,
                _visual: (0, 0),
            },
            size: terminal::size()?,
            screen: 0,
            window: true,
            has_edited: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(self.stdout, terminal::EnterAlternateScreen)?;

        self.redraw_screen()?;

        while self.window {
            match self.mode {
                Modes::Insert | Modes::Normal => {
                    execute!(
                        self.stdout,
                        cursor::MoveTo(self.cursor.normal.0, self.cursor.r#virtual)
                    )?;
                }
                Modes::Commanding => {
                    execute!(
                        self.stdout,
                        cursor::MoveTo(self.cursor.command, self.size.1)
                    )?;
                }
                _ => {}
            }

            crate::event::handle(self)?;
        }

        execute!(self.stdout, terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    pub fn redraw_line(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::MoveTo(0, self.cursor.r#virtual))?;
        execute!(self.stdout, Clear(ClearType::CurrentLine))?;
        write!(
            self.stdout,
            "{}",
            self.buffer.lines[self.cursor.normal.1 as usize]
        )?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn _redraw_line_at(&mut self, idx: u16) -> Result<()> {
        if idx > self.size.1 {
            panic!("index is bigger than terminal's screen height");
        }
        execute!(self.stdout, cursor::MoveTo(0, idx))?;
        execute!(self.stdout, Clear(ClearType::CurrentLine))?;
        write!(self.stdout, "{}", self.buffer.lines[idx as usize])?;
        self.stdout.flush()?;

        Ok(())
    }

    pub fn redraw_screen(&mut self) -> Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;

        for i in self.screen..(self.screen + (self.size.1 - 1)) {
            execute!(self.stdout, cursor::MoveTo(0, i - self.screen))?;

            if i < self.buffer.lines.len() as u16 {
                write!(self.stdout, "{}", self.buffer.lines[i as usize])?;
            } else {
                write!(self.stdout, "~")?;
            }
        }
        self.redraw_status()?;

        Ok(())
    }

    pub fn redraw_status(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            cursor::MoveTo(self.cursor.normal.0, self.size.1)
        )?;
        execute!(self.stdout, Clear(ClearType::CurrentLine))?;

        match self.mode {
            Modes::Insert | Modes::Normal => {
                execute!(self.stdout, cursor::MoveTo(0, self.size.1))?;
                write!(self.stdout, "[{}]", self.mode)?;
                execute!(self.stdout, cursor::MoveTo(self.size.0 - 18, self.size.1))?;
                write!(
                    self.stdout,
                    "{}:{}",
                    self.cursor.normal.1 + 1,
                    self.cursor.normal.0 + 1
                )?;
            }
            Modes::Commanding => {
                execute!(self.stdout, cursor::MoveTo(0, self.size.1))?;
                write!(self.stdout, "{}", self.buffer.command)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn scroll_down(&mut self) -> Result<()> {
        self.screen += 1;
        self.redraw_screen()?;

        Ok(())
    }

    pub fn scroll_up(&mut self) -> Result<()> {
        if self.screen != 0 {
            self.screen -= 1;
        }
        self.redraw_screen()?;

        Ok(())
    }

    pub fn command(&mut self, command: String) -> Result<()> {
        match command.as_str().strip_prefix(":").unwrap() {
            "q" => {
                if self.has_edited {
                    self.draw_error("The file has changed".to_string())?;
                } else {
                    self.window = false;
                }
            }
            "q!" => {
                self.window = false;
            }
            "wq" => match self.buffer.save() {
                Ok(_) => self.window = false,
                Err(err) => self.draw_error(err.to_string())?,
            },
            "w" => {
                if let Err(err) = self.buffer.save() {
                    self.draw_error(err.to_string())?
                }
            }
            ":" => self.redraw_status()?,
            unknown => {
                if unknown.starts_with("goto") {
                    self.draw_error("Not implemented".to_string())?;
                } else {
                    self.draw_error(format!("Unknown command: {}", unknown))?;
                }
            }
        }

        Ok(())
    }

    pub fn draw_error(&self, error: String) -> Result<()> {
        execute!(self.stdout.lock(), cursor::MoveTo(0, self.size.1))?;
        execute!(self.stdout.lock(), Clear(ClearType::CurrentLine))?;
        write!(self.stdout.lock(), "{}", format!("ERROR: {error}").on_red())?;
        self.stdout.lock().flush()?;

        Ok(())
    }
}
