//! Path picker overlay — @ file/directory completion in the editor.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::PathBuf;

use crate::theme::Theme;

/// A file system entry for the picker.
#[derive(Debug, Clone)]
pub struct PathEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    /// Display name (relative to current dir).
    pub display: String,
    /// Relative path from working_dir for insertion.
    pub rel_path: String,
}

/// Path picker overlay for @ file/directory completion.
pub struct PathPicker {
    /// All matching entries in current directory.
    entries: Vec<PathEntry>,
    /// Currently selected index.
    selected: usize,
    /// List state.
    list_state: ListState,
    /// Theme.
    theme: Theme,
    /// Root working directory (never changes).
    working_dir: PathBuf,
    /// Current directory being browsed.
    current_dir: PathBuf,
    /// Current filter query (filters names in current_dir).
    query: String,
    /// Whether the picker is visible.
    pub visible: bool,
    /// Whether the user confirmed a selection.
    pub confirmed: bool,
    /// The selected path for insertion (relative to working_dir).
    pub selected_path: Option<String>,
}

impl PathPicker {
    pub fn new(theme: Theme, working_dir: PathBuf) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            entries: Vec::new(),
            selected: 0,
            list_state,
            theme,
            current_dir: working_dir.clone(),
            working_dir,
            query: String::new(),
            visible: false,
            confirmed: false,
            selected_path: None,
        }
    }

    /// Show the picker, resetting to the working directory root.
    pub fn show(&mut self) {
        self.visible = true;
        self.confirmed = false;
        self.selected_path = None;
        self.query.clear();
        self.current_dir = self.working_dir.clone();
        self.populate();
    }

    /// Hide the picker.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Push a character to the filter query.
    pub fn push_query(&mut self, c: char) {
        self.query.push(c);
        self.populate();
    }

    /// Remove last character from filter query.
    pub fn pop_query(&mut self) {
        if self.query.is_empty() {
            // Go up to parent directory if possible.
            if self.current_dir != self.working_dir
                && let Some(parent) = self.current_dir.parent()
            {
                self.current_dir = parent.to_path_buf();
                self.populate();
            }
        } else {
            self.query.pop();
            self.populate();
        }
    }

    /// Enter the selected directory.
    pub fn enter_directory(&mut self) {
        if let Some(entry) = self.entries.get(self.selected)
            && entry.is_dir
        {
            self.current_dir = entry.path.clone();
            self.query.clear();
            self.populate();
        }
    }

    /// Populate entries from current directory matching the query.
    fn populate(&mut self) {
        self.entries.clear();
        let query_lower = self.query.to_lowercase();

        // Add ".." entry if not at working_dir root.
        if self.current_dir != self.working_dir {
            self.entries.push(PathEntry {
                path: self
                    .current_dir
                    .parent()
                    .unwrap_or(&self.working_dir)
                    .to_path_buf(),
                is_dir: true,
                display: "..".into(),
                rel_path: String::new(), // not selectable as file
            });
        }

        let mut found: Vec<PathEntry> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                // Skip hidden files unless query starts with .
                if name.starts_with('.') && !self.query.starts_with('.') {
                    continue;
                }
                // Filter by query.
                if !self.query.is_empty() && !name.to_lowercase().contains(&query_lower) {
                    continue;
                }
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                let rel_path = self
                    .current_dir
                    .strip_prefix(&self.working_dir)
                    .map(|rel| {
                        if rel.as_os_str().is_empty() {
                            name.clone()
                        } else {
                            format!("{}/{}", rel.display(), name)
                        }
                    })
                    .unwrap_or_else(|_| name.clone());

                found.push(PathEntry {
                    path: entry.path(),
                    is_dir,
                    display: if is_dir {
                        format!("{name}/")
                    } else {
                        name.clone()
                    },
                    rel_path,
                });
            }
        }

        // Sort: directories first (after ".."), then alphabetical.
        found.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.display.to_lowercase().cmp(&b.display.to_lowercase()))
        });

        self.entries.extend(found);
        self.selected = 0;
        if self.entries.is_empty() || (self.entries.len() == 1 && self.entries[0].display == "..")
        {
            self.list_state.select(if self.entries.is_empty() {
                None
            } else {
                Some(0)
            });
        } else {
            // Skip ".." selection if there are real entries.
            let start = self.entries.iter().position(|e| e.display != "..").unwrap_or(0);
            self.selected = start;
            self.list_state.select(Some(start));
        }
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

    /// Check if the selected entry is a directory (for Enter handling in app).
    pub fn selected_is_dir(&self) -> bool {
        self.entries
            .get(self.selected)
            .map(|e| e.is_dir)
            .unwrap_or(false)
    }

    /// Get the selected path for insertion as @path.
    pub fn take_selection(&mut self) -> Option<String> {
        self.confirmed = true;
        self.visible = false;
        let entry = self.entries.get(self.selected)?;
        if entry.display == ".." {
            return None;
        }
        Some(entry.rel_path.clone())
    }

    /// Render the picker overlay.
    pub fn render(&mut self, area: Rect, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        // Show current directory path relative to working_dir.
        let dir_label = if self.current_dir == self.working_dir {
            ".".to_string()
        } else {
            self.current_dir
                .strip_prefix(&self.working_dir)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| self.current_dir.display().to_string())
        };

        let title = if self.query.is_empty() {
            format!(" @{dir_label}/  (enter=open dir, tab=select file, esc=cancel) ")
        } else {
            format!(" @{dir_label} | {}  (esc=cancel) ", self.query)
        };

        // Position just above the input area.
        let overlay_height = area.height.min(12);
        let overlay_width = area.width.min(50);
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

        // File list.
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|e| {
                let prefix = if e.display == ".." { "  .." } else { "  " };
                ListItem::new(Line::from(Span::raw(format!("{prefix}{}", e.display))))
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

        // Footer.
        let count = self.entries.len();
        let footer = Paragraph::new(Span::styled(
            format!(
                "{count} entries  arrows=navigate  enter=open dir  tab=select  esc=cancel"
            ),
            Style::default().fg(self.theme.dim),
        ));
        frame.render_widget(footer, chunks[1]);
    }
}
