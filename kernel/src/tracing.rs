use core::{num::NonZeroU64, sync::atomic::{AtomicU64, Ordering::Relaxed}};

use alloc::{collections::BTreeMap, string::String};
use spin::Mutex;
use tracing::{Metadata, Subscriber, span};
use tracing_core::span::Current;

struct Span {
    metadata: &'static Metadata<'static>,
    fields: BTreeMap<&'static str, String>,
    parent: Option<span::Id>,
}

struct DebugVisitor(BTreeMap<&'static str, String>);

impl DebugVisitor {
    fn new() -> Self {
        DebugVisitor(BTreeMap::new())
    }
}

impl tracing::field::Visit for DebugVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        self.0.insert(field.name(), alloc::format!("{:?}", value));
    }
}

struct PrintVisitor;

impl tracing::field::Visit for PrintVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        // we kind of assume message gets recorded first otherwise it looks ugly
        match field.name() {
            "message" => print!("{:?} ", value),
            x => print!("{}={:?}, ", x, value),
        }
    }
}

pub struct PutcharSubscriber {
    spans: Mutex<BTreeMap<u64, Span>>,
    next: AtomicU64,
    current_span: AtomicU64,
}

impl PutcharSubscriber {
    pub fn new() -> Self {
        PutcharSubscriber {
            spans: spin::Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(1),
            current_span: AtomicU64::new(0),
        }
    }

    fn get_current_span(&self) -> Option<span::Id> {
        NonZeroU64::new(self.current_span.load(Relaxed)).map(span::Id::from_non_zero_u64)
    }
}

impl Subscriber for PutcharSubscriber {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, attrs: &span::Attributes<'_>) -> span::Id {
        let mut visitor = DebugVisitor::new();
        attrs.record(&mut visitor);
        let span = Span {
            metadata: attrs.metadata(),
            fields: visitor.0,
            parent: attrs.parent().cloned()
                .or(self.get_current_span()),
        };

        let id = self.next.fetch_add(1, Relaxed);

        self.spans.lock().insert(id, span);

        span::Id::from_u64(id)
    }

    fn record(&self, id: &span::Id, values: &span::Record<'_>) {
        let mut visitor = DebugVisitor::new();
        values.record(&mut visitor);
        self.spans.lock().get_mut(&id.into_u64()).unwrap().fields.append(&mut visitor.0);
    }

    fn record_follows_from(&self, _id: &span::Id, _follows: &span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let spans = self.spans.lock();
        let id = event.parent().cloned()
            .or(self.get_current_span());
        match id {
            Some(id) => {
                fn print_span_with_parents(spans: &BTreeMap<u64, Span>, span: &Span) {
                    if let Some(ref parent) = span.parent {
                        let span = spans.get(&parent.into_u64()).unwrap();
                        print_span_with_parents(spans, span);
                    }

                    print!("in {} ", span.metadata.name());
                    for (name, value) in span.fields.iter() {
                        print!("{}={} ", name, value);
                    }
                    println!();
                }

                let span = spans.get(&id.into_u64()).unwrap();
                print_span_with_parents(&spans, span);

                print!("  \\ {}: {} ", event.metadata().level(), event.metadata().name().trim_start_matches("event "));
            },
            None => print!("{}: {} ", event.metadata().level(), event.metadata().name().trim_start_matches("event ")),
        }
        event.record(&mut PrintVisitor);
        println!();
    }

    fn enter(&self, span: &span::Id) {
        self.current_span.store(span.into_u64(), Relaxed);
    }

    fn exit(&self, span: &span::Id) {
        let parent = self.spans.lock().get(&span.into_u64()).unwrap().parent.clone();
        self.current_span.store(
            parent.as_ref().map(span::Id::into_u64).unwrap_or(0),
            Relaxed);
    }

    fn current_span(&self) -> Current {
        match self.current_span.load(Relaxed) {
            0 => Current::none(),
            id => {
                let metadata = self.spans.lock().get(&id).unwrap().metadata;
                Current::new(span::Id::from_u64(id), metadata)
            },
        }
    }
}
