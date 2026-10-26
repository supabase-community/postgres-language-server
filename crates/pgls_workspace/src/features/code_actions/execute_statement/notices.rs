//! sqlx has no handler for the notices Postgres sends, such as `raise notice` or a warning. It
//! only reports them as tracing events with the target `sqlx::postgres::notice`, so the statement
//! runs with a subscriber that collects these events and passes everything else on.

use std::fmt;
use std::sync::{Arc, Mutex};

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Dispatch, Event, Level, Metadata, Subscriber};

const NOTICE_TARGET: &str = "sqlx::postgres::notice";

/// Collects the notices of the code that runs with [`Self::dispatch`] as its subscriber.
#[derive(Clone, Default)]
pub(super) struct NoticeCollector {
    notices: Arc<Mutex<Vec<String>>>,
}

impl NoticeCollector {
    /// A subscriber that collects the notices and passes every other event and span on to the
    /// current one.
    pub(super) fn dispatch(&self) -> Dispatch {
        Dispatch::new(Collector {
            inner: tracing::dispatcher::get_default(Dispatch::clone),
            notices: Arc::clone(&self.notices),
        })
    }

    /// The collected notices, each as `SEVERITY: message`.
    pub(super) fn take(&self) -> Vec<String> {
        std::mem::take(&mut self.notices.lock().unwrap())
    }
}

struct Collector {
    inner: Dispatch,
    notices: Arc<Mutex<Vec<String>>>,
}

fn is_notice(metadata: &Metadata<'_>) -> bool {
    metadata.target() == NOTICE_TARGET
}

impl Subscriber for Collector {
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if is_notice(metadata) {
            Interest::always()
        } else {
            self.inner.register_callsite(metadata)
        }
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        is_notice(metadata) || self.inner.enabled(metadata)
    }

    fn event(&self, event: &Event<'_>) {
        let metadata = event.metadata();
        if is_notice(metadata) {
            let mut visitor = MessageVisitor(None);
            event.record(&mut visitor);
            if let Some(message) = visitor.0 {
                // sqlx maps the severity to a level; INFO and LOG both become TRACE.
                let severity = match *metadata.level() {
                    Level::ERROR => "ERROR",
                    Level::WARN => "WARNING",
                    Level::INFO => "NOTICE",
                    Level::DEBUG => "DEBUG",
                    Level::TRACE => "INFO",
                };
                self.notices
                    .lock()
                    .unwrap()
                    .push(format!("{severity}: {message}"));
            }
            if !self.inner.enabled(metadata) {
                return;
            }
        }
        self.inner.event(event);
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        self.inner.new_span(span)
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        self.inner.record(span, values);
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.inner.record_follows_from(span, follows);
    }

    fn enter(&self, span: &Id) {
        self.inner.enter(span);
    }

    fn exit(&self, span: &Id) {
        self.inner.exit(span);
    }

    fn clone_span(&self, id: &Id) -> Id {
        self.inner.clone_span(id)
    }

    fn try_close(&self, id: Id) -> bool {
        self.inner.try_close(id)
    }
}

struct MessageVisitor(Option<String>);

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0 = Some(format!("{value:?}"));
        }
    }
}
