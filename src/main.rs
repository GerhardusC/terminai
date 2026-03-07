use anyhow::Result;
use reqwest::blocking::Response;
use std::{io::Read, sync::OnceLock, thread};

use cursive::{
    Cursive, CursiveExt,
    view::{Nameable, Resizable, ScrollStrategy, Scrollable, SizeConstraint},
    views::{
        Button, Dialog, DummyView, LinearLayout, NamedView, ResizedView, ScrollView, TextArea,
        TextView,
    },
};
use cursive_aligned_view::Alignable;
use serde::{Deserialize, Serialize};

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

fn main() {
    let mut siv = Cursive::new();

    let v = TextView::empty().scrollable().with_name("asd");

    siv.add_layer(ResizedView::with_full_screen(
        LinearLayout::vertical()
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Free,
                TextView::empty().scrollable().with_name("output-area"),
            ))
            .child(
                ResizedView::with_full_width(
                    Dialog::around(
                        TextArea::new().scrollable()
                            .scroll_strategy(ScrollStrategy::StickToBottom).with_name("prompt-area"),
                    )
                    .title("Prompt")
                    .button("CLEAR INPUT", |s| {
                        s.call_on_name("prompt-area", |v: &mut NamedView<ScrollView<TextArea>>| {
                            v.get_mut().get_inner_mut().set_content("");
                        });
                    })
                    .button("CLEAR OUTPUT", |s| {
                        s.call_on_name("output-area", |v: &mut NamedView<ScrollView<TextView>>| {
                            v.get_mut().get_inner_mut().set_content("");
                        });
                    })
                    .button("Prompt", |s| {
                        let prompt = s
                            .call_on_name("prompt-area", |v: &mut NamedView<ScrollView<TextArea>>| {
                                v.get_mut().get_inner_mut().get_content().to_owned()
                            });
                        if let Some(prompt) = prompt {
                            stream_res_to_gui(s, prompt.clone());
                            show_message(s, format!("Prompt sent: {}", &prompt));
                        }
                    }),
                )
                .fixed_height(10),
            )
            .align_bottom_center(),
    ));

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
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

#[derive(Deserialize)]
struct PartialGapiResponse {
    data: PartialGapiResponseData,
}

#[derive(Deserialize)]
struct PartialGapiResponseData {
    candidates: Vec<Candidate>,
    // TODO: Usage metadata.
}

#[derive(Deserialize)]
struct GapiContent {
    parts: Vec<GapiPart>,
    role: String,
}

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

fn stream_res_to_gui(s: &mut Cursive, prompt: String) {
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
                    v.get_mut().get_inner_mut()
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
            parts: vec![GapiPart {
                text: msg,
            }],
        }],
    };

    let mut res = client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:streamGenerateContent?alt=sse")
        .header("x-goog-api-key", get_api_key())
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()?;

    Ok(res)
}

fn show_message(siv: &mut Cursive, message: impl ToString) {
    siv.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(message.to_string()))
            .child(DummyView)
            .child(Button::new("Ok", |s| {
                s.pop_layer();
            })),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_call_gemini() {
        let res = call_api("Hello, please give a very short response".to_owned());
        dbg!(&res);
        if let Ok(res) = res {
            std::fs::write("./example.txt", res.text().expect("Res is valid text").as_bytes());
        } else {
            panic!("Req failed")
        }
    }
}
