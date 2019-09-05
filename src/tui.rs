/// Terminal user interface (TUI) functionality, with ncurses-like API,
/// built on top of the termion crate.

use crate::config::Config;
use log::{error, warn};
use std::io::{Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style};

/// A wrapper around the terminal for creating a window.
pub struct Window {
    /// Key input from Stdin.
    stdin: Keys<Stdin>,
    /// Stdout, with terminal in raw-mode (no input line buffering, no echo).
    stdout: RawTerminal<Stdout>,
    /// Yat configuration.
    pub config: Config,
}

impl Drop for Window {
    /// Ensure the terminal is reset if the Window is dropped.
    fn drop(&mut self) {
        self.endwin();
        self.show_cursor();
    }
}

impl Window {
    /// Create a new Window, using terminal's stdin and stdout.
    pub fn new(stdin: Stdin, stdout: Stdout, config: Config) -> Result<Window, ()> {
        let raw = match stdout.into_raw_mode() {
            Ok(out) => out,
            Err(_) => {
                error!("Unable to set terminal to raw mode.");
                return Err(());
            }
        };
        Ok(Window {
            stdin: stdin.keys(),
            stdout: raw,
            config,
        })
    }

    /// Find the terminal's dimensions.
    pub fn get_max_yx(&self) -> (usize, usize) {
        let (y, x) = termion::terminal_size().unwrap_or_else(|err| {
            warn!("Unable to determine terminal size: {}.", err);
            (0, 0)
        });
        (x as usize, y as usize)
    }

    /// Hide cursor from terminal.
    pub fn hide_cursor(&mut self) {
        write!(self.stdout, "{}", cursor::Hide).unwrap_or_else(|err| {
            warn!("Unable to hide cursor: {}.", err);
        });
    }

    /// Display cursor on terminal.
    pub fn show_cursor(&mut self) {
        write!(self.stdout, "{}", cursor::Show).unwrap_or_else(|err| {
            warn!("Unable to show cursor: {}", err);
        });
    }

    /// Flush stdout buffer to terminal.
    pub fn refresh(&mut self) {
        self.stdout.flush().unwrap_or_else(|err| {
            warn!("Unable to flush stdout: {}", err);
        });
    }

    /// Return the key input from stdin.
    pub fn getch(&mut self) -> Option<Key> {
        match self.stdin.next() {
            Some(Ok(key)) => Some(key),
            _ => None,
        }
    }

    /// Move the cursor to position at row y, column x (zero-indexed).
    pub fn mv(&mut self, y: usize, x: usize) {
        write!(self.stdout, "{}", cursor::Goto(1 + x as u16, 1 + y as u16)).unwrap_or_else(|err| {
            warn!("Unable to mv cursor: {}", err);
        });
    }

    /// Add colour to subsequent printed text.
    pub fn colour_on(&mut self, fg: usize, bg: usize) {
        let fgcol = match fg {
            0 => self.config.colour0.fg(),
            1 => self.config.colour1.fg(),
            2 => self.config.colour2.fg(),
            3 => self.config.colour3.fg(),
            4 => self.config.colour4.fg(),
            5 => self.config.colour5.fg(),
            6 => self.config.colour6.fg(),
            7 => self.config.colour7.fg(),
            8 => self.config.colourfg.fg(),
            _ => return (),
        };

        let bgcol = match bg {
            0 => self.config.colour0.bg(),
            1 => self.config.colour1.bg(),
            2 => self.config.colour2.bg(),
            3 => self.config.colour3.bg(),
            4 => self.config.colour4.bg(),
            5 => self.config.colour5.bg(),
            6 => self.config.colour6.bg(),
            7 => self.config.colour7.bg(),
            8 => self.config.colourbg.bg(),
            _ => return (),
        };
        
        write!(self.stdout, "{}{}", fgcol, bgcol).unwrap_or_else(|err| {
            warn!("Unable to turn colour on: {}", err);
        });
    }

    /// Reset colours to default foreground and background.
    pub fn colour_off(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            self.config.colourfg.fg(),
            self.config.colourbg.bg()
        )
        .unwrap_or_else(|err| {
            warn!("Unable to turn colour off: {}", err);
        });
    }

    /// Reset colours to terminal defaults.
    pub fn colour_reset(&mut self) {
        write!(
            self.stdout,
            "{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset)
        )
        .unwrap_or_else(|err| {
            warn!("Unable to turn colour off: {}", err);
        });
    }

    /// Print text at row y, column x (zero-indexed).
    pub fn mvprintw(&mut self, y: usize, x: usize, text: &str) {
        write!(
            self.stdout,
            "{}{}",
            cursor::Goto(1 + x as u16, 1 + y as u16),
            text
        )
        .unwrap_or_else(|err| {
            warn!("Unable to mvprintw: {}", err);
        });
    }

    /// Print text at row y, column x (zero-indexed), truncated to ensure
    /// the text does not spill beyond width.
    pub fn wrap_print(&mut self, y: usize, x: usize, width: usize, text: &str) {
        let len = text.len();
        let wid = width as usize - 3;
        let limit = if len > wid { wid } else { len };
        self.mvprintw(y, x, &text[..limit]);
        if len > wid {
            self.mvprintw(y, x + width - 3, "...");
        }
    }

    /// Print a rectangular border.
    pub fn border(&mut self, lower_left: (usize, usize), dimensions: (usize, usize)) {
        let (y, x) = lower_left;
        let (height, width) = dimensions;

        self.mvprintw(y + 1 - height, x, &self.config.ulcorner.clone());
        self.mvprintw(y, x, &self.config.llcorner.clone());

        self.mvprintw(y + 1 - height, x + width - 1, &self.config.urcorner.clone());
        self.mvprintw(y, x + width - 1, &self.config.lrcorner.clone());

        for j in (y + 2 - height)..y {
            self.mvprintw(j, x, &self.config.vline.clone());
            self.mvprintw(j, x + width - 1, &self.config.vline.clone());
        }

        for i in (x + 1)..(x + width - 1) {
            self.mvprintw(y, i, &self.config.hline.clone());
            self.mvprintw(y + 1 - height, i, &self.config.hline.clone());
        }
    }

    /// Fill a rectangular region with character ch.
    pub fn rectangle(&mut self, ch: &str, lower_left: (usize, usize), dimensions: (usize, usize)) {
        let (y, x) = lower_left;
        let (height, width) = dimensions;

        for j in (y - height + 1)..y {
            for i in x..(x + width - 1) {
                self.mvprintw(j, i, ch);
                self.mvprintw(j, i + width - 1, ch);
            }
        }
    }

    /// Clear stdout.
    pub fn clear(&mut self) {
        write!(self.stdout, "{}", clear::All).unwrap_or_else(|err| {
            warn!("Unable to clear stdout: {}", err);
        });
    }

    /// Reset stdout.
    pub fn endwin(&mut self) {
        self.colour_reset();
        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            style::Reset,
            cursor::Goto(1, 1)
        )
        .unwrap_or_else(|err| {
            warn!("Unable to endwin: {}", err);
        });
    }
}
