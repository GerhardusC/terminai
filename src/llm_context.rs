use std::{
    fmt::Display,
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::Duration,
};

use cursive::{
    CbSink,
    views::{DummyView, LinearLayout, NamedView, ScrollView, TextContent, TextView},
};

use crate::{custom_views::spinner_view::SpinnerView, models::ollama, utils::show_message};

pub struct LlmContext {
    pub conversation: Vec<Message>,
    update_rx: Receiver<LlmContextUpdateMessage>,
    pub update_tx: Sender<LlmContextUpdateMessage>,
    pub sink: CbSink,
    pub text_content: TextContent,
    current_message: String,
}

pub enum LoadingState {
    Ready,
    Fetching,
    Streaming,
    Thinking,
}

pub enum LlmContextUpdateMessage {
    // DO API INTERACTIONS TODO: Add associated model
    CallApi,
    UpdateLoadingState(LoadingState),
    CurrentMessageEnd,

    // CURRENT MESSAGE
    AppendToCurrentMessage(String),

    // FOR DEBUGGING
    ViewCurrentContext,

    // FULL CONTEXT
    AddMessage(Message),
    Clear,

    // SYSTEM
    Stop,
}

#[derive(Debug)]
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
            current_message: String::from(""),
        }
    }

    pub fn start(mut self) {
        // This is the only thread abble to write to the messages rwlock.
        thread::spawn(move || {
            let messages_ref = Arc::new(RwLock::new(self.conversation));
            loop {
                let sink = self.sink.clone();
                let messages_ref2 = messages_ref.clone();
                if let Ok(msg) = self.update_rx.recv() {
                    match msg {
                        LlmContextUpdateMessage::CallApi => {
                            let messages_ref = messages_ref.clone();
                            let sink = self.sink.clone();
                            let update_tx = self.update_tx.clone();

                            thread::spawn(move || {
                                ollama::stream_res_to_llm_context(update_tx, sink, messages_ref);
                            });
                        }
                        LlmContextUpdateMessage::UpdateLoadingState(loading_state) => {
                            match loading_state {
                                LoadingState::Ready => {
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            "loading-view",
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(DummyView);
                                            },
                                        );
                                    }));
                                }
                                LoadingState::Fetching => {
                                    let sink_cp = sink.clone();
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            "loading-view",
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(TextView::new("Fetching"));
                                            },
                                        );
                                    }));
                                }
                                LoadingState::Streaming => {
                                    let sink_cp = sink.clone();
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            "loading-view",
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(TextView::new("Cooking"));
                                            },
                                        );
                                    }));
                                }
                                LoadingState::Thinking => {
                                    let sink_cp = sink.clone();
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            "loading-view",
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(TextView::new("Thinking"));
                                            },
                                        );
                                    }));
                                }
                            };
                        }
                        LlmContextUpdateMessage::CurrentMessageEnd => {
                            if let Ok(mut messages) = messages_ref2.write() {
                                messages
                                    .push(Message::new(Role::Model, self.current_message.clone()));
                            }

                            self.current_message.clear();
                            let content = self.text_content.clone();
                            let _ = sink.send(Box::new(move |_| {
                                content.append("\n");
                            }));
                            let _ =
                                self.update_tx
                                    .send(LlmContextUpdateMessage::UpdateLoadingState(
                                        LoadingState::Ready,
                                    ));
                        }
                        LlmContextUpdateMessage::AppendToCurrentMessage(msg) => {
                            self.current_message.push_str(&msg);

                            // All content gets sent to update screen by default
                            let content = self.text_content.clone();
                            let _ = sink.send(Box::new(move |s| {
                                content.append(msg);
                                s.call_on_name(
                                    "response-container",
                                    |v: &mut NamedView<ScrollView<TextView>>| {
                                        v.get_mut().scroll_to_bottom();
                                    },
                                );
                            }));
                        }
                        LlmContextUpdateMessage::AddMessage(message) => {
                            if let Ok(mut messages) = messages_ref2.write() {
                                messages.push(message);
                            }
                            self.text_content.append("\n\n");
                        }
                        LlmContextUpdateMessage::Clear => {
                            if let Ok(mut messages) = messages_ref2.write() {
                                messages.clear();
                            }
                            self.current_message.clear();
                            self.text_content.set_content("");
                        }
                        LlmContextUpdateMessage::ViewCurrentContext => {
                            let messages = messages_ref2.clone();
                            let _ = sink.send(Box::new(move |s| {
                                let mut msg_str = None;
                                if let Ok(messages) = messages.read() {
                                    msg_str = Some(format!("{:#?}", &messages));
                                };
                                show_message(s, format!("{:#?}", msg_str));
                            }));
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

#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}
