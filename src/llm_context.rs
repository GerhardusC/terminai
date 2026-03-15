use std::{
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use cursive::{CbSink, views::TextContent};

use crate::models::ollama;

pub struct LlmContext {
    pub conversation: Vec<Message>,
    update_rx: Receiver<LlmContextUpdateMessage>,
    pub update_tx: Sender<LlmContextUpdateMessage>,
    pub sink: CbSink,
    pub text_content: TextContent,
    status: LoadingState,
    current_message: String,
}

pub enum LoadingState {
    Ready,
    Fetching,
    Streaming,
}

pub enum LlmContextUpdateMessage {
    // DO API INTERACTIONS TODO: Add associated model
    CallApi,
    UpdateLoadingState(LoadingState),
    CurrentMessageEnd,

    // CURRENT MESSAGE
    AppendToCurrentMessage(String),
    ClearCurrentMessage,

    // FULL CONTEXT
    AddMessage(Message),
    Clear,

    // SYSTEM
    Stop,
}

pub enum Role {
    User,
    Model,
    System,
    Assistant,
    Tool,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::User => "user",
            Role::Model => "model",
            Role::System => "system",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        })
    }
}

impl LlmContext {
    pub fn new(sink: CbSink, text_content: TextContent) -> Self {
        let (update_tx, update_rx) = mpsc::channel::<LlmContextUpdateMessage>();

        Self {
            conversation: vec![],
            update_tx,
            update_rx,
            sink,
            text_content,
            status: LoadingState::Ready,
            current_message: String::from(""),
        }
    }

    pub fn start(mut self) {
        thread::spawn(move || {
            loop {
                if let Ok(msg) = self.update_rx.recv() {
                    match msg {
                        LlmContextUpdateMessage::CallApi => {
                            ollama::stream_res_to_llm_context(
                                self.update_tx.clone(),
                                self.sink.clone(),
                                &self,
                            );
                        }
                        LlmContextUpdateMessage::UpdateLoadingState(loading_state) => {
                            self.status = loading_state
                        }
                        LlmContextUpdateMessage::CurrentMessageEnd => {
                            self.conversation
                                .push(Message::new(Role::System, self.current_message.clone()));
                            self.text_content.append("\n\n");
                        }
                        LlmContextUpdateMessage::AppendToCurrentMessage(msg) => {
                            self.current_message.push_str(&msg);
                            self.text_content.append(msg);
                        }
                        LlmContextUpdateMessage::ClearCurrentMessage => {
                            self.current_message.clear()
                        }
                        LlmContextUpdateMessage::AddMessage(message) => {
                            self.conversation.push(message)
                        }
                        LlmContextUpdateMessage::Clear => {
                            self.conversation.clear();
                            self.current_message.clear();
                            self.text_content.set_content("");
                        }
                        LlmContextUpdateMessage::Stop => break,
                    }
                }
            }
        });
    }
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }
}

pub struct Message {
    pub role: Role,
    pub content: String,
}
