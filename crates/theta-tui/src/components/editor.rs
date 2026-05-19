//! Input editor component — multiline text input with history.

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::components::{Action, Component};
use crate::theme::Theme;

/// Multiline text editor for user input.
pub struct Editor {
    /// The text buffer.
    text: String,
    /// Cursor position (byte offset).
    cursor: usize,
    /// Whether focused.
    focused: bool,
    /// Theme.
    #[allow(dead_code)]
    theme: Theme,
    /// History of submitted messages.
    history: Vec<String>,
    /// Current history index (for up/down browsing).
    history_idx: usize,
    /// Temporary save for history browsing.
    saved_text: String,
}

impl Editor {
    pub fn new(theme: Theme) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            focused: false,
            theme,
            history: Vec::new(),
            history_idx: 0,
            saved_text: String::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    fn delete_before(&mut self) {
        if self.cursor > 0
            && let Some(prev) = self.text[..self.cursor].chars().last()
        {
            let len = prev.len_utf8();
            self.text.replace_range(self.cursor - len..self.cursor, "");
            self.cursor -= len;
        }
    }

    fn delete_after(&mut self) {
        if self.cursor < self.text.len()
            && let Some(next) = self.text[self.cursor..].chars().next()
        {
            self.text
                .replace_range(self.cursor..self.cursor + next.len_utf8(), "");
        }
    }

    fn move_left(&mut self) {
        if self.cursor > 0
            && let Some(prev) = self.text[..self.cursor].chars().last()
        {
            self.cursor -= prev.len_utf8();
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.text.len()
            && let Some(next) = self.text[self.cursor..].chars().next()
        {
            self.cursor += next.len_utf8();
        }
    }

    fn move_word_left(&mut self) {
        while self.cursor > 0 {
            if let Some(prev) = self.text[..self.cursor].chars().last() {
                if prev.is_whitespace() {
                    self.move_left();
                } else {
                    break;
                }
            }
        }
        while self.cursor > 0 {
            if let Some(prev) = self.text[..self.cursor].chars().last() {
                if !prev.is_whitespace() {
                    self.move_left();
                } else {
                    break;
                }
            }
        }
    }

    #[allow(dead_code)]
    fn move_word_right(&mut self) {
        while self.cursor < self.text.len() {
            if let Some(next) = self.text[self.cursor..].chars().next() {
                if !next.is_whitespace() {
                    self.move_right();
                } else {
                    break;
                }
            }
        }
        while self.cursor < self.text.len() {
            if let Some(next) = self.text[self.cursor..].chars().next() {
                if next.is_whitespace() {
                    self.move_right();
                } else {
                    break;
                }
            }
        }
    }

    fn move_start(&mut self) {
        self.cursor = 0;
    }

    fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    fn submit(&mut self) -> Option<String> {
        let text = self.text.trim().to_string();
        self.text.clear();
        self.cursor = 0;
        if text.is_empty() {
            return None;
        }
        self.history.push(text.clone());
        self.history_idx = self.history.len();
        Some(text)
    }

    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_idx == self.history.len() {
            self.saved_text = self.text.clone();
        }
        if self.history_idx > 0 {
            self.history_idx -= 1;
            self.text = self.history[self.history_idx].clone();
            self.cursor = self.text.len();
        }
    }

    fn history_down(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_idx < self.history.len() - 1 {
            self.history_idx += 1;
            self.text = self.history[self.history_idx].clone();
            self.cursor = self.text.len();
        } else if self.history_idx == self.history.len() - 1 {
            self.history_idx += 1;
            self.text = self.saved_text.clone();
            self.cursor = self.text.len();
        }
    }
}

impl Component for Editor {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        let title = if self.focused {
            " Input (Enter to send, Alt+Enter for newline) "
        } else {
            " Input "
        };

        let cursor_style = if self.focused {
            Style::default().fg(self.theme.accent).bg(Color::DarkGray)
        } else {
            Style::default().fg(self.theme.dim)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border))
            .title(title)
            .title_style(if self.focused {
                Style::default().fg(self.theme.accent)
            } else {
                Style::default().fg(self.theme.dim)
            });

        // Build text with cursor indicator.
        let mut spans = Vec::new();
        for (i, c) in self.text.char_indices() {
            let at_cursor = self.focused && i == self.cursor;
            spans.push(Span::styled(
                c.to_string(),
                if at_cursor {
                    cursor_style
                } else {
                    Style::default()
                },
            ));
        }
        if self.focused && self.cursor >= self.text.len() {
            spans.push(Span::styled(" ", cursor_style));
        }

        let para = Paragraph::new(Line::from(spans)).block(block);

        frame.render_widget(para, area);
    }

    fn handle_event(&mut self, event: &Event) -> Option<Action> {
        if !self.focused {
            return None;
        }
        let Event::Key(key) = event else {
            return None;
        };

        match key {
            // Enter on its own = submit (not Alt+Enter)
            crossterm::event::KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if let Some(text) = self.submit() {
                    return Some(Action::SendMessage(text));
                }
            }
            // Alt+Enter inserts a literal newline.
            crossterm::event::KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::ALT,
                ..
            } => {
                self.insert_char('\n');
            }
            // Ctrl+J also inserts a newline (common terminal binding)
            crossterm::event::KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.insert_char('\n');
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                ..
            } => {
                self.insert_char(*c);
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                self.delete_before();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                self.delete_after();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                self.move_left();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                self.move_right();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.history_up();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.history_down();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                self.move_start();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::End, ..
            } => {
                self.move_end();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.move_start();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.move_end();
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                // Delete word backward.
                let old = self.cursor;
                self.move_word_left();
                let new = self.cursor;
                self.text.replace_range(new..old, "");
            }
            crossterm::event::KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.insert_char('\t');
            }
            _ => {}
        }
        None
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}
