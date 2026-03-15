use anyhow::Result;
use std::{collections::HashMap, io::Read, sync::mpsc::Sender};

use cursive::CbSink;
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::{
    llm_context::{LlmContextUpdateMessage, LoadingState, Message, Role},
    utils::show_message,
};

pub fn stream_res_to_llm_context(
    sender: Sender<LlmContextUpdateMessage>,
    sink: CbSink,
    context: Vec<OutgoingMessage>,
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
    let mut role: Role = Role::Assistant;
    let mut thinking = false;
    while let Ok(x) = res.read(&mut buf)
        && x > 0
    {
        let res = String::from_utf8(buf[..x].to_vec()).expect("all valid utf8");

        let parsed: Result<OllamaStreamingResponse, serde_json::Error> = serde_json::from_str(&res);

        match parsed {
            Ok(parsed) => {
                if parsed.message.content == "<think>" {
                    thinking = true;
                    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
                        LoadingState::Thinking,
                    ));
                } else if parsed.message.content == "</think>" {
                    thinking = false;
                    let _ = sender.send(LlmContextUpdateMessage::UpdateLoadingState(
                        LoadingState::Streaming,
                    ));
                };

                if thinking {
                    let _ = sender.send(LlmContextUpdateMessage::AddToCurrentThought(
                        parsed.message.content,
                    ));
                } else {
                    let _ = sender.send(LlmContextUpdateMessage::AppendToCurrentMessage(
                        parsed.message.content,
                    ));
                }

                role = parsed.message.role.as_str().into();
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
    let _ = sender.send(LlmContextUpdateMessage::CurrentMessageEnd(role));
}

pub fn call_api(messages: Vec<OutgoingMessage>) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

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

impl From<&Message> for OutgoingMessage {
    fn from(value: &Message) -> Self {
        Self {
            role: value.role.to_string(),
            content: value.content.to_string(),
        }
    }
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
