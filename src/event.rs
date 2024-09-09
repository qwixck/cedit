use crate::editor::Modes;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

pub fn handle(editor: &mut crate::editor::Editor) -> std::io::Result<()> {
    if event::poll(std::time::Duration::ZERO)? {
        match event::read()? {
            Event::Key(event) => {
                if event.kind == event::KeyEventKind::Press {
                    match event.code {
                        KeyCode::Char(x) => {
                            if x == ':' && editor.mode == Modes::Normal {
                                editor.mode = Modes::Commanding;
                                editor.buffer.command.push(':');
                                editor.cursor.command += 1;
                            } else if editor.mode == Modes::Commanding {
                                editor
                                    .buffer
                                    .command
                                    .insert(editor.cursor.command as usize, x);
                                editor.cursor.command += 1;
                            } else {
                                if editor.mode == Modes::Insert {
                                    if !editor.has_edited {
                                        editor.has_edited = true;
                                    }

                                    editor.buffer.lines[editor.cursor.normal.1 as usize]
                                        .insert(editor.cursor.normal.0 as usize, x);
                                    editor.cursor.normal.0 += 1;
                                    editor.redraw_line()?;
                                }
                            }
                        }
                        KeyCode::Backspace => match editor.mode {
                            Modes::Insert => {
                                if !editor.has_edited {
                                    editor.has_edited = true;
                                }
                                if editor.cursor.normal.0 != 0 {
                                    if event.modifiers.contains(KeyModifiers::CONTROL) {

                                        return Ok(());
                                    }

                                    editor.cursor.normal.0 -= 1;

                                    editor.buffer.lines[editor.cursor.normal.1 as usize]
                                        .remove(editor.cursor.normal.0 as usize);
                                    editor.redraw_line()?;
                                } else {
                                    if editor.cursor.normal.1 != 0 {
                                        if editor.buffer.lines[editor.cursor.normal.1 as usize]
                                            .len()
                                            != 0
                                        {
                                            let buf = editor.buffer.lines
                                                [editor.cursor.normal.1 as usize]
                                                .clone();
                                            editor
                                                .buffer
                                                .lines
                                                .remove(editor.cursor.normal.1 as usize);
                                            editor.cursor.normal.1 -= 1;

                                            if editor.screen != 0 {
                                                editor.scroll_up()?;
                                            } else {
                                                editor.cursor.r#virtual -= 1;
                                            }

                                            if editor.buffer.lines[editor.cursor.normal.1 as usize]
                                                .len()
                                                != 0
                                            {
                                                editor.cursor.normal.0 = editor.buffer.lines
                                                    [editor.cursor.normal.1 as usize]
                                                    .len()
                                                    as u16;
                                            } else {
                                                editor.cursor.normal.0 = 0;
                                            }
                                            editor.buffer.lines[editor.cursor.normal.1 as usize]
                                                .push_str(buf.as_str());
                                        } else {
                                            editor
                                                .buffer
                                                .lines
                                                .remove(editor.cursor.normal.1 as usize);
                                            editor.cursor.normal.1 -= 1;

                                            if editor.screen != 0 {
                                                editor.scroll_up()?;
                                            } else {
                                                editor.cursor.r#virtual -= 1;
                                            }

                                            editor.cursor.normal.0 = editor.buffer.lines
                                                [editor.cursor.normal.1 as usize]
                                                .len()
                                                as u16;
                                        }
                                    }

                                    if editor.cursor.normal.1 != editor.buffer.lines.len() as u16 {
                                        editor.redraw_screen()?;
                                    }
                                }
                                editor.redraw_screen()?;
                            }
                            Modes::Commanding => {
                                if editor.cursor.command != 0 {
                                    editor.cursor.command -= 1;
                                    editor.buffer.command.remove(editor.cursor.command as usize);

                                    if editor.buffer.command.is_empty() {
                                        editor.mode = Modes::Normal;
                                    }
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Enter => match editor.mode {
                            Modes::Insert => {
                                if !editor.has_edited {
                                    editor.has_edited = true;
                                }

                                editor
                                    .buffer
                                    .lines
                                    .insert(editor.cursor.normal.1 as usize + 1, String::new());

                                if editor.cursor.normal.0
                                    != editor.buffer.lines[editor.cursor.normal.1 as usize].len()
                                        as u16
                                {
                                    let buf = editor.buffer.lines[editor.cursor.normal.1 as usize]
                                        .split_off(editor.cursor.normal.0 as usize);

                                    editor.cursor.normal.1 += 1;
                                    editor.cursor.normal.0 = 0;

                                    editor.buffer.lines[editor.cursor.normal.1 as usize]
                                        .push_str(buf.as_str().trim_start());

                                    editor.cursor.normal.0 = 0;
                                } else {
                                    editor.cursor.normal.1 += 1;
                                    editor.cursor.normal.0 = 0;
                                }

                                if editor.cursor.r#virtual == (editor.size.1 - 2) {
                                    editor.scroll_down()?;
                                } else {
                                    editor.cursor.r#virtual += 1;
                                }

                                editor.redraw_screen()?;
                            }
                            Modes::Commanding => {
                                editor.cursor.command = 0;
                                editor.mode = Modes::Normal;
                                editor.command(editor.buffer.command.clone())?;
                                editor.buffer.command = String::new();

                                return Ok(());
                            }
                            Modes::Normal => editor.mode = Modes::Insert,
                            _ => {}
                        },
                        KeyCode::Up => {
                            if editor.mode == Modes::Insert || editor.mode == Modes::Normal {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.normal.1 = 0;
                                    editor.cursor.r#virtual = 0;
                                    editor.screen = 0;

                                    editor.redraw_screen()?;
                                    return Ok(());
                                }

                                if editor.cursor.normal.1 != 0 {
                                    editor.cursor.normal.1 -= 1;

                                    if editor.screen != 0 {
                                        if editor.cursor.r#virtual == 0 {
                                            editor.scroll_up()?;
                                        } else {
                                            editor.cursor.r#virtual -= 1;
                                        }
                                    } else {
                                        editor.cursor.r#virtual -= 1;
                                    }

                                    if editor.cursor.normal.0
                                        > editor.buffer.lines[editor.cursor.normal.1 as usize].len()
                                            as u16
                                    {
                                        editor.cursor.normal.0 = editor.buffer.lines
                                            [editor.cursor.normal.1 as usize]
                                            .len()
                                            as u16;
                                    }
                                }
                            }
                        }
                        KeyCode::Down => {
                            if editor.mode == Modes::Insert || editor.mode == Modes::Normal {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.normal.1 = (editor.buffer.lines.len() - 1) as u16;
                                    
                                    if editor.buffer.lines.len() as u16 >= editor.size.1 {
                                        editor.cursor.r#virtual = editor.size.1 - 2;
                                        editor.screen = (editor.buffer.lines.len() + 1) as u16 - editor.size.1;
                                    } else {
                                        editor.cursor.r#virtual = (editor.buffer.lines.len() - 1) as u16;
                                    }
                                    editor.redraw_screen()?;
                                    return Ok(());
                                }
                                
                                if editor.cursor.normal.1 + 1 != editor.buffer.lines.len() as u16 {
                                    editor.cursor.normal.1 += 1;

                                    if editor.cursor.r#virtual == (editor.size.1 - 2) {
                                        editor.scroll_down()?;
                                    } else {
                                        editor.cursor.r#virtual += 1;
                                    }

                                    if editor.cursor.normal.0
                                        > editor.buffer.lines[editor.cursor.normal.1 as usize].len()
                                            as u16
                                    {
                                        editor.cursor.normal.0 = editor.buffer.lines
                                            [editor.cursor.normal.1 as usize]
                                            .len()
                                            as u16;
                                    }
                                }
                            }
                        }
                        KeyCode::Left => match editor.mode {
                            Modes::Insert | Modes::Normal => {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.normal.0 = 0;
                                    return Ok(());
                                }

                                if editor.cursor.normal.0 != 0 {
                                    editor.cursor.normal.0 -= 1;
                                }
                            }
                            Modes::Commanding => {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.command = 0;
                                    return Ok(());
                                }

                                if editor.cursor.command != 0 {
                                    editor.cursor.command -= 1;
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Right => match editor.mode {
                            Modes::Insert | Modes::Normal => {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.normal.0 = editor.buffer.lines[editor.cursor.normal.1 as usize].len() as u16;
                                    return Ok(());
                                }

                                if editor.cursor.normal.0
                                    != editor.buffer.lines[editor.cursor.normal.1 as usize].len()
                                        as u16
                                {
                                    editor.cursor.normal.0 += 1;
                                }
                            }
                            Modes::Commanding => {
                                if event.modifiers.contains(KeyModifiers::ALT) {
                                    editor.cursor.command = editor.buffer.command.len() as u16;
                                    return Ok(());
                                }

                                if editor.cursor.command != editor.buffer.command.len() as u16 {
                                    editor.cursor.command += 1;
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Esc => match editor.mode {
                            Modes::Insert => editor.mode = Modes::Normal,
                            Modes::Commanding => {
                                editor.buffer.command = String::new();
                                editor.cursor.command = 0;
                                editor.mode = Modes::Normal;
                            }
                            _ => {}
                        },
                        KeyCode::Tab => match editor.mode {
                            Modes::Insert => {
                                if !editor.has_edited {
                                    editor.has_edited = true;
                                }
                                editor.buffer.lines[editor.cursor.normal.1 as usize]
                                    .push_str("    ");
                                editor.cursor.normal.0 += 4;
                            }
                            Modes::Commanding => {
                                editor.buffer.command.push_str("    ");
                                editor.cursor.command += 4;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    editor.redraw_status()?;
                }
            }
            Event::Resize(nwidth, nheight) => {
                editor.size.0 = nwidth;
                editor.size.1 = nheight;
                editor.redraw_screen()?;
            }
            _ => {}
        }
    }

    Ok(())
}
