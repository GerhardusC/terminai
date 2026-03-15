use std::sync::{Arc, Mutex};

use cursive::{
    Cursive, CursiveExt,
    event::{Event, Key},
    view::{Nameable, Resizable, ScrollStrategy, Scrollable, SizeConstraint},
    views::{
        Dialog, LinearLayout, NamedView, ResizedView, ScrollView, TextArea, TextContent, TextView,
    },
};
use cursive_aligned_view::Alignable;
use terminai::{
    custom_views::{llm_prompt_view::LlmPromptView, llm_response_view::LlmResponseView},
    llm_context::{LlmContext, Message, Role},
    models::ollama::{self, LlmContextManager},
};

fn main() {
    let mut siv = Cursive::new();

    let sink = siv.cb_sink().clone();

    let context = Arc::new(Mutex::new(LlmContext::new(sink, TextContent::new(""))));
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
                                .add_message(Message::new(Role::User, prompt));
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

    let main_screen_id = siv.active_screen();
    let test_screen_id = siv.add_screen();

    siv.set_screen(test_screen_id);
    let sink = siv.cb_sink().clone();

    let text_content = TextContent::new("");
    let llm_context = LlmContext::new(sink, text_content.clone());
    let llm_prompt_view = LlmPromptView::new(llm_context.update_tx.clone());
    let llm_response_view = LlmResponseView::new(text_content);

    llm_context.start();

    siv.add_layer(
        LinearLayout::vertical()
            .child(llm_response_view)
            .child(LinearLayout::horizontal().with_name("loading-view"))
            .child(llm_prompt_view),
    );

    siv.add_global_callback(Event::Key(Key::F1), move |s| {
        s.set_screen(main_screen_id);
    });

    siv.add_global_callback(Event::Key(Key::F2), move |s| {
        s.set_screen(test_screen_id);
    });

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
