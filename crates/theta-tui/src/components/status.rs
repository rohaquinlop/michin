//! Status bar component.

use crossterm::event::Event;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::components::{Action, Component};
use crate::theme::Theme;

pub struct StatusBar {
    pub model: String,
    pub session_id: String,
    pub thinking: String,
    pub agent_state: String,
    pub tool_progress: String,
    theme: Theme,
}

impl StatusBar {
    pub fn new(theme: Theme) -> Self {
        Self {
            model: String::new(),
            session_id: String::new(),
            thinking: String::new(),
            agent_state: String::new(),
            tool_progress: String::new(),
            theme,
        }
    }

    pub fn set_agent_state(&mut self, state: &str) {
        self.agent_state = state.to_string();
    }

    pub fn set_tool_progress(&mut self, progress: &str) {
        self.tool_progress = progress.to_string();
    }
}

impl Component for StatusBar {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        let model_str = self.model.clone();
        let thinking_str = format!(" | thinking: {}", self.thinking);
        let session_str = format!(" | session: {}", self.session_id);

        let left = vec![
            Span::styled(model_str, Style::default().fg(self.theme.accent)),
            Span::styled(thinking_str, Style::default().fg(self.theme.dim)),
            Span::styled(session_str, Style::default().fg(self.theme.dim)),
        ];

        let state_color = match self.agent_state.as_str() {
            "streaming" | "tool executing" => self.theme.warning,
            "error" => self.theme.error,
            _ => self.theme.success,
        };

        let right_text = if self.tool_progress.is_empty() {
            format!("[{}]", self.agent_state)
        } else {
            format!("[{}] {}", self.agent_state, self.tool_progress)
        };

        let right = vec![Span::styled(right_text, Style::default().fg(state_color))];

        // Pad to fill width.
        let left_str: String = left.iter().map(|s| s.content.as_ref()).collect();
        let right_str: String = right.iter().map(|s| s.content.as_ref()).collect();
        let total_width = area.width as usize;

        let pad = if left_str.len() + right_str.len() < total_width {
            " ".repeat(total_width - left_str.len() - right_str.len())
        } else {
            " ".to_string()
        };

        let mut spans = left;
        spans.push(Span::raw(pad));
        spans.extend(right);

        let para =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));
        frame.render_widget(para, area);
    }

    fn handle_event(&mut self, _event: &Event) -> Option<Action> {
        None
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn focus(&mut self, _focused: bool) {}
}
