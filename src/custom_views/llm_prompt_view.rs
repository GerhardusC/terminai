use std::sync::mpsc::Sender;

use cursive::{
    View,
    view::{ScrollStrategy, Scrollable},
    views::{Dialog, NamedView, ResizedView, TextArea},
};

use crate::llm_context::LlmContextUpdateMessage;

pub struct LlmPromptView {
    view: ResizedView<Dialog>,
}

type PromptArea = NamedView<TextArea>;

impl LlmPromptView {
    pub fn new(sender: Sender<LlmContextUpdateMessage>, prompt_area: PromptArea) -> Self {
        let sender_p = sender.clone();
        let sender_t = sender.clone();
        let sender_ctx = sender.clone();
        let sender_clear_ctx = sender.clone();
        let sender_clear_prompt = sender.clone();
        let sender_clear_output = sender.clone();

        let view = ResizedView::with_full_width(
            Dialog::around(
                prompt_area
                    .scrollable()
                    .scroll_strategy(ScrollStrategy::StickToBottom),
            )
            .button("PROMPT", move |_| {
                let _ = sender_p.send(LlmContextUpdateMessage::AddUserPrompt);
            })
            .button("TOGGLE THOUGHTS", move |_| {
                let _ = sender_t.send(LlmContextUpdateMessage::ToggleThoughts);
            })
            .button("CLEAR PROMPT", move |_| {
                let _ = sender_clear_prompt.send(LlmContextUpdateMessage::ClearPrompt);
            })
            .button("CLEAR OUTPUT", move |_| {
                let _ = sender_clear_output.send(LlmContextUpdateMessage::ClearOutput);
            })
            .button("CLEAR CONTEXT", move |_| {
                let _ = sender_clear_ctx.send(LlmContextUpdateMessage::ClearContext);
            })
            .button("VIEW CONTEXT", move |_| {
                let _ = sender_ctx.send(LlmContextUpdateMessage::ViewCurrentContext);
            }),
        );

        Self { view }
    }
}

impl View for LlmPromptView {
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
