use std::sync::{Arc, Mutex};

use cursive::{
    Cursive, CursiveExt,
    view::{Nameable, Resizable, ScrollStrategy, Scrollable, SizeConstraint},
    views::{Dialog, LinearLayout, NamedView, ResizedView, ScrollView, TextArea, TextView},
};
use cursive_aligned_view::Alignable;
use terminai::models::{LlmContext, LlmContextManager, Message, ollama};

fn main() {
    let mut siv = Cursive::new();

    let context = Arc::new(Mutex::new(LlmContext::new()));
    let context1 = context.clone();

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
                        TextArea::new()
                            .scrollable()
                            .scroll_strategy(ScrollStrategy::StickToBottom)
                            .with_name("prompt-area"),
                    )
                    .title("Prompt")
                    .button("PROMPT", move |s| {
                        let prompt = s.call_on_name(
                            "prompt-area",
                            |v: &mut NamedView<ScrollView<TextArea>>| {
                                v.get_mut().get_inner_mut().get_content().to_owned()
                            },
                        );
                        if let Some(prompt) = prompt {
                            context
                                .clone()
                                .add_message(Message::new("user".to_owned(), prompt));
                            ollama::stream_res_to_gui(s, context.clone());
                        }
                    })
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
                    .button("CLEAR CONTEXT", move |_s| {
                        context1.clear();
                    }),
                )
                .fixed_height(10),
            )
            .with_name("prompt-container")
            .align_bottom_center(),
    ));

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
