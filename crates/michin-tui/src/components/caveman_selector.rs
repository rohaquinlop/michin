//! Caveman mode selector overlay — shown on `/caveman` (no args) to pick
//! a compression level. Each level shows a short description.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::theme::Theme;

/// A caveman level entry in the selector.
#[derive(Debug, Clone)]
pub struct CavemanLevelEntry {
    pub id: String,
    pub label: String,
}

/// Caveman mode selector overlay component.
pub struct CavemanSelector {
    /// Available levels (always all 7, no runtime fetch).
    levels: Vec<CavemanLevelEntry>,
    /// Currently selected index.
    selected: usize,
    /// List state.
    list_state: ListState,
    /// Theme.
    theme: Theme,
    /// Whether to show the selector.
    pub visible: bool,
    /// Whether user confirmed (Enter) or cancelled (Esc).
    pub confirmed: bool,
}

impl CavemanSelector {
    pub fn new(theme: Theme) -> Self {
        let levels = vec![
            CavemanLevelEntry {
                id: "off".to_string(),
                label: "Disable caveman — normal verbose responses".to_string(),
            },
            CavemanLevelEntry {
                id: "lite".to_string(),
                label: "No filler/hedging. Keep articles + full sentences. Professional but tight."
                    .to_string(),
            },
            CavemanLevelEntry {
                id: "full".to_string(),
                label: "Drop articles, fragments OK, short synonyms. Classic caveman.".to_string(),
            },
            CavemanLevelEntry {
                id: "ultra".to_string(),
                label: "Abbreviate prose words, arrows for causality. Never abbreviate code."
                    .to_string(),
            },
            CavemanLevelEntry {
                id: "wenyan-lite".to_string(),
                label: "Semi-classical Chinese register. Drop filler but keep grammar.".to_string(),
            },
            CavemanLevelEntry {
                id: "wenyan-full".to_string(),
                label: "Maximum classical Chinese terseness. 之/乃/為/其 particles.".to_string(),
            },
            CavemanLevelEntry {
                id: "wenyan-ultra".to_string(),
                label: "Extreme abbreviation with classical Chinese feel.".to_string(),
            },
        ];
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            levels,
            selected: 0,
            list_state,
            theme,
            visible: false,
            confirmed: false,
        }
    }

    /// Show the selector, pre-selecting the current level if active.
    pub fn show(&mut self, current: Option<&str>) {
        self.visible = true;
        self.confirmed = false;
        self.selected = current
            .and_then(|c| self.levels.iter().position(|l| l.id == c))
            .unwrap_or(0);
        self.list_state.select(Some(self.selected));
    }

    /// Hide the selector.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Get the selected level ID, if any.
    pub fn selected_level(&self) -> Option<&str> {
        self.levels.get(self.selected).map(|e| e.id.as_str())
    }

    /// Move selection up.
    pub fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    /// Move selection down.
    pub fn select_down(&mut self) {
        self.selected = (self.selected + 1).min(self.levels.len().saturating_sub(1));
        self.list_state.select(Some(self.selected));
    }

    /// Render the selector overlay.
    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        // Center the overlay on screen.
        let overlay_width = area.width.min(64);
        let overlay_height = area.height.min(18);
        let overlay_x = (area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (area.height.saturating_sub(overlay_height)) / 2;

        let overlay = Rect {
            x: area.x + overlay_x,
            y: area.y + overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(self.theme.bg))
            .border_style(Style::default().fg(self.theme.accent))
            .title(" Caveman Mode ");

        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        // Level list — two lines per entry: id + description, then blank spacer.
        let items: Vec<ListItem> = self
            .levels
            .iter()
            .map(|entry| {
                ListItem::new(ratatui::text::Text::from(vec![
                    ratatui::text::Line::from(Span::raw(format!(
                        "  {}  {}",
                        entry.id, entry.label
                    ))),
                    ratatui::text::Line::from(Span::raw("")),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().fg(self.theme.accent).bg(Color::DarkGray))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Footer help.
        let help = Paragraph::new(Span::styled(
            "Up/Down move | Enter select | Esc close",
            Style::default().fg(self.theme.dim),
        ));
        frame.render_widget(help, chunks[1]);
    }
}
