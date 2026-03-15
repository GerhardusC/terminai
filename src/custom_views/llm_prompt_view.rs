use std::sync::mpsc::Sender;

use cursive::{
    View,
    view::{Nameable, ScrollStrategy, Scrollable},
    views::{Dialog, NamedView, ResizedView, TextArea},
};

use crate::llm_context::{LlmContextUpdateMessage, Message, Role};

pub struct LlmPromptView {
    view: ResizedView<Dialog>,
    pub update_tx: Sender<LlmContextUpdateMessage>,
}

impl LlmPromptView {
    pub fn new(sender: Sender<LlmContextUpdateMessage>) -> Self {
        let sender_p = sender.clone();
        let sender_ctx = sender_p.clone();
        let view = ResizedView::with_full_width(
            Dialog::around(
                TextArea::new()
                    .with_name("prompt-area")
                    .scrollable()
                    .scroll_strategy(ScrollStrategy::StickToBottom),
            )
            .button("PROMPT", move |s| {
                if let Some(prompt) = s
                    .call_on_name("prompt-area", move |v: &mut NamedView<TextArea>| {
                        v.get_mut().get_content().to_owned()
                    })
                {
                    let _ = sender_p.send(LlmContextUpdateMessage::AddMessage(Message::new(
                        Role::User,
                        prompt,
                    )));
                    let _ = sender_p.send(LlmContextUpdateMessage::CallApi);
                };
            })
            .button("CLEAR", move |s| {
                s.call_on_name("prompt-area", move |v: &mut NamedView<TextArea>| {
                    v.get_mut().set_content("");
                });
            })
            .button("VIEW CONTEXT", move |_| {
                let _ = sender_ctx.send(LlmContextUpdateMessage::ViewCurrentContext);
            }),
        );

        Self {
            view,
            update_tx: sender,
        }
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
