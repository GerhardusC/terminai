use std::fmt::Display;

use cursive::Cursive;

use crate::models::{LlmContext, Message};

pub struct AppState {
    pub loading: bool,
    pub streaming: bool,
    pub llm_context: LlmContext,
}

impl AppState {
    fn new() -> Self {
        Self {
            loading: false,
            streaming: false,
            llm_context: LlmContext::default(),
        }
    }
    pub fn init(s: &mut Cursive) {
        let app_state = Self::new();
        s.set_user_data(app_state);
    }
}

pub enum Role {
    User,
    System,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::User => "user",
            Role::System => "system",
        })
    }
}

pub fn push_message_to_context(s: &mut Cursive, message: &str, role: Role) {
    s.user_data::<AppState>().map(|x| {
        x.llm_context
            .conversation
            .push(Message::new(role.to_string(), message.to_string()))
    });
}

pub fn clear_context(s: &mut Cursive) {
    s.user_data::<AppState>()
        .map(|x| x.llm_context.conversation.clear());
}

pub fn set_is_loading(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = true;
        user_data.streaming = false;
    }
}

pub fn set_is_streaming(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = false;
        user_data.streaming = true;
    }
}

pub fn set_ready(s: &mut Cursive) {
    let user_data = s.user_data::<AppState>();
    if let Some(user_data) = user_data {
        user_data.loading = false;
        user_data.streaming = false;
    }
}

pub fn get_is_loading(s: &mut Cursive) -> bool {
    s.user_data::<AppState>()
        .map(|x| x.loading)
        .unwrap_or(false)
}

pub fn get_is_streaming(s: &mut Cursive) -> bool {
    s.user_data::<AppState>()
        .map(|x| x.loading)
        .unwrap_or(false)
}
