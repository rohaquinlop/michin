//! Command picker overlay — / slash command completion in the editor.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::theme::Theme;

/// A command or skill entry.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    /// The text to insert (e.g., "help", "model gpt-5.5").
    pub name: String,
    /// Short description.
    pub description: String,
}

/// Command picker overlay for / completion.
pub struct CommandPicker {
    /// All matching entries.
    entries: Vec<CommandEntry>,
    /// Currently selected index.
    selected: usize,
    /// List state.
    list_state: ListState,
    /// Theme.
    theme: Theme,
    /// Current filter query.
    query: String,
    /// Whether the picker is visible.
    pub visible: bool,
    /// Whether the user confirmed a selection.
    pub confirmed: bool,
    /// The selected command text for insertion.
    pub selected_command: Option<String>,
}

impl CommandPicker {
    pub fn new(theme: Theme) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            entries: Vec::new(),
            selected: 0,
            list_state,
            theme,
            query: String::new(),
            visible: false,
            confirmed: false,
            selected_command: None,
        }
    }

    /// Show the picker with a list of available commands.
    pub fn show(&mut self, entries: Vec<CommandEntry>) {
        self.visible = true;
        self.confirmed = false;
        self.selected_command = None;
        self.query.clear();
        self.entries = entries;
        self.selected = 0;
        if self.entries.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Push a character to the filter query.
    pub fn push_query(&mut self, c: char) {
        self.query.push(c);
    }

    /// Remove last character from filter query.
    pub fn pop_query(&mut self) {
        self.query.pop();
    }

    /// Move selection up.
    pub fn select_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    /// Move selection down.
    pub fn select_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.entries.len().saturating_sub(1));
        self.list_state.select(Some(self.selected));
    }

    /// Get the selected command for insertion.
    pub fn take_selection(&mut self) -> Option<String> {
        self.confirmed = true;
        self.visible = false;
        self.entries
            .get(self.selected)
            .map(|e| e.name.clone())
    }

    /// Return filtered entries matching the query.
    fn filtered(&self) -> Vec<&CommandEntry> {
        let q = self.query.to_lowercase();
        if q.is_empty() {
            self.entries.iter().collect()
        } else {
            self.entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&q))
                .collect()
        }
    }

    /// Render the picker overlay.
    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let title = if self.query.is_empty() {
            " /  (type to filter, enter/tab to insert, esc to cancel) ".to_string()
        } else {
            format!(" /{}  (esc to cancel) ", self.query)
        };

        // Position just above the input area.
        let overlay_height = area.height.min(14);
        let overlay_width = area.width.min(55);
        let overlay_x = 2;
        let overlay_y = area.height.saturating_sub(overlay_height + 4).max(2);

        let overlay = Rect {
            x: area.x + overlay_x,
            y: area.y + overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent))
            .title(title);

        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        let filtered = self.filtered();
        let count = filtered.len();
        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let is_selected = i == self.selected;
                let line = format!(
                    "  {:<20} {}",
                    format!("/{}", e.name),
                    e.description
                );
                let style = if is_selected {
                    Style::default()
                } else {
                    Style::default().fg(self.theme.dim)
                };
                ListItem::new(Line::from(Span::styled(line, style)))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(self.theme.accent)
                    .bg(Color::DarkGray),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        let footer = Paragraph::new(Span::styled(
            format!("{count} commands  arrows=navigate  enter/tab=select  esc=cancel"),
            Style::default().fg(self.theme.dim),
        ));
        frame.render_widget(footer, chunks[1]);
    }
}
