use std::io::Write;

use chrono::Utc;
use tracing::{Level, Subscriber, field::Visit};
use tracing_subscriber::{Layer, fmt::MakeWriter};

use crate::models::{Severity, SimplifiedLogEntry, SourceLocation};

mod models;

pub struct GCPFormattingLayer<W: for<'a> MakeWriter<'a> + 'static> {
    make_writer: W,
}

impl<W> GCPFormattingLayer<W>
where
    W: for<'a> MakeWriter<'a> + 'static,
{
    pub fn new(make_writer: W) -> Self {
        Self { make_writer }
    }
}

impl<S, W> Layer<S> for GCPFormattingLayer<W>
where
    S: Subscriber,
    W: for<'a> MakeWriter<'a> + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);

        let log_entry = SimplifiedLogEntry {
            severity: map_severity(*event.metadata().level()),
            #[cfg(test)]
            time: chrono::DateTime::<Utc>::UNIX_EPOCH,
            #[cfg(not(test))]
            time: Utc::now(),
            message: std::borrow::Cow::Borrowed(visitor.message.as_ref().map_or("", |i| i)),
            labels: {
                let mut labels = serde_json::Map::new();
                for (key, value) in visitor.other_fields {
                    labels.insert(
                        key,
                        serde_json::Value::String(value.trim_matches('"').to_owned()),
                    );
                }
                if labels.is_empty() {
                    None
                } else {
                    Some(labels)
                }
            },
            source_location: map_source_location(
                event.metadata().file(),
                event.metadata().module_path(),
                event.metadata().line(),
            ),
            ..Default::default()
        };

        let buffer = {
            let mut b = serde_json::to_string(&log_entry).expect("Serializing SimplifiedLogEntry");
            b.push('\n');
            b
        };

        self.make_writer
            .make_writer_for(event.metadata())
            .write_all(buffer.as_bytes())
            .expect("Writing to std::io::Write instance");
    }
}

fn map_severity(level: Level) -> Severity {
    match level {
        Level::TRACE => Severity::Default,
        Level::DEBUG => Severity::Debug,
        Level::INFO => Severity::Info,
        Level::WARN => Severity::Warning,
        Level::ERROR => Severity::Error,
    }
}

fn map_source_location(
    file: Option<&str>,
    function: Option<&str>,
    line: Option<u32>,
) -> Option<SourceLocation> {
    match (file, function, line) {
        (None, None, None) => None,
        _ => Some(SourceLocation {
            file: file.map(|i| i.to_owned()),
            function: function.map(|i| i.to_owned()),
            line,
        }),
    }
}

#[derive(Default, Debug)]
struct EventVisitor {
    message: Option<String>,
    other_fields: Vec<(String, String)>,
}

impl Visit for EventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        match field.name() {
            "message" => self.message = Some(format!("{value:?}")),
            name => self
                .other_fields
                .push((name.to_owned(), format!("{value:?}"))),
        }
    }
}

#[cfg(test)]
mod tests;
