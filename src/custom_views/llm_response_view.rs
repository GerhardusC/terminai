use cursive::{
    View,
    view::{Nameable, Scrollable, SizeConstraint},
    views::{NamedView, ResizedView, ScrollView, TextContent, TextView},
};

pub struct LlmResponseView {
    view: ResizedView<NamedView<ScrollView<TextView>>>,
}

impl LlmResponseView {
    pub fn new(content: TextContent) -> Self {
        let view = ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Free,
            TextView::new_with_content(content)
                .scrollable()
                .with_name("response-container"),
        );

        Self { view }
    }
}

impl View for LlmResponseView {
    fn draw(&self, printer: &cursive::Printer) {
        self.view.draw(printer);
    }

    fn layout(&mut self, size: cursive::Vec2) {
        self.view.layout(size);
    }

    fn needs_relayout(&self) -> bool {
        self.view.needs_relayout()
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        self.view.required_size(constraint)
    }

    fn on_event(&mut self, e: cursive::event::Event) -> cursive::event::EventResult {
        self.view.on_event(e)
    }

    fn call_on_any(&mut self, a: &cursive::view::Selector, b: cursive::event::AnyCb) {
        self.view.call_on_any(a, b);
    }

    fn focus_view(
        &mut self,
        v: &cursive::view::Selector,
    ) -> Result<cursive::event::EventResult, cursive::view::ViewNotFound> {
        self.view.focus_view(v)
    }

    fn take_focus(
        &mut self,
        source: cursive::direction::Direction,
    ) -> Result<cursive::event::EventResult, cursive::view::CannotFocus> {
        self.view.take_focus(source)
    }

    fn important_area(&self, view_size: cursive::Vec2) -> cursive::Rect {
        self.view.important_area(view_size)
    }

    fn type_name(&self) -> &'static str {
        self.view.type_name()
    }
}
