use cursive::{
    Cursive, CursiveExt,
    event::{Event, Key},
    view::Nameable,
    views::{LinearLayout, TextContent},
};
use terminai::{
    custom_views::{llm_prompt_view::LlmPromptView, llm_response_view::LlmResponseView},
    llm_context::LlmContext,
};

fn main() {
    let mut siv = Cursive::new();

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

    let main_screen_id = siv.active_screen();
    let test_screen_id = siv.add_screen();

    siv.add_global_callback(Event::Key(Key::F1), move |s| {
        s.set_screen(main_screen_id);
    });

    siv.add_global_callback(Event::Key(Key::F2), move |s| {
        s.set_screen(test_screen_id);
    });

    siv.add_global_callback('q', |s| s.quit());
    siv.run();
}
