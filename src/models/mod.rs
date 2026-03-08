use std::sync::{Arc, Mutex};

pub mod gemini;
pub mod ollama;

pub struct LlmContext {
    conversation: Vec<Message>,
}

impl LlmContext {
    pub fn new() -> Self {
        Self {
            conversation: vec![],
        }
    }
}

impl Default for LlmContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Message {
    /// Role: system, user, assistant, tool
    pub fn new(role: String, content: String) -> Self {
        Self { role, content }
    }
}

pub struct Message {
    /// system, user, assistant, tool
    role: String,
    content: String,
}

pub trait LlmContextManager {
    fn add_message(self, msg: Message) -> Self;
    fn clear(&self);
}

impl LlmContextManager for Arc<Mutex<LlmContext>> {
    fn add_message(self, msg: Message) -> Self {
        if let Ok(mut context) = self.lock() {
            context.conversation.push(msg);
        }
        self
    }

    fn clear(&self) {
        if let Ok(mut context) = self.lock() {
            context.conversation.clear();
        }
    }
}
