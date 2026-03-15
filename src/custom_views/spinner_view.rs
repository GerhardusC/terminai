use std::{
    iter::Cycle,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
    vec::IntoIter,
};

use cursive::{
    CbSink, View,
    views::{TextContent, TextView},
};

pub struct SpinnerStopSignal;

pub struct Spinner {
    cycle: Cycle<IntoIter<String>>,
    cb_sink: CbSink,
    content: TextContent,
    spin_receiver: Receiver<SpinnerStopSignal>,
    delay: Duration,
}

pub struct SpinnerView {
    view: TextView,
    spin_sender: Sender<SpinnerStopSignal>,
}

impl Spinner {
    pub fn spin(mut self) {
        thread::spawn(move || {
            let mut x = self.cycle.next();
            loop {
                let res = self.spin_receiver.recv_timeout(self.delay);

                if res.is_ok() {
                    break;
                } else {
                    let current_msg = x.unwrap_or("⠙".to_string()).clone();
                    let content_ref = self.content.clone();
                    let _ = self.cb_sink.send(Box::new(move |_| {
                        content_ref.set_content(current_msg);
                    }));
                    x = self.cycle.next();
                };
            }
        });
    }
}

impl SpinnerView {
    pub fn new(cb_sink: CbSink, delay: Duration) -> Self {
        let content = TextContent::new("⠙");
        let text_view = TextView::new_with_content(content.clone());

        let (tx, rx) = mpsc::channel::<SpinnerStopSignal>();

        let spinner = Spinner {
            cycle: vec![
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ]
            .into_iter()
            .cycle(),
            cb_sink,
            content,
            spin_receiver: rx,
            delay,
        };

        spinner.spin();

        Self {
            view: text_view,
            spin_sender: tx,
        }
    }
}

impl View for SpinnerView {
    fn draw(&self, printer: &cursive::Printer) {
        self.view.draw(printer)
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
}

impl Drop for SpinnerView {
    fn drop(&mut self) {
        let _ = self.spin_sender.send(SpinnerStopSignal);
    }
}
