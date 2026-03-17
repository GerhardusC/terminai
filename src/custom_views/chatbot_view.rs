use crate::{
    custom_views::{llm_prompt_view::LlmPromptView, llm_response_view::LlmResponseView},
    llm_context::LlmContext,
};
use cursive::{CbSink, View, views::LinearLayout};

pub struct ChatbotView {
    view: LinearLayout,
}

impl ChatbotView {
    pub fn new(sink: CbSink) -> Self {
        let llm_context = LlmContext::new(sink);

        let (update_tx, text_content) = (
            llm_context.update_tx.clone(),
            llm_context.text_content.clone(),
        );

        let (prompt_area, loading_area) = llm_context.start();

        let llm_prompt_view = LlmPromptView::new(update_tx, prompt_area);
        let llm_response_view = LlmResponseView::new(text_content);

        let view = LinearLayout::vertical()
            .child(llm_response_view)
            .child(loading_area)
            .child(llm_prompt_view);

        Self { view }
    }
}

impl View for ChatbotView {
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
