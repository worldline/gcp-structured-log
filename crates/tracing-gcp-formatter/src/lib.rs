use std::{borrow::Cow, io::Write};

use chrono::{DateTime, Utc};
use tracing::{Level, Metadata, Subscriber, field::Visit, span};
use tracing_subscriber::{Layer, fmt::MakeWriter};

use crate::models::{Severity, SimplifiedLogEntry, SourceLocation};

mod models;

pub struct GCPFormattingLayer<W: for<'a> MakeWriter<'a> + 'static> {
    make_writer: W,
    pid: u32,
    hostname: Option<String>,
}

impl<W> GCPFormattingLayer<W>
where
    W: for<'a> MakeWriter<'a> + 'static,
{
    pub fn new(make_writer: W) -> Self {
        Self {
            make_writer,
            pid: Self::pid(),
            hostname: Self::hostname(),
        }
    }

    #[cfg(test)]
    fn pid() -> u32 {
        1
    }

    #[cfg(not(test))]
    fn pid() -> u32 {
        std::process::id()
    }

    fn hostname() -> Option<String> {
        match (cfg!(test), cfg!(feature = "hostname")) {
            (true, _) => Some("test-hostname".to_owned()),
            (_, true) => Some(gethostname::gethostname().to_string_lossy().into_owned()),
            (_, false) => None,
        }
    }

    #[cfg(test)]
    fn now() -> DateTime<Utc> {
        chrono::DateTime::<Utc>::UNIX_EPOCH
    }

    #[cfg(not(test))]
    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn emit(&self, entry: &SimplifiedLogEntry, meta: &Metadata<'_>) -> Result<(), std::io::Error> {
        let buffer = {
            let mut b = serde_json::to_string(entry).expect("Serializing SimplifiedLogEntry");
            b.push('\n');
            b
        };

        self.make_writer
            .make_writer_for(meta)
            .write_all(buffer.as_bytes())
    }
}

impl<S, W> Layer<S> for GCPFormattingLayer<W>
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'a> MakeWriter<'a> + 'static,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);

        let message = match (visitor.message, ctx.lookup_current()) {
            (Some(message), Some(span)) => {
                format!("[{} - EVENT] {}", span.name().to_uppercase(), message)
            }
            (_, Some(span)) => format!("[{} - EVENT]", span.name().to_uppercase()),
            (Some(message), _) => message,
            _ => "[EVENT]".to_owned(),
        };

        let entry = SimplifiedLogEntry {
            severity: map_severity(*event.metadata().level()),
            time: Self::now(),
            message: Cow::Borrowed(&message),
            labels: map_labels(visitor.other_fields, self.pid, self.hostname.as_deref()),
            source_location: map_source_location(
                event.metadata().file(),
                event.metadata().module_path(),
                event.metadata().line(),
            ),
            ..Default::default()
        };

        let _ = self.emit(&entry, event.metadata());
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        _id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = EventVisitor::default();
        attrs.record(&mut visitor);

        let entry = SimplifiedLogEntry {
            severity: map_severity(*attrs.metadata().level()),
            time: Self::now(),
            message: Cow::Borrowed(&format!(
                "[{} - START]",
                attrs.metadata().name().to_uppercase()
            )),
            labels: map_labels(visitor.other_fields, self.pid, self.hostname.as_deref()),
            source_location: map_source_location(
                attrs.metadata().file(),
                attrs.metadata().module_path(),
                attrs.metadata().line(),
            ),
            ..Default::default()
        };

        let _ = self.emit(&entry, attrs.metadata());
    }

    fn on_close(&self, id: span::Id, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let Some(span) = ctx.span(&id) else {
            return;
        };

        let entry = SimplifiedLogEntry {
            severity: map_severity(*span.metadata().level()),
            time: Self::now(),
            message: Cow::Borrowed(&format!(
                "[{} - END]",
                span.metadata().name().to_uppercase(),
            )),
            labels: None,
            source_location: map_source_location(
                span.metadata().file(),
                span.metadata().module_path(),
                span.metadata().line(),
            ),
            ..Default::default()
        };

        let _ = self.emit(&entry, span.metadata());
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

fn map_labels(
    other_fields: Vec<(&str, serde_json::Value)>,
    pid: u32,
    hostname: Option<&str>,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let mut labels = serde_json::Map::new();
    labels.insert("pid".to_owned(), serde_json::json!(pid));
    if let Some(hostname) = hostname {
        labels.insert("hostname".to_owned(), serde_json::json!(hostname));
    }
    for (key, value) in other_fields {
        labels.insert(key.to_owned(), value);
    }
    Some(labels)
}

#[derive(Default, Debug)]
struct EventVisitor<'a> {
    message: Option<String>,
    other_fields: Vec<(&'a str, serde_json::Value)>,
}

impl<'a> Visit for EventVisitor<'a> {
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.other_fields.push((field.name(), value.into()))
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.other_fields.push((field.name(), value.into()))
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.other_fields.push((field.name(), value.into()))
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.other_fields
            .push((field.name(), value.trim_matches('"').into()))
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.other_fields.push((field.name(), value.into()))
    }

    fn record_bytes(&mut self, field: &tracing::field::Field, value: &[u8]) {
        self.other_fields.push((field.name(), value.into()))
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        match field.name() {
            "message" => self.message = Some(format!("{value:?}")),
            name => self.other_fields.push((name, format!("{value:?}").into())),
        }
    }
}

#[cfg(test)]
mod tests;
