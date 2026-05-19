//! Chat message display — scrollable conversation view.

use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::components::{Action, Component};
use crate::theme::Theme;

/// A single chat message to display.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
    pub tool_name: Option<String>,
    pub is_streaming: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    Tool,
    System,
}

/// Scrollable chat message list.
pub struct Chat {
    pub messages: Vec<ChatMessage>,
    scroll: usize,
    focused: bool,
    theme: Theme,
}

impl Chat {
    pub fn new(theme: Theme) -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
            focused: false,
            theme,
        }
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }

    pub fn update_last(&mut self, text: &str, role: ChatRole, is_streaming: bool) {
        if let Some(last) = self.messages.last_mut()
            && last.role == role && last.is_streaming {
                last.text.push_str(text);
                last.is_streaming = is_streaming;
                return;
            }
        self.messages.push(ChatMessage {
            role,
            text: text.to_string(),
            tool_name: None,
            is_streaming,
        });
    }

    pub fn finish_last(&mut self, role: ChatRole) {
        if let Some(last) = self.messages.last_mut()
            && last.role == role {
                last.is_streaming = false;
            }
    }

    fn format_line(&self, msg: &ChatMessage) -> Line<'static> {
        let (prefix_text, style) = match msg.role {
            ChatRole::User => ("> ".to_string(), Style::default().fg(self.theme.accent)),
            ChatRole::Assistant => (
                "\u{2713} ".to_string(),
                Style::default().fg(self.theme.success),
            ),
            ChatRole::Tool => (
                format!("\u{2699} {} ", msg.tool_name.as_deref().unwrap_or("tool")),
                Style::default().fg(self.theme.warning),
            ),
            ChatRole::System => ("# ".to_string(), Style::default().fg(self.theme.dim)),
        };

        let body = if msg.role == ChatRole::Tool {
            truncate_output(&msg.text, 500)
        } else {
            msg.text.clone()
        };

        let suffix = if msg.is_streaming {
            Span::styled("\u{258c}", Style::default().fg(self.theme.accent))
        } else {
            Span::raw("")
        };

        Line::from(vec![
            Span::styled(prefix_text, style),
            Span::raw(body),
            suffix,
        ])
    }
}

impl Component for Chat {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        let title = if self.focused {
            " Chat (j/k scroll) "
        } else {
            " Chat "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border))
            .title(title)
            .title_style(Style::default().fg(self.theme.accent));

        let lines: Vec<Line> = self
            .messages
            .iter()
            .rev()
            .skip(self.scroll)
            .flat_map(|msg| {
                let mut out = vec![Line::raw("")];
                out.push(self.format_line(msg));
                out
            })
            .collect();

        let para = Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .block(block);

        frame.render_widget(para, area);
    }

    fn handle_event(&mut self, event: &Event) -> Option<Action> {
        if !self.focused {
            return None;
        }
        let Event::Key(key) = event else {
            return None;
        };

        match key.code {
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down
                if self.scroll > 0 => {
                    self.scroll -= 1;
                }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                self.scroll = self.scroll.saturating_add(1);
            }
            crossterm::event::KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            crossterm::event::KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_add(10);
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

fn truncate_output(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}... ({} chars total)", &text[..max_len], text.len())
    }
}
