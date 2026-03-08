use anyhow::Result;
use std::{io::Read, sync::OnceLock, thread};

use cursive::{
    Cursive,
    views::{NamedView, ScrollView, TextView},
};
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::utils::show_message;

static API_KEY: OnceLock<String> = OnceLock::new();

fn get_api_key() -> String {
    API_KEY
        .get_or_init(|| {
            std::fs::read_to_string("secrets.txt")
                .expect("Secrets file should exist")
                .trim()
                .to_string()
        })
        .to_string()
}

// E.G. data: {
//   "candidates": [
//     {
//       "content": {
//         "parts": [
//           {
//             "text": "Using the Gemini API is relatively straightforward. Google provides a platform called **Google AI Studio** that allows you to get an"
//           }
//         ],
//         "role": "model"
//       },
//       "index": 0
//     }
//   ],
//   "usageMetadata": {
//     "promptTokenCount": 16,
//     "candidatesTokenCount": 24,
//     "totalTokenCount": 416,
//     "promptTokensDetails": [
//       {
//         "modality": "TEXT",
//         "tokenCount": 16
//       }
//     ],
//     "thoughtsTokenCount": 376
//   },
//   "modelVersion": "gemini-3-flash-preview",
//   "responseId": "jJusaaG3HvTzxs0P7_3ywQY"
// }

// RESPONSES TODO:

#[allow(unused)]
#[derive(Deserialize)]
struct PartialGapiResponse {
    data: PartialGapiResponseData,
}

#[allow(unused)]
#[derive(Deserialize)]
struct PartialGapiResponseData {
    candidates: Vec<Candidate>,
    // TODO: Usage metadata.
}

#[allow(unused)]
#[derive(Deserialize)]
struct GapiContent {
    parts: Vec<GapiPart>,
    role: String,
}

#[allow(unused)]
#[derive(Deserialize)]
struct Candidate {
    content: GapiContent,
    index: u32,
}

#[derive(Serialize)]
struct GapiReqBody {
    contents: Vec<GapiReqParts>,
}

#[derive(Serialize)]
struct GapiReqParts {
    parts: Vec<GapiPart>,
}

#[derive(Serialize, Deserialize)]
struct GapiPart {
    text: String,
}

pub fn stream_res_to_gui(s: &mut Cursive, prompt: String) {
    let sink = s.cb_sink().clone();

    thread::spawn(move || {
        let res = call_api(prompt);

        let Ok(mut res) = res else {
            let _ = sink.send(Box::new(move |s| {
                show_message(s, "Unable to send api request");
            }));
            return;
        };
        let mut buf = [0; 0xFF];
        while let Ok(x) = res.read(&mut buf)
            && x > 0
        {
            let _ = sink.send(Box::new(move |s| {
                s.call_on_name("output-area", |v: &mut NamedView<ScrollView<TextView>>| {
                    v.get_mut()
                        .get_inner_mut()
                        .append(String::from_utf8(buf[..x].to_vec()).expect("all valid utf8"));

                    v.get_mut().scroll_to_bottom();
                });
            }));
            buf.fill(0);
        }
    });
}

fn call_api(msg: String) -> Result<Response> {
    let client = reqwest::blocking::Client::new();

    let req_body = GapiReqBody {
        contents: vec![GapiReqParts {
            parts: vec![GapiPart { text: msg }],
        }],
    };

    let res = client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:streamGenerateContent?alt=sse")
        .header("x-goog-api-key", get_api_key())
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()?;

    Ok(res)
}
