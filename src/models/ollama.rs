use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    io::Read,
    sync::{Arc, Mutex, RwLock, mpsc::Sender},
    thread,
    time::Duration,
};

use cursive::{
    CbSink, Cursive,
    views::{DummyView, LinearLayout, NamedView, ScrollView, TextArea, TextView},
};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::{
    custom_views::spinner_view::SpinnerView,
    llm_context::{LlmContext, LlmContextUpdateMessage, LoadingState, Message, Role},
    utils::show_message,
};

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

pub fn stream_res_to_gui(s: &mut Cursive, context: Arc<Mutex<LlmContext>>) {
    let sink = s.cb_sink().clone();

    thread::spawn(move || {
        let sink2 = sink.clone();
        // NOTE: ASYNC REQ STARTS
        let _ = sink.send(Box::new(move |s| {
            s.call_on_name("prompt-container", |v: &mut NamedView<LinearLayout>| {
                v.get_mut().insert_child(
                    1,
                    LinearLayout::horizontal()
                        .child(DummyView)
                        .child(SpinnerView::new(sink2, Duration::from_millis(50)))
                        .child(DummyView)
                        .child(TextView::new("Thinking...")),
                );
            });
            s.call_on_name("prompt-area", |v: &mut NamedView<ScrollView<TextArea>>| {
                v.get_mut().get_inner_mut().set_content("");
            });
        }));

        let res = call_api_with_mux(context.clone());

        let mut res = match res {
            Ok(res) => res,
            Err(e) => {
                let e = format!("{e}");
                let _ = sink.send(Box::new(move |s| {
                    // REMOVE LOADING SPINNER IF RES IS NOT OK.
                    s.call_on_name("prompt-container", |v: &mut NamedView<LinearLayout>| {
                        v.get_mut().remove_child(1);
                    });
                    show_message(s, e);
                }));
                return;
            }
        };

        // NOTE: STREAMING STARTS
        let _ = sink.send(Box::new(move |s| {
            s.call_on_name("prompt-container", |v: &mut NamedView<LinearLayout>| {
                v.get_mut().remove_child(1);
            });
        }));

        let full_message = Arc::new(Mutex::new(String::new()));
        let mut buf = [0; 0x1FF];
        while let Ok(x) = res.read(&mut buf)
            && x > 0
        {
            // Each iteration of the loop needs a reference.
            let full_message = full_message.clone();
            let _ = sink.send(Box::new(move |s| {
                let sink = s.cb_sink().clone();
                s.call_on_name("output-area", |v: &mut NamedView<ScrollView<TextView>>| {
                    let res = String::from_utf8(buf[..x].to_vec()).expect("all valid utf8");

                    let parsed: Result<OllamaStreamingResponse, serde_json::Error> =
                        serde_json::from_str(&res);

                    match parsed {
                        Ok(parsed) => {
                            if let Ok(mut part) = full_message.clone().lock() {
                                part.extend(parsed.message.content.chars());
                            }
                            v.get_mut().get_inner_mut().append(parsed.message.content);
                            v.get_mut().scroll_to_bottom();
                        }
                        Err(e) => {
                            let _ = sink.send(Box::new(move |s| {
                                if e.is_eof() {
                                    return;
                                }
                                show_message(s, format!("Err: {}, Res: {:?}", e, res.clone()));
                            }));
                        }
                    }
                });
            }));
            buf.fill(0);
        }

        if let Ok(msg) = full_message.lock() {
            context.add_message(Message::new(Role::System, msg.to_string()));
        }

        // Add new line after response
        let _ = sink.send(Box::new(|s| {
            s.call_on_name("output-area", |v: &mut NamedView<ScrollView<TextView>>| {
                v.get_mut().get_inner_mut().append("\n\n");
                v.get_mut().scroll_to_bottom();
            });
        }));
    });
}

pub fn stream_res_to_llm_context(
    sender: Sender<LlmContextUpdateMessage>,
    sink: CbSink,
    context: Arc<RwLock<Vec<Message>>>,
) {
    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
        LoadingState::Fetching,
    ));
    let res = call_api(context);
    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
        LoadingState::Streaming,
    ));

    let mut res = match res {
        Ok(res) => res,
        Err(e) => {
            let e = e.to_string();
            let _ = sink.send(Box::new(move |s| {
                show_message(s, e);
            }));
            let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
                LoadingState::Ready,
            ));
            return;
        }
    };

    let mut buf = [0; 0xFFF];
    while let Ok(x) = res.read(&mut buf)
        && x > 0
    {
        let res = String::from_utf8(buf[..x].to_vec()).expect("all valid utf8");

        let parsed: Result<OllamaStreamingResponse, serde_json::Error> = serde_json::from_str(&res);

        match parsed {
            Ok(parsed) => {
                if parsed.message.content == "<think>" {
                    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
                        LoadingState::Thinking,
                    ));
                } else if parsed.message.content == "</think>" {
                    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
                        LoadingState::Streaming,
                    ));
                };

                let _ = sender.send(LlmContextUpdateMessage::AppendToCurrentMessage(
                    parsed.message.content,
                ));
            }
            Err(e) => {
                let _ = sink.send(Box::new(move |s| {
                    if e.is_eof() {
                        return;
                    }
                    show_message(s, format!("Err: {}, Res: {:?}", e, res.clone()));
                }));
            }
        }
        buf.fill(0);
    }
    let _ = sender.send(LlmContextUpdateMessage::CurrentMessageEnd);
}

fn call_api_with_mux(context: Arc<Mutex<LlmContext>>) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

    let messages = match context.lock() {
        Ok(context) => context
            .conversation
            .iter()
            .map(|x| OutgoingMessage {
                role: x.role.to_string(),
                content: x.content.clone(),
            })
            .collect::<Vec<_>>(),
        Err(_) => vec![],
    };

    let req_body = OllamaStreamingRequest {
        model: "gemma3".to_owned(),
        messages,
    };

    let res = client
        .post("http://localhost:11434/api/chat")
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()?;

    Ok(res)
}

pub fn call_api(conversation: Arc<RwLock<Vec<Message>>>) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

    let messages = match conversation.read() {
        Ok(conversation) => conversation
            .iter()
            .map(|x| OutgoingMessage {
                role: x.role.to_string(),
                content: x.content.clone(),
            })
            .collect::<Vec<_>>(),
        Err(e) => return Err(anyhow!("Mutext poisoned: {}", e)),
    };

    let req_body = OllamaStreamingRequest {
        model: "deepseek-r1".to_owned(),
        messages,
    };

    let res = client
        .post("http://gerhardus-desktop.local:11434/api/chat")
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()?;

    Ok(res)
}

#[derive(Serialize, Deserialize)]
pub struct OllamaStreamingRequest {
    model: String,
    messages: Vec<OutgoingMessage>,
}

#[derive(Serialize, Deserialize)]
pub struct OutgoingMessage {
    /// system, user, assistant, tool
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaStreamingResponse {
    model: String,
    created_at: String,
    message: OllamaMessage,
    done: bool,
    done_reason: Option<String>,
    total_duration: Option<i64>,
    load_duration: Option<i64>,
    prompt_eval_count: Option<i64>,
    prompt_eval_duration: Option<i64>,
    eval_count: Option<i64>,
    eval_duration: Option<i64>,
    logprobs: Option<Vec<Logprob>>,
}

#[derive(Serialize, Deserialize)]
pub struct Logprob {
    token: Option<String>,
    logprob: Option<i64>,
    bytes: Option<Vec<i64>>,
    top_logprobs: Option<Vec<Logprob>>,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaMessage {
    role: String,
    content: String,
    thinking: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    images: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ToolCall {
    function: ModelFunction,
}

#[derive(Serialize, Deserialize)]
pub struct ModelFunction {
    name: String,
    description: String,
    arguments: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Arguments {}
