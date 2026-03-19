use std::{
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use cursive::{
    CbSink,
    theme::{Color, Style},
    utils::markup::StyledString,
    view::Nameable,
    views::{DummyView, LinearLayout, NamedView, TextArea, TextContent, TextView},
};

use crate::{
    custom_views::spinner_view::SpinnerView,
    models::ollama::{self, OutgoingMessage},
    utils::show_message,
};

static LOADING_VIEW_NAME: &str = "loading-view";
static PROMPT_AREA_NAME: &str = "prompt-area";

type LoadingArea = NamedView<LinearLayout>;
type PromptArea = NamedView<TextArea>;

pub struct LlmContext {
    pub conversation: Vec<Message>,
    update_rx: Receiver<LlmContextUpdateMessage>,
    pub update_tx: Sender<LlmContextUpdateMessage>,
    pub sink: CbSink,
    pub text_content: TextContent,
    current_message: String,
    current_thought: String,
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
    CurrentMessageEnd(Role),
    AddToCurrentThought(String),

    // VISUAL ONLY
    ClearOutput,

    // CURRENT MESSAGE
    AppendToCurrentMessage(String),
    ClearPrompt,

    // FOR DEBUGGING
    ViewCurrentContext,

    // FULL CONTEXT
    AddMessage(Message),
    AddUserPrompt,
    ClearContext,

    // SYSTEM
    Error(String),
    Stop,
}

#[derive(Debug)]
pub enum Role {
    User,
    Model,
    System,
    Assistant,
    Tool,
    Other(String),
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Role::User => "user",
            Role::Model => "model",
            Role::System => "system",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
            Role::Other(s) => s,
        })
    }
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        match value {
            "user" => Role::User,
            "model" => Role::Model,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            "system" => Role::System,
            _ => Role::Other(value.to_owned()),
        }
    }
}

impl LlmContext {
    pub fn new(sink: CbSink) -> Self {
        let (update_tx, update_rx) = mpsc::channel::<LlmContextUpdateMessage>();
        let text_content = TextContent::new("");

        Self {
            conversation: vec![],
            update_tx,
            update_rx,
            sink,
            text_content,
            current_message: String::from(""),
            current_thought: String::from(""),
        }
    }

    fn create_loading_view() -> LoadingArea {
        LinearLayout::horizontal().with_name(LOADING_VIEW_NAME)
    }

    fn create_prompt_area() -> PromptArea {
        TextArea::new().with_name(PROMPT_AREA_NAME)
    }

    pub fn start(mut self) -> (PromptArea, LoadingArea) {
        // This thread is the only one that can write to views.
        thread::spawn(move || {
            loop {
                let sink = self.sink.clone();
                if let Ok(msg) = self.update_rx.recv() {
                    match msg {
                        LlmContextUpdateMessage::CallApi => {
                            let update_tx = self.update_tx.clone();

                            let messages = self
                                .conversation
                                .iter()
                                .map(|item| item.into())
                                .collect::<Vec<OutgoingMessage>>();

                            thread::spawn(move || {
                                ollama::stream_res_to_llm_context(update_tx, messages);
                            });
                        }
                        LlmContextUpdateMessage::UpdateLoadingState(loading_state) => {
                            match loading_state {
                                LoadingState::Ready => {
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            LOADING_VIEW_NAME,
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
                                            LOADING_VIEW_NAME,
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(DummyView);
                                                v.get_mut().add_child(TextView::new("Fetching"));
                                            },
                                        );
                                    }));
                                }
                                LoadingState::Streaming => {
                                    let sink_cp = sink.clone();
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            LOADING_VIEW_NAME,
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(DummyView);
                                                v.get_mut().add_child(TextView::new("Cooking"));
                                            },
                                        );
                                    }));
                                }
                                LoadingState::Thinking => {
                                    let sink_cp = sink.clone();
                                    let _ = sink.send(Box::new(|s| {
                                        s.call_on_name(
                                            LOADING_VIEW_NAME,
                                            |v: &mut NamedView<LinearLayout>| {
                                                v.get_mut().clear();
                                                v.get_mut().add_child(SpinnerView::new(
                                                    sink_cp,
                                                    Duration::from_millis(50),
                                                ));
                                                v.get_mut().add_child(DummyView);
                                                v.get_mut().add_child(TextView::new("Thinking"));
                                            },
                                        );
                                    }));
                                }
                            };
                        }
                        LlmContextUpdateMessage::CurrentMessageEnd(role) => {
                            self.conversation.push(Message::new(
                                role,
                                self.current_message.clone(),
                                Some(self.current_thought.clone()),
                            ));
                            self.current_message.clear();
                            self.current_thought.clear();

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
                            let _ = sink.send(Box::new(move |_| {
                                content.append(msg);
                            }));
                        }
                        LlmContextUpdateMessage::AddToCurrentThought(thought) => {
                            self.current_thought.push_str(&thought);
                            let mut styled_thought = StyledString::new();
                            styled_thought.append_styled(
                                &thought,
                                Style::from(Color::Dark(cursive::theme::BaseColor::Green)),
                            );
                            self.text_content.append(styled_thought);
                        }
                        LlmContextUpdateMessage::AddMessage(message) => {
                            self.conversation.push(message);
                        }
                        LlmContextUpdateMessage::ClearPrompt => {
                            self.current_message.clear();
                            let _ = self.sink.send(Box::new(|s| {
                                s.call_on_name(PROMPT_AREA_NAME, |v: &mut NamedView<TextArea>| {
                                    v.get_mut().set_content("");
                                });
                            }));
                        }
                        LlmContextUpdateMessage::ClearOutput => {
                            self.text_content.set_content("");
                        }
                        LlmContextUpdateMessage::ClearContext => {
                            self.conversation.clear();
                            self.current_message.clear();
                            self.text_content.set_content("");
                        }
                        LlmContextUpdateMessage::ViewCurrentContext => {
                            let messages = format!("{:?}", &self.conversation);
                            let _ = sink.send(Box::new(move |s| {
                                show_message(s, messages);
                            }));
                        }
                        LlmContextUpdateMessage::AddUserPrompt => {
                            let update_tx = self.update_tx.clone();
                            let _ = sink.send(Box::new(move |s| {
                                if let Some(prompt) = s.call_on_name(
                                    PROMPT_AREA_NAME,
                                    move |v: &mut NamedView<TextArea>| {
                                        v.get_mut().get_content().to_owned()
                                    },
                                ) {
                                    let _ = update_tx.send(LlmContextUpdateMessage::AddMessage(
                                        Message::new(Role::User, prompt, None),
                                    ));

                                    let _ = update_tx.send(LlmContextUpdateMessage::CallApi);
                                    let _ = update_tx.send(LlmContextUpdateMessage::ClearPrompt);
                                };
                            }));
                        }
                        LlmContextUpdateMessage::Error(e) => {
                            let _ = sink.send(Box::new(move |s| {
                                show_message(s, e);
                            }));
                        }
                        LlmContextUpdateMessage::Stop => break,
                    }
                }
            }
        });

        let (prompt_area, loading_view) = (Self::create_prompt_area(), Self::create_loading_view());
        (prompt_area, loading_view)
    }
}

impl Message {
    pub fn new(role: Role, content: String, thought: Option<String>) -> Self {
        Self {
            role,
            content,
            thought,
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub thought: Option<String>,
}
