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
    pub turn_index: u32,
    pub show_diagnostics: bool,
    spinner_idx: usize,
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
            turn_index: 0,
            show_diagnostics: false,
            spinner_idx: 0,
            theme,
        }
    }

    pub fn set_agent_state(&mut self, state: &str) {
        self.agent_state = state.to_string();
    }

    pub fn set_tool_progress(&mut self, progress: &str) {
        self.tool_progress = progress.to_string();
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_turn_index(&mut self, turn_index: u32) {
        self.turn_index = turn_index;
    }

    pub fn set_show_diagnostics(&mut self, show: bool) {
        self.show_diagnostics = show;
    }
}

impl Component for StatusBar {
    fn render(&mut self, area: Rect, frame: &mut Frame) {
        let total_width = area.width as usize;
        let model_str = short_middle(&self.model, 24);
        let thinking_str = format!(" | thinking: {}", self.thinking);

        let left = vec![
            Span::styled(model_str, Style::default().fg(self.theme.accent)),
            Span::styled(thinking_str, Style::default().fg(self.theme.dim)),
        ];

        let state_color = if self.agent_state.starts_with("error")
            || self.agent_state.starts_with("tool error")
        {
            self.theme.error
        } else if self.agent_state.starts_with("streaming")
            || self.agent_state.starts_with("thinking")
            || self.agent_state.starts_with("tool")
            || self.agent_state.starts_with("compacting")
            || self.agent_state.starts_with("retrying")
        {
            self.theme.warning
        } else {
            self.theme.success
        };

        let mode = mode_from_state(&self.agent_state);
        let active = matches!(mode, "thinking" | "tool" | "retry" | "stream");
        let spinner = if active {
            const FRAMES: [&str; 4] = ["|", "/", "-", "\\"];
            let frame = FRAMES[self.spinner_idx % FRAMES.len()];
            self.spinner_idx = self.spinner_idx.wrapping_add(1);
            format!(" {frame}")
        } else {
            String::new()
        };
        let step = if self.tool_progress.is_empty() {
            "-".to_string()
        } else {
            self.tool_progress.clone()
        };
        let mut right_text = if self.show_diagnostics {
            format!(
                "[mode: {mode}] [step: {step}] [turn: {}]{spinner}",
                self.turn_index
            )
        } else if step == "-" {
            format!("[{mode}]{spinner}")
        } else {
            format!("[{mode}] {step}{spinner}")
        };
        right_text = truncate_chars(&right_text, total_width.saturating_div(2).max(12));

        let right = vec![Span::styled(right_text, Style::default().fg(state_color))];

        // Pad to fill width.
        let left_str: String = left.iter().map(|s| s.content.as_ref()).collect();
        let right_str: String = right.iter().map(|s| s.content.as_ref()).collect();

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

fn mode_from_state(state: &str) -> &str {
    if state.starts_with("retrying") {
        "retry"
    } else if state.starts_with("tool") {
        "tool"
    } else if state.starts_with("thinking") {
        "thinking"
    } else if state.starts_with("streaming") {
        "stream"
    } else if state.starts_with("error") {
        "error"
    } else {
        "idle"
    }
}

fn short_middle(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars || max_chars < 5 {
        return text.to_string();
    }
    let head_len = (max_chars - 3) / 2;
    let tail_len = max_chars - 3 - head_len;
    let head: String = text.chars().take(head_len).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(tail_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    let mut chars = text.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
