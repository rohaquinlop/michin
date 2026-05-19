//! Application — main TUI event loop and layout management.

use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use tokio::sync::mpsc;

use crate::components::chat::{Chat, ChatMessage, ChatRole};
use crate::components::editor::Editor;
use crate::components::login_flow::{LoginFlow, known_providers};
use crate::components::status::StatusBar;
use crate::components::{Action, Component};
use crate::keybinding::{Keybinding, default_bindings, resolve_event};
use crate::terminal;
use crate::theme::Theme;

/// Commands sent from the TUI back to the interactive handler.
#[derive(Debug, Clone)]
pub enum TuiAction {
    /// Switch to a different model.
    SwitchModel(String),
    /// Change thinking level.
    SetThinking(String),
    /// Fork the current session.
    ForkSession,
    /// Login result from the login flow.
    LoginResult { provider: String, token: String },
}

/// Events sent from the agent loop to the TUI.
#[derive(Debug, Clone)]
pub enum TuiEvent {
    TextDelta(String),
    ThinkingDelta(String),
    ToolStart { name: String, id: String },
    ToolProgress { name: String, message: String },
    ToolEnd { name: String, output: String },
    TurnStart,
    TurnEnd { stop_reason: String },
    AgentEnd,
    Error(String),
}

/// The main TUI application.
pub struct App {
    chat: Chat,
    editor: Editor,
    status: StatusBar,
    keybindings: Vec<Keybinding>,
    focus_idx: usize,
    running: bool,
    /// Send user messages to the agent.
    pub message_tx: mpsc::UnboundedSender<String>,
    /// Send structured actions back to the interactive handler.
    pub action_tx: mpsc::UnboundedSender<TuiAction>,
    /// Receive TUI events from the agent.
    pub event_rx: mpsc::UnboundedReceiver<TuiEvent>,
    #[allow(dead_code)]
    theme: Theme,
    streaming: bool,
    current_tool: Option<String>,
    /// Active login flow (replaces chat+editor when set).
    login_flow: Option<LoginFlow>,
}

impl App {
    pub fn new(
        theme: Theme,
        model: &str,
        session_id: &str,
        thinking: &str,
        event_rx: mpsc::UnboundedReceiver<TuiEvent>,
        message_tx: mpsc::UnboundedSender<String>,
        action_tx: mpsc::UnboundedSender<TuiAction>,
    ) -> Self {
        let mut status = StatusBar::new(theme.clone());
        status.model = model.to_string();
        status.session_id = session_id.to_string();
        status.thinking = thinking.to_string();
        status.set_agent_state("idle");

        Self {
            chat: Chat::new(theme.clone()),
            editor: Editor::new(theme.clone()),
            status,
            theme: theme.clone(),
            keybindings: default_bindings(),
            focus_idx: 0,
            running: true,
            message_tx,
            action_tx,
            event_rx,
            streaming: false,
            current_tool: None,
            login_flow: None,
        }
    }

    /// Run the TUI event loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        terminal::setup()?;
        let mut term = terminal::create_terminal()?;

        let result = self.run_loop(&mut term).await;

        terminal::restore()?;
        result
    }

    async fn run_loop(
        &mut self,
        term: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> anyhow::Result<()> {
        let mut reader = EventStream::new();

        while self.running {
            term.draw(|frame| self.draw(frame))?;

            tokio::select! {
                crossterm_event = reader.next() => {
                    if let Some(Ok(event)) = crossterm_event {
                        self.handle_input_event(&event);
                    }
                }
                Some(event) = self.event_rx.recv() => {
                    self.handle_agent_event(event);
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if let Some(ref mut login) = self.login_flow {
            let main = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(8)])
                .split(area);
            self.status.render(main[0], frame);
            login.render(main[1], frame);
            return;
        }

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .split(area);

        self.status.render(main[0], frame);
        self.chat.render(main[1], frame);
        self.editor.render(main[2], frame);
    }

    fn handle_input_event(&mut self, event: &crossterm::event::Event) {
        // If login flow is active, handle its events exclusively.
        if let Some(ref mut login) = self.login_flow {
            let _ = login.handle_event(event);
            if login.is_done() {
                if !login.is_cancelled()
                    && let Some((provider, token)) = login.take_result()
                {
                    let _ = self
                        .action_tx
                        .send(TuiAction::LoginResult { provider, token });
                    self.chat.add_message(ChatMessage {
                        role: ChatRole::System,
                        text: "Token saved successfully.".into(),
                        tool_name: None,
                        is_streaming: false,
                    });
                }
                self.login_flow = None;
            }
            return;
        }

        if let Some(action) = resolve_event(event, &self.keybindings) {
            self.handle_action(action);
            return;
        }

        if let crossterm::event::Event::Key(key) = event
            && key.code == crossterm::event::KeyCode::Tab
        {
            self.focus_idx = (self.focus_idx + 1) % 2;
            self.editor.focus(self.focus_idx == 0);
            self.chat.focus(self.focus_idx == 1);
            return;
        }

        let action = if self.focus_idx == 0 {
            self.editor.handle_event(event)
        } else {
            self.chat.handle_event(event)
        };

        if let Some(action) = action {
            self.handle_action(action);
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::SendMessage(text) => {
                // Intercept slash commands before sending to agent.
                if let Some(slash) = text.strip_prefix('/') {
                    self.handle_slash_command(slash);
                    return;
                }
                self.chat.add_message(ChatMessage {
                    role: ChatRole::User,
                    text: text.clone(),
                    tool_name: None,
                    is_streaming: false,
                });
                self.status.set_agent_state("streaming");
                self.streaming = true;
                let _ = self.message_tx.send(text);
            }
            Action::Quit => {
                self.running = false;
            }
            _ => {}
        }
    }

    /// Handle a slash command (text after the initial `/`).
    fn handle_slash_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match command {
            "help" | "h" => {
                let help_text = [
                    "Slash commands:",
                    "  /model <id>    Switch to a different model",
                    "  /thinking <lvl> Set thinking level (off, low, medium, high)",
                    "  /clear         Clear the chat display",
                    "  /session       Show current session info",
                    "  /fork          Fork the current session",
                    "  /login         Configure provider authentication",
                    "  /help          Show this help",
                ]
                .join("\n");
                self.chat.add_message(ChatMessage {
                    role: ChatRole::System,
                    text: help_text,
                    tool_name: None,
                    is_streaming: false,
                });
            }
            "model" => {
                if arg.is_empty() {
                    self.chat.add_message(ChatMessage {
                        role: ChatRole::System,
                        text: "Usage: /model <model-id>".into(),
                        tool_name: None,
                        is_streaming: false,
                    });
                } else {
                    let _ = self.action_tx.send(TuiAction::SwitchModel(arg.to_string()));
                    self.chat.add_message(ChatMessage {
                        role: ChatRole::System,
                        text: format!("Switching model to {arg}..."),
                        tool_name: None,
                        is_streaming: false,
                    });
                }
            }
            "thinking" => {
                if arg.is_empty() {
                    self.chat.add_message(ChatMessage {
                        role: ChatRole::System,
                        text: "Usage: /thinking <off|low|medium|high>".into(),
                        tool_name: None,
                        is_streaming: false,
                    });
                } else {
                    let _ = self.action_tx.send(TuiAction::SetThinking(arg.to_string()));
                    self.status.thinking = arg.to_string();
                    self.chat.add_message(ChatMessage {
                        role: ChatRole::System,
                        text: format!("Thinking level set to {arg}"),
                        tool_name: None,
                        is_streaming: false,
                    });
                }
            }
            "clear" => {
                self.chat.messages.clear();
            }
            "session" | "s" => {
                let info = format!(
                    "Session: {}\nModel: {}\nThinking: {}",
                    self.status.session_id, self.status.model, self.status.thinking
                );
                self.chat.add_message(ChatMessage {
                    role: ChatRole::System,
                    text: info,
                    tool_name: None,
                    is_streaming: false,
                });
            }
            "login" => {
                // Start the login flow.
                let providers = known_providers(false, false, false, false);
                self.login_flow = Some(LoginFlow::new(self.theme.clone(), providers));
            }
            "fork" => {
                let _ = self.action_tx.send(TuiAction::ForkSession);
                self.chat.add_message(ChatMessage {
                    role: ChatRole::System,
                    text: "Forking session...".into(),
                    tool_name: None,
                    is_streaming: false,
                });
            }
            _ => {
                self.chat.add_message(ChatMessage {
                    role: ChatRole::System,
                    text: format!(
                        "Unknown command: /{command}. Type /help for available commands."
                    ),
                    tool_name: None,
                    is_streaming: false,
                });
            }
        }
    }

    fn handle_agent_event(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::TextDelta(text) => {
                if self.streaming {
                    self.chat.update_last(&text, ChatRole::Assistant, true);
                }
            }
            TuiEvent::ThinkingDelta(text) => {
                self.chat
                    .update_last(&format!("[thinking] {text}"), ChatRole::System, true);
            }
            TuiEvent::ToolStart { name, .. } => {
                self.current_tool = Some(name.clone());
                self.status.set_agent_state("tool executing");
                self.status.set_tool_progress(&format!("running {name}..."));
                self.chat.add_message(ChatMessage {
                    role: ChatRole::Tool,
                    text: format!("{name} executing..."),
                    tool_name: Some(name),
                    is_streaming: true,
                });
            }
            TuiEvent::ToolProgress { message, .. } => {
                self.status.set_tool_progress(&message);
            }
            TuiEvent::ToolEnd {
                name: _name,
                output,
            } => {
                self.current_tool = None;
                self.status.set_tool_progress("");
                self.status.set_agent_state("streaming");
                if let Some(last) = self.chat.messages.last_mut()
                    && last.role == ChatRole::Tool
                {
                    last.text = output;
                    last.is_streaming = false;
                }
            }
            TuiEvent::TurnStart => {
                self.streaming = true;
                self.status.set_agent_state("streaming");
            }
            TuiEvent::TurnEnd { stop_reason } => {
                self.chat.finish_last(ChatRole::Assistant);
                self.streaming = false;
                self.status
                    .set_agent_state(&format!("idle (stopped: {stop_reason})"));
            }
            TuiEvent::AgentEnd => {
                self.chat.finish_last(ChatRole::Assistant);
                self.streaming = false;
                self.status.set_agent_state("idle");
            }
            TuiEvent::Error(msg) => {
                self.chat.add_message(ChatMessage {
                    role: ChatRole::System,
                    text: format!("Error: {msg}"),
                    tool_name: None,
                    is_streaming: false,
                });
                self.status.set_agent_state("error");
            }
        }
    }
}
