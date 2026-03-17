use cursive::{
    Cursive, CursiveExt,
    event::{Event, Key},
};
use terminai::custom_views::chatbot_view::ChatbotView;

fn main() {
    let mut siv = Cursive::new();

    let sink = siv.cb_sink().clone();

    siv.add_layer(ChatbotView::new(sink));

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
