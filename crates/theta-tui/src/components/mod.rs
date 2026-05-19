//! TUI components.

use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};

pub mod chat;
pub mod editor;
pub mod login_flow;
pub mod model_selector;
pub mod session_picker;
pub mod status;

pub use login_flow::LoginFlow;
pub use model_selector::{ModelEntry, ModelSelector};
pub use session_picker::{SessionInfo, SessionPicker};

/// Actions that components can request from the App.
#[derive(Debug, Clone)]
pub enum Action {
    SendMessage(String),
    Quit,
    SwitchModel(String),
    SetThinking(String),
    ClearChat,
    SessionInfo,
    ForkSession,
    ShowHelp,
    ShowModelSelector,
    None,
}

/// A renderable TUI component.
pub trait Component: Send {
    fn render(&mut self, area: Rect, frame: &mut Frame);
    fn handle_event(&mut self, event: &Event) -> Option<Action>;
    fn is_focused(&self) -> bool;
    fn focus(&mut self, focused: bool);
}
