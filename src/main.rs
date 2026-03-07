use anyhow::Result;
use std::{io::Read, sync::OnceLock};

use cursive::{
    Cursive, CursiveExt,
    view::{Nameable, Resizable, Scrollable, SizeConstraint},
    views::{Dialog, LinearLayout, NamedView, ResizedView, ScrollView, TextArea, TextView},
};
use cursive_aligned_view::Alignable;
use serde::Serialize;

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

    siv.add_layer(ResizedView::with_full_screen(
        LinearLayout::vertical()
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Free,
                TextView::new(lorem(0xFFF))
                    .with_name("output-area")
                    .scrollable(),
            ))
            .child(
                ResizedView::with_full_width(
                    Dialog::around(ScrollView::new(
                        TextArea::new().with_name("prompt-area").scrollable(),
                    ))
                    .title("Prompt")
                    .button("CLEAR", |s| {
                        s.call_on_name("prompt-area", |v: &mut NamedView<TextArea>| {
                            v.get_mut().set_content("");
                        });
                    }),
                )
                .fixed_height(10),
            )
            .align_bottom_center(),
    ));

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}

fn call_api() -> Result<String> {
    let client = reqwest::blocking::Client::new();

    let req_body = GapiReqBody {
        contents: vec![GapiReqParts {
            parts: GapiReqPart {
                text: "Hello! Please can you explain to me how to use the gemini api.".to_owned(),
            },
        }],
    };

    let mut res = client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:streamGenerateContent?alt=sse")
        .header("x-goog-api-key", get_api_key())
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()?;

    let mut acc = Vec::new();
    let mut buf = [0; 0xFF];
    while let Ok(x) = res.read(&mut buf)
        && x > 0
    {
        acc.extend_from_slice(&buf[..x]);
        println!("Bytes read: {}: {:?}", x, String::from_utf8(buf.to_vec()));
        buf.fill(0);
    }

    Ok(String::from_utf8(acc)?)
}

#[derive(Serialize)]
struct GapiReqBody {
    contents: Vec<GapiReqParts>,
}

#[derive(Serialize)]
struct GapiReqParts {
    parts: GapiReqPart,
}

#[derive(Serialize)]
struct GapiReqPart {
    text: String,
}

fn lorem(letters: u32) -> String {
    let mut acc = String::new();
    let choose = "qwerttyuioipasdfgghjklzxcvbnm             ";
    for _ in 0..letters {
        let n = rand::random_range(0..(choose.len() - 1));
        acc.push(choose.chars().nth(n).expect("Indexing in range"));
    }

    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_call_gemini() {
        let res = call_api();
        dbg!(&res);
        assert!(true);
    }
}
