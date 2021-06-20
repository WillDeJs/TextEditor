use crossterm::input;
use std::result::Result;
use std::io::Write;
use crossterm::RawScreen;
use crossterm::TerminalCursor;
use crossterm::ClearType;

use crate::editor::Position;
pub type InputEvent = crossterm::InputEvent;
pub type KeyEvent = crossterm::KeyEvent;

// Numerical constants for some keys
const K_ENTER : usize= 0x0D;
const K_DELETE : usize= 0x53;
const K_BACKSPACE : usize = 0x8;
const K_TAB : usize= 0x9;
const K_PAGEDOWN : usize = 0x51;
const K_PAGEUP : usize = 0x49;
const K_HOME : usize = 0x47;
const K_END : usize = 0x4F;
const K_ARROW_UP : usize = 0x48;
const K_ARROW_DOWN : usize = 0x50;
const K_ARROW_LEFT : usize = 0x4B;
const K_ARROW_RIGHT : usize = 0x4D;
const K_ESCAPE : usize = 0x1B;


pub type Color = crossterm::Color;
pub struct Size {
    width: u16,
    height: u16,
}

pub struct Terminal {
    pub size: Size,
    _stdout : Result<RawScreen, std::io::Error>,
    _cursor : TerminalCursor,
    _internal: crossterm::Terminal,
}
///
/// Wrapper around a crossterm terminal with default 
/// Configuration for the text editor being written.
/// 
/// This configuration includes cursor and color setup 
///     Mouse input disabled
///     Raw mode enabled by default
///     Keypress events handling
/// 
/// 
#[allow(unused_must_use)]
impl Terminal {
    pub fn default() -> Result<Terminal, std::io::Error> {
        let _terminal = crossterm::Terminal::new();
        let size = _terminal.size().expect("Could not get terminal size");
        crossterm::input().disable_mouse_mode();
        Ok(Terminal {
            size: Size {
                width: size.0,
                height: size.1,
            },
            _cursor: crossterm::TerminalCursor::new(),
            _internal: _terminal,
            _stdout: Ok(RawScreen::into_raw_mode().unwrap()),
        })
    }
    pub fn cursor_position(&self,position : &Position) {
        let Position{x, y} = position;
        self._cursor.goto(*x as u16, *y as u16);
    }
    pub fn current_cursor_position(&self) -> std::result::Result<Position, &str> {
        let (x,y) = self._cursor.pos().or_else(|_| Err("Something went wrong getting cursor position"))?;
        Ok(Position{x: x as usize,y:y as usize})
    }
    pub fn read_keypress(&self) -> Option<InputEvent> {
        loop {
            // match input().read_sync().next() {
            //     Some(event) => return Some(event),
            //     _ => (),
            // };
            match input().read_char() {
                Ok(c) => {
                    return Some(InputEvent::Keyboard(self.map_to_key(c)));
                }
                _  => (),
             };
        }
    }
    ///
    /// Set the background color for the  to the given color
    /// color-> Crossterm::TerminalColor
    /// 
    pub fn set_bg_color(&self, color: Color) {
        crossterm::TerminalColor::new().set_bg(color);
    }

    ///
    /// Set the foreground color for the  to the given color
    /// color-> Crossterm::TerminalColor
    /// 
    pub fn set_fg_color(&self, color: Color) {
        crossterm::TerminalColor::new().set_fg(color);
    }

    /// Reset the color for the background
    /// As the color must be set and reset on the terminal
    pub fn reset_bg_color(&self) {
        crossterm::TerminalColor::new().reset();
    }

    /// Reset the color for the foreground
    /// As the color must be set and reset on the terminal
    pub fn reset_fg_color(&self) {
        crossterm::TerminalColor::new().reset();
    }

    /// Get the current size of the terminal
    /// Returns a Size struct which is formated {width, height}
    pub fn size(&self) -> &Size {
        &self.size
    }

    /// Helper method since I was lazy to be unpacking width and height from size
    /// This retrieves the height
    pub fn height(&self) -> usize {
        self.size.height as usize - 2
    }
    
    /// Helper method since I was lazy to be unpacking width and height from size
    /// This retrieves the width
    pub fn width(&self) -> usize {
        self.size.width as usize
    }
    pub fn clear_screen(&self) {
        self._internal.clear(ClearType::All);
    }
    pub fn refresh_screen(&self) -> Result<(), std::io::Error> {
        std::io::stdout().flush()
    }
    pub fn cursor_hide (&self) {
        self._cursor.hide();

    }
    pub fn cursor_show(&self) {
        self._cursor.show();
    }
    pub fn flush(&self) {
        std::io::stdout().flush();
    }
    pub fn clear_current_line(&self) {
        self._internal.clear(ClearType::CurrentLine);

    }

    fn is_control_key (&self, c: char) -> bool {
        let numc = c as usize;
        return numc < 32;
    }

    // crosterm handles reading chars vs reading events for key pressed indipendently
    // This means that for instance if you press shift + r.
    // You'll catch 'R' when reading characteres but not when reading keypress events.
    // This behavior also makes impossible to get symbols such as !@#$%^&*()_+ thefore we manually handle it
    // Keyboard codes  here: http://www.philipstorr.id.au/pcbook/book3/scancode.htm
    // Control codes here: https://www.windmill.co.uk/ascii-control-codes.html
    //  ASCII codes here: http://www.asciitable.com/
    fn map_to_key(&self, c: char) -> KeyEvent {
        let numc = c as usize;
        if numc == K_ENTER {
            return KeyEvent::Enter;
        } else if numc == K_DELETE {
            return KeyEvent::Delete;
        } else if numc == K_BACKSPACE {
            return KeyEvent::Backspace;
        } else if numc == K_TAB {
            return KeyEvent::Tab;
        } else if numc == K_PAGEDOWN {
            return KeyEvent::PageDown;
        } else if numc == K_PAGEUP {
            return KeyEvent::PageUp;
        } else if numc == K_HOME {
            return KeyEvent::Home;
        } else if numc == K_END {
            return KeyEvent::End;
        } else if numc == K_ARROW_UP {
            return KeyEvent::Up;
        } else if numc == K_ARROW_DOWN {
            return KeyEvent::Down;
        } else if numc == K_ARROW_LEFT {
            return KeyEvent::Left;
        } else if numc == K_ARROW_RIGHT {
            return KeyEvent::Right;
        }else if numc == K_ESCAPE {
            return KeyEvent::Esc;
        }
        else if self.is_control_key(c) {
            KeyEvent::Ctrl((c as u8 + 0x40) as char)
        } else {
            KeyEvent::Char(c)
        }
    }
}
