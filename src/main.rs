use cursive::{
    Cursive, CursiveExt,
    view::{Nameable, Resizable, Scrollable, SizeConstraint},
    views::{
        Dialog, LinearLayout, NamedView, ResizedView, ScrollView, TextArea, TextView,
    },
};
use cursive_aligned_view::Alignable;

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

fn lorem(letters: u32) -> String {
    let mut acc = String::new();
    let choose = "qwerttyuioipasdfgghjklzxcvbnm             ";
    for _ in 0..letters {
        let n = rand::random_range(0..(choose.len() - 1));
        acc.push(choose.chars().nth(n).expect("Indexing in range"));
    }

    acc
}
