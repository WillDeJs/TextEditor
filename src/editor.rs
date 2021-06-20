use crate::document::Document;
use crate::document::SearchDirection;
use crate::row::Row;
use crate::terminal::Color;
use crate::terminal::Terminal;
use crate::terminal::{InputEvent, KeyEvent};
use std::result::Result;
use std::time::Duration;
use std::time::Instant;

/// Console Editor
///
pub struct Editor {
    terminal: Terminal,
    cursor_position: Position,
    should_quit: bool,
    document: Document,
    offset: Position,
    status_message: StatusMessage,
}

#[derive(Default, Debug, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}
impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            text: String::new(),
            time: Instant::now(),
        }
    }
}
impl StatusMessage {
    fn from(status: String) -> Self {
        Self {
            text: status,
            time: Instant::now(),
        }
    }
}
pub enum Command {
    Execute(fn() -> bool),
    Save,
    Search,
    Cancel,
    Quit,
}
impl Editor {
    /// Default constructor, takes no argument and builds an Editor object.
    pub fn default() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let document = if args.len() > 1 {
            Document::open(&args[1]).unwrap_or_default()
        } else {
            Document::default()
        };
        Self {
            terminal: Terminal::default().expect("Error: Could not create terminal on device"),
            should_quit: false,
            cursor_position: Position::default(),
            document: document,
            offset: Position::default(),
            status_message: StatusMessage::default(),
        }
    }

    /// Runs an editor on the console.
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                let _ = self.clear_screen();
                self.die(error, 1);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_input() {
                let _ = self.clear_screen();
                self.die(error, 1);
            }
        }
    }

    /// Process any key pressed by the user on the console
    pub fn process_input(&mut self) -> Result<(), std::io::Error> {
        let key_pressed = self.terminal.read_keypress();
        if let Some(event) = key_pressed {
            //     self.document.insert(event, &self.cursor_position);
            //     self.move_cursor(KeyEvent::Right);
            // }
            match event {
                InputEvent::Keyboard(key) => match key {
                    KeyEvent::Char(e) => {
                        self.document.insert(e, &self.cursor_position);
                        self.move_cursor(KeyEvent::Right)
                    }
                    KeyEvent::Ctrl('Q') => {
                        let _ = self.quit()?;
                    }
                    KeyEvent::Ctrl('S') => {
                        let _ = self.save()?;
                    }
                    KeyEvent::Ctrl('F') => {
                        self.search();
                    }
                    KeyEvent::Enter => {
                        self.document.insert('\n', &self.cursor_position);
                        self.move_cursor(KeyEvent::Down);
                        self.move_cursor(KeyEvent::Home);
                    }
                    KeyEvent::Tab => {
                        self.document.insert('\t', &self.cursor_position);
                        self.move_cursor(KeyEvent::Right)
                    }

                    KeyEvent::Backspace => {
                        if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                            self.move_cursor(KeyEvent::Left);
                            self.document.delete(&self.cursor_position);
                        }
                    }
                    KeyEvent::Delete => {
                        self.document.delete(&self.cursor_position);
                    }
                    KeyEvent::Left
                    | KeyEvent::Right
                    | KeyEvent::Up
                    | KeyEvent::Down
                    | KeyEvent::PageDown
                    | KeyEvent::PageUp
                    | KeyEvent::Home
                    | KeyEvent::End => self.move_cursor(key),
                    _ => (),
                },
                _ => (),
            }
        }
        self.scroll();
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide();
        self.terminal.cursor_position(&Position::default());
        if self.should_quit {
            self.terminal.clear_screen();
            println!("Goodbye...");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            self.terminal.cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        self.terminal.cursor_show();
        self.terminal.refresh_screen()
    }

    fn clear_screen(&self) -> Result<(), std::io::Error> {
        self.terminal.cursor_hide();
        self.terminal.clear_screen();
        self.draw_rows();
        self.terminal.cursor_show();
        self.terminal.flush();
        self.terminal.cursor_position(&Position::default());
        Ok(())
    }

    fn draw_rows(&self) {
        let height = self.terminal.height();
        for terminal_row in 0..height {
            self.terminal.clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row))
            {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }
    fn draw_row(&self, row: &Row) {
        let width = self.terminal.width();
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn move_cursor(&mut self, key: KeyEvent) {
        let Position { mut x, mut y } = self.cursor_position;
        let doc_len = self.document.len();
        let height = self.terminal.height();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            KeyEvent::Up => y = y.saturating_sub(1),
            KeyEvent::Down => {
                if y < doc_len {
                    y = y.saturating_add(1);
                }
            }
            KeyEvent::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            KeyEvent::Right => {
                if x < width {
                    x += 1;
                } else if y < doc_len {
                    y += 1;
                    x = 0;
                }
            }
            KeyEvent::PageDown => y = y.saturating_add(height),
            KeyEvent::PageUp => y = y.saturating_sub(height),
            KeyEvent::End => x = width,
            KeyEvent::Home => x = 0,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width
        }
        self.cursor_position = Position { x, y };
        // self.terminal.cursor_position(&self.cursor_position);
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.width();
        let height = self.terminal.height();
        let mut offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    #[allow(non_snake_case)]
    fn draw_welcome_message(&self) {
        let VERSION = std::env::var("CARGO_PKG_VERSION").unwrap();
        let mut welcome_message = format!("Text editor -- version {}", VERSION);
        let width = self.terminal.width();
        let len = welcome_message.len();
        #[allow(clippy::integer_arithmetic, clippy::integer_division)]
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }
    fn draw_status_bar(&self) {
        let width = self.terminal.width();
        let default_filename = "[No name]".to_string();
        let filename = match &self.document.filename {
            Some(filename) => filename,
            None => &default_filename,
        };

        let is_modified = if self.document.is_dirty() {
            "(modified)"
        } else {
            ""
        };
        let mut status = format!(
            "{} - {} lines{}",
            filename,
            self.document.len(),
            is_modified
        );
        let line_indicator = format!(
            "{} | {}/{}",
            self.document.filetype,
            self.cursor_position.y + 1,
            self.document.len()
        );

        let length = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(length)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        // self.terminal.cursor_position(&Position{x: 0, y: height});
        if self.document.is_dirty() {
            self.terminal.set_bg_color(Color::Red);
        } else {
            self.terminal.set_bg_color(Color::DarkCyan);
        }
        println!("{}", status);
        self.terminal.reset_bg_color();
        // self.terminal.cursor_position(&self.cursor_position);
    }

    fn draw_message_bar(&self) {
        self.terminal.clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.width());
            print!("{}", text);
        }
    }
    fn prompt(&mut self, message: &str) -> Result<String, std::io::Error> {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", message, result));
            self.refresh_screen()?;
            if let Some(event) = self.terminal.read_keypress() {
                match event {
                    InputEvent::Keyboard(key) => match key {
                        KeyEvent::Enter => {
                            self.status_message = StatusMessage::from(String::new());
                            break;
                        }
                        KeyEvent::Char(c) => {
                            result.push(c);
                        }
                        KeyEvent::Backspace => {
                            if !result.is_empty() {
                                result.pop();
                            }
                        }
                        KeyEvent::Esc => {
                            self.status_message = StatusMessage::from(String::new());
                            result.clear();
                            break;
                        }
                        _ => (),
                    },
                    _ => (),
                }
            };
        }
        Ok(result)
    }

    fn quit(&mut self) -> Result<String, std::io::Error> {
        if !self.document.is_dirty() {
            self.should_quit = true;
        } else {
            if let Ok(answer) = self.prompt("Quit without saving? (Y/N)") {
                if answer.to_lowercase() == "y" {
                    self.should_quit = true;
                }
            }
        }
        Ok(String::new())
    }

    fn save(&mut self) -> Result<String, std::io::Error> {
        if self.document.filename.is_none() {
            if let Some(document) = Some(self.prompt("Save as:")?) {
                if !document.is_empty() {
                    self.document.filename = Some(document)
                }
            }
        }
        if self.document.filename.is_some() {
            self.document.save()?;
        }
        return Ok(String::from("Saved successfully..."));
    }

    fn search(&mut self) {
        if let Ok(query) = self.prompt("Search: ") {
            loop {
                self.status_message =
                    StatusMessage::from(format!("Searching '{}': (ESC | <- | ->)", &query));
                let _ = self.refresh_screen();
                if let Some(event) = self.terminal.read_keypress() {
                    let current_position = self.cursor_position.clone();
                    match event {
                        InputEvent::Keyboard(KeyEvent::Left) => {
                            if let Some(position) = self.document.find(
                                &query,
                                current_position.clone(),
                                SearchDirection::Backward,
                            ) {
                                self.cursor_position = position;
                                self.scroll();
                            }
                        }
                        InputEvent::Keyboard(KeyEvent::Right) => {
                            if let Some(position) = self.document.find(
                                &query,
                                current_position.clone(),
                                SearchDirection::Forward,
                            ) {
                                self.cursor_position = position;
                                self.scroll();
                            }
                        }
                        InputEvent::Keyboard(KeyEvent::Esc) => {
                            self.status_message = StatusMessage::from("".to_string());
                            self.document.search_string = Option::None;
                            self.document.hightlight();
                            break;
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    fn die<T>(&self, error: T, errnum: i32)
    where
        T: std::fmt::Display,
    {
        self.terminal.clear_screen();
        println!("{}", error);
        std::process::exit(errnum);
    }
}
