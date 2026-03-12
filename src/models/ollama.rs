use anyhow::Result;
use std::{
    collections::HashMap,
    io::Read,
    sync::{Arc, Mutex},
    thread,
};

use cursive::{
    Cursive,
    views::{NamedView, ScrollView, TextView},
};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::{
    app_state::{set_is_loading, set_is_streaming, set_ready},
    models::{LlmContext, LlmContextManager, Message},
    utils::show_message,
};

pub fn stream_res_to_gui(s: &mut Cursive, context: Arc<Mutex<LlmContext>>) {
    let sink = s.cb_sink().clone();

    thread::spawn(move || {
        let res = call_api(context.clone());

        let _ = sink.send(Box::new(|s| {
            set_is_loading(s);
        }));

        let Ok(mut res) = res else {
            let _ = sink.send(Box::new(move |s| {
                set_ready(s);
                show_message(s, "Unable to send api request");
            }));
            return;
        };

        let _ = sink.send(Box::new(|s| {
            set_is_streaming(s);
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
            context.add_message(Message::new("system".to_owned(), msg.to_string()));
        }

        // Add new line after response
        let _ = sink.send(Box::new(|s| {
            s.call_on_name("output-area", |v: &mut NamedView<ScrollView<TextView>>| {
                v.get_mut().get_inner_mut().append("\n\n");
                v.get_mut().scroll_to_bottom();
            });
            // Unset loading state
            set_ready(s);
        }));
    });
}

fn call_api(context: Arc<Mutex<LlmContext>>) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

    let messages = match context.lock() {
        Ok(context) => {
            context
                .conversation
                .iter()
                .map(|x| OutgoingMessage {
                    // TODO: Rmove these clones, this is bad.
                    role: x.role.clone(),
                    content: x.content.clone(),
                })
                .collect::<Vec<_>>()
        }
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
