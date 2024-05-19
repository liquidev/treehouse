use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::RwLock;
use std::{io::stderr, sync::mpsc, time::Duration};

use owo_colors::{AnsiColors, Color, OwoColorize};
use tracing::{span, Level};

pub struct TraceliConfig {
    update_interval: Duration,
}

impl Default for TraceliConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_millis(10),
        }
    }
}

impl TraceliConfig {
    /// Start a traceli thread in the background that will handle collecting and displaying logs
    /// in the command line.
    pub fn start(self) -> Traceli {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("Traceli printing thread".into())
            .spawn(move || Traceli::thread(self, rx))
            .expect("failed to spawn Traceli printing thread");
        Traceli {
            span_registry: RwLock::new(SpanRegistry {
                names: vec![],
                ids: HashMap::new(),
            }),
            tx,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpanData {
    name: &'static str,
}

struct SpanRegistry {
    names: Vec<SpanData>,
    ids: HashMap<SpanData, span::Id>,
}

pub struct Traceli {
    span_registry: RwLock<SpanRegistry>,
    tx: mpsc::Sender<Event>,
}

enum Event {
    Enter { name: &'static str },
    Exit,
    Log { text: String },
}

impl Traceli {
    fn thread(config: TraceliConfig, rx: mpsc::Receiver<Event>) {
        let mut stack = vec![];

        loop {
            for event in rx.try_iter() {
                match event {
                    Event::Enter { name } => {
                        stack.push(name);
                    }
                    Event::Exit => {
                        stack.pop();
                    }
                    Event::Log { text } => Self::render_log(&stack, &text),
                }
            }
            Self::render_stack(&stack);
            std::thread::sleep(config.update_interval);
        }
    }

    fn render_log(stack: &[&'static str], log: &str) {
        let mut stderr = stderr().lock();
        _ = write!(stderr, "\r\x1B[0K{log}\n");
        Self::render_stack(stack);
    }

    fn render_stack(stack: &[&'static str]) {
        let mut stderr = stderr().lock();
        _ = write!(stderr, "\r\x1B[0K");
        for crumb in stack {
            _ = write!(stderr, "{} {crumb} ", ">".dimmed());
        }
    }
}

impl tracing::Subscriber for Traceli {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let span_registry = self.span_registry.read().unwrap();
        let span_data = SpanData {
            name: span.metadata().name(),
        };
        if let Some(id) = span_registry.ids.get(&span_data) {
            id.clone()
        } else {
            drop(span_registry);
            let mut span_registry = self.span_registry.write().unwrap();
            let id = span::Id::from_u64(span_registry.names.len() as u64 + 1);
            span_registry.names.push(span_data.clone());
            span_registry.ids.insert(span_data, id.clone());
            id
        }
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let mut text = String::new();
        let level = match *event.metadata().level() {
            Level::TRACE => "trace:",
            Level::DEBUG => "debug:",
            Level::INFO => "info:",
            Level::WARN => "warning:",
            Level::ERROR => "error:",
        };
        let color = match *event.metadata().level() {
            Level::TRACE => AnsiColors::Magenta,
            Level::DEBUG => AnsiColors::Blue,
            Level::INFO => AnsiColors::BrightGreen,
            Level::WARN => AnsiColors::BrightYellow,
            Level::ERROR => AnsiColors::BrightRed,
        };
        _ = write!(text, "{}", level.color(color));

        struct Visitor<'a> {
            text: &'a mut String,
        }

        impl<'a> tracing::field::Visit for Visitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    _ = write!(self.text, " {:?}", value);
                } else {
                    _ = write!(
                        self.text,
                        " {}{}{:?}",
                        field.name().dimmed(),
                        " = ".dimmed(),
                        value
                    );
                }
            }
        }

        event.record(&mut Visitor { text: &mut text });

        _ = self.tx.send(Event::Log { text });
    }

    fn enter(&self, span: &span::Id) {
        let span_registry = self.span_registry.read().unwrap();
        _ = self.tx.send(Event::Enter {
            name: span_registry.names[span.into_u64() as usize - 1].name,
        });
    }

    fn exit(&self, span: &span::Id) {
        _ = self.tx.send(Event::Exit);
    }
}
