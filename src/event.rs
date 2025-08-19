use crate::editor::Modes;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

impl crate::editor::Editor {
    pub fn handle(&mut self) -> std::io::Result<()> {
        if event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                Event::Key(event) => {
                    if event.kind == event::KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char(x) => {
                                if x == ':' && self.mode == Modes::Normal {
                                    self.mode = Modes::Commanding;
                                    self.buffer.command.push(':');
                                    self.cursor.command += 1;
                                } else if self.mode == Modes::Commanding {
                                    self.buffer.command.insert(self.cursor.command as usize, x);
                                    self.cursor.command += 1;
                                } else if self.mode == Modes::Insert {
                                    if !self.has_edited {
                                        self.has_edited = true;
                                    }

                                    self.buffer.lines[self.cursor.normal.1 as usize]
                                        .insert(self.cursor.normal.0 as usize, x);
                                    self.cursor.normal.0 += 1;
                                    self.redraw_line()?;
                                }
                            }
                            KeyCode::Backspace => match self.mode {
                                Modes::Insert => {
                                    if !self.has_edited {
                                        self.has_edited = true;
                                    }
                                    if self.cursor.normal.0 != 0 {
                                        if event.modifiers.contains(KeyModifiers::CONTROL) {
                                            if self.cursor.normal.0 != 0 {
                                                if self.buffer.lines[self.cursor.normal.1 as usize]
                                                    .chars()
                                                    .nth(self.cursor.normal.0 as usize - 1)
                                                    .map(|c| c.is_whitespace())
                                                    == Some(true)
                                                {
                                                    while self.cursor.normal.0 != 0
                                                        && self.buffer.lines
                                                            [self.cursor.normal.1 as usize]
                                                            .chars()
                                                            .nth(self.cursor.normal.0 as usize - 1)
                                                            .map(|c| c.is_whitespace())
                                                            == Some(true)
                                                    {
                                                        self.cursor.normal.0 -= 1;

                                                        self.buffer.lines
                                                            [self.cursor.normal.1 as usize]
                                                            .remove(self.cursor.normal.0 as usize);
                                                    }
                                                } else {
                                                    while self.cursor.normal.0 != 0
                                                        && self.buffer.lines
                                                            [self.cursor.normal.1 as usize]
                                                            .chars()
                                                            .nth(self.cursor.normal.0 as usize - 1)
                                                            .map(|c| !c.is_whitespace())
                                                            == Some(true)
                                                    {
                                                        self.cursor.normal.0 -= 1;

                                                        self.buffer.lines
                                                            [self.cursor.normal.1 as usize]
                                                            .remove(self.cursor.normal.0 as usize);
                                                    }
                                                }
                                                self.redraw_line()?;
                                            }
                                            return Ok(());
                                        }
                                        self.cursor.normal.0 -= 1;

                                        self.buffer.lines[self.cursor.normal.1 as usize]
                                            .remove(self.cursor.normal.0 as usize);
                                        self.redraw_line()?;
                                    } else {
                                        if self.cursor.normal.1 != 0 {
                                            if !self.buffer.lines[self.cursor.normal.1 as usize]
                                                .is_empty()
                                            {
                                                let buf = self.buffer.lines
                                                    [self.cursor.normal.1 as usize]
                                                    .clone();
                                                self.buffer
                                                    .lines
                                                    .remove(self.cursor.normal.1 as usize);
                                                self.cursor.normal.1 -= 1;

                                                if self.screen != 0 {
                                                    self.scroll_up()?;
                                                } else {
                                                    self.cursor.viewport.1 -= 1;
                                                }

                                                if !self.buffer.lines[self.cursor.normal.1 as usize]
                                                    .is_empty()
                                                {
                                                    self.cursor.normal.0 = self.buffer.lines
                                                        [self.cursor.normal.1 as usize]
                                                        .len()
                                                        as u16;
                                                } else {
                                                    self.cursor.normal.0 = 0;
                                                }
                                                self.buffer.lines[self.cursor.normal.1 as usize]
                                                    .push_str(buf.as_str());
                                            } else {
                                                self.buffer
                                                    .lines
                                                    .remove(self.cursor.normal.1 as usize);
                                                self.cursor.normal.1 -= 1;

                                                if self.screen != 0 {
                                                    self.scroll_up()?;
                                                } else {
                                                    self.cursor.viewport.1 -= 1;
                                                }

                                                self.cursor.normal.0 = self.buffer.lines
                                                    [self.cursor.normal.1 as usize]
                                                    .len()
                                                    as u16;
                                            }
                                        }
                                        if self.cursor.normal.0 != 0 && self.cursor.normal.1 != 0 {
                                            self.redraw_screen()?;
                                        }
                                    }
                                }
                                Modes::Commanding => {
                                    if self.cursor.command != 0 {
                                        self.cursor.command -= 1;
                                        self.buffer.command.remove(self.cursor.command as usize);

                                        if self.buffer.command.is_empty() {
                                            self.mode = Modes::Normal;
                                        }
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Enter => match self.mode {
                                Modes::Insert => {
                                    if !self.has_edited {
                                        self.has_edited = true;
                                    }

                                    self.buffer
                                        .lines
                                        .insert(self.cursor.normal.1 as usize + 1, String::new());

                                    if self.cursor.normal.0
                                        != self.buffer.lines[self.cursor.normal.1 as usize].len()
                                            as u16
                                    {
                                        let buf = self.buffer.lines[self.cursor.normal.1 as usize]
                                            .split_off(self.cursor.normal.0 as usize);

                                        self.cursor.normal.1 += 1;
                                        self.cursor.normal.0 = 0;

                                        self.buffer.lines[self.cursor.normal.1 as usize]
                                            .push_str(buf.as_str().trim_start());

                                        self.cursor.normal.0 = 0;
                                    } else {
                                        self.cursor.normal.1 += 1;
                                        self.cursor.normal.0 = 0;
                                    }

                                    if self.cursor.viewport.1 == (self.size.1 - 2) {
                                        self.scroll_down()?;
                                    } else {
                                        self.cursor.viewport.1 += 1;
                                    }

                                    self.redraw_screen()?;
                                }
                                Modes::Commanding => {
                                    self.cursor.command = 0;
                                    self.mode = Modes::Normal;
                                    self.command(self.buffer.command.clone())?;
                                    self.buffer.command = String::new();

                                    return Ok(());
                                }
                                Modes::Normal => self.mode = Modes::Insert,
                                _ => {}
                            },
                            KeyCode::Up => {
                                if self.mode == Modes::Insert || self.mode == Modes::Normal {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.normal.1 = 0;

                                        if self.cursor.normal.0
                                            > self.buffer.lines[self.cursor.normal.1 as usize].len()
                                                as u16
                                        {
                                            self.cursor.normal.0 = self.buffer.lines
                                                [self.cursor.normal.1 as usize]
                                                .len()
                                                as u16;
                                        }

                                        self.cursor.viewport.1 = 0;
                                        self.screen = 0;

                                        self.redraw_screen()?;
                                        return Ok(());
                                    }

                                    if self.cursor.normal.1 != 0 {
                                        self.cursor.normal.1 -= 1;

                                        if self.screen != 0 {
                                            if self.cursor.viewport.1 == 0 {
                                                self.scroll_up()?;
                                            } else {
                                                self.cursor.viewport.1 -= 1;
                                            }
                                        } else {
                                            self.cursor.viewport.1 -= 1;
                                        }

                                        if self.cursor.normal.0
                                            > self.buffer.lines[self.cursor.normal.1 as usize].len()
                                                as u16
                                        {
                                            self.cursor.normal.0 = self.buffer.lines
                                                [self.cursor.normal.1 as usize]
                                                .len()
                                                as u16;
                                        }
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if self.mode == Modes::Insert || self.mode == Modes::Normal {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.normal.1 = (self.buffer.lines.len() - 1) as u16;

                                        if self.cursor.normal.0
                                            > self.buffer.lines[self.cursor.normal.1 as usize].len()
                                                as u16
                                        {
                                            self.cursor.normal.0 = self.buffer.lines
                                                [self.cursor.normal.1 as usize]
                                                .len()
                                                as u16;
                                        }

                                        if self.buffer.lines.len() as u16 >= self.size.1 {
                                            self.cursor.viewport.1 = self.size.1 - 2;
                                            self.screen =
                                                (self.buffer.lines.len() + 1) as u16 - self.size.1;
                                        } else {
                                            self.cursor.viewport.1 =
                                                (self.buffer.lines.len() - 1) as u16;
                                        }
                                        self.redraw_screen()?;
                                        return Ok(());
                                    }

                                    if self.cursor.normal.1 + 1 != self.buffer.lines.len() as u16 {
                                        self.cursor.normal.1 += 1;

                                        if self.cursor.viewport.1 == (self.size.1 - 2) {
                                            self.scroll_down()?;
                                        } else {
                                            self.cursor.viewport.1 += 1;
                                        }

                                        if self.cursor.normal.0
                                            > self.buffer.lines[self.cursor.normal.1 as usize].len()
                                                as u16
                                        {
                                            self.cursor.normal.0 = self.buffer.lines
                                                [self.cursor.normal.1 as usize]
                                                .len()
                                                as u16;
                                        }
                                    }
                                }
                            }
                            KeyCode::Left => match self.mode {
                                Modes::Insert | Modes::Normal => {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.normal.0 = 0;
                                        return Ok(());
                                    }

                                    if self.cursor.normal.0 != 0 {
                                        self.cursor.normal.0 -= 1;
                                    }
                                }
                                Modes::Commanding => {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.command = 0;
                                        return Ok(());
                                    }

                                    if self.cursor.command != 0 {
                                        self.cursor.command -= 1;
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Right => match self.mode {
                                Modes::Insert | Modes::Normal => {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.normal.0 =
                                            self.buffer.lines[self.cursor.normal.1 as usize].len()
                                                as u16;
                                        return Ok(());
                                    }

                                    if self.cursor.normal.0
                                        != self.buffer.lines[self.cursor.normal.1 as usize].len()
                                            as u16
                                    {
                                        self.cursor.normal.0 += 1;
                                    }
                                }
                                Modes::Commanding => {
                                    if event.modifiers.contains(KeyModifiers::ALT) {
                                        self.cursor.command = self.buffer.command.len() as u16;
                                        return Ok(());
                                    }

                                    if self.cursor.command != self.buffer.command.len() as u16 {
                                        self.cursor.command += 1;
                                    }
                                }
                                _ => {}
                            },
                            KeyCode::Esc => match self.mode {
                                Modes::Insert => self.mode = Modes::Normal,
                                Modes::Commanding => {
                                    self.buffer.command = String::new();
                                    self.cursor.command = 0;
                                    self.mode = Modes::Normal;
                                }
                                _ => {}
                            },
                            KeyCode::Tab => match self.mode {
                                Modes::Insert => {
                                    if !self.has_edited {
                                        self.has_edited = true;
                                    }
                                    self.buffer.lines[self.cursor.normal.1 as usize]
                                        .insert_str(self.cursor.normal.0 as usize, "    ");
                                    self.cursor.normal.0 += 4;
                                    self.redraw_line()?;
                                }
                                Modes::Commanding => {
                                    self.buffer.command.push_str("    ");
                                    self.cursor.command += 4;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                        self.redraw_status()?;
                    }
                }
                Event::Resize(nwidth, nheight) => {
                    self.size.0 = nwidth;
                    self.size.1 = nheight;
                    self.redraw_screen()?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
