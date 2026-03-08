use cursive::{
    Cursive,
    views::{Button, Dialog, DummyView, LinearLayout, TextView},
};

pub fn show_message(siv: &mut Cursive, message: impl ToString) {
    siv.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(message.to_string()))
            .child(DummyView)
            .child(Button::new("Ok", |s| {
                s.pop_layer();
            })),
    ));
}
