use std::{borrow::Cow, io::Write};

use chrono::{DateTime, Utc};
use tracing::{Level, Metadata, Subscriber, field::Visit, span};
use tracing_subscriber::{Layer, fmt::MakeWriter, registry::LookupSpan};

use crate::models::{Severity, SimplifiedLogEntry, SourceLocation};

mod models;

/// A [`tracing_subscriber::Layer`] that formats tracing events and spans as
/// [GCP Structured Logging] JSON entries and writes them using the provided
/// [`MakeWriter`].
///
/// Each tracing event or span lifecycle transition produces one newline-delimited
/// JSON log entry with the following GCP fields:
///
/// - `severity` — mapped from the tracing [`Level`] (see table below)
/// - `message` — the event message, prefixed with the span name when emitted inside a span
/// - `time` — RFC 3339 UTC timestamp
/// - `logging.googleapis.com/labels` — always contains `pid`; contains `hostname` when the
///   `hostname` feature is enabled (on by default); also includes all span and event fields
/// - `logging.googleapis.com/sourceLocation` — file path, line number, and module path
///   derived from tracing metadata
///
/// ## Severity mapping
///
/// | `tracing` level | GCP severity |
/// |-----------------|--------------|
/// | `TRACE`         | `DEFAULT`    |
/// | `DEBUG`         | `DEBUG`      |
/// | `INFO`          | `INFO`       |
/// | `WARN`          | `WARNING`    |
/// | `ERROR`         | `ERROR`      |
///
/// ## Message format
///
/// | Context             | Emitted `message`             |
/// |---------------------|-------------------------------|
/// | Event outside span  | `message`                     |
/// | Event inside span   | `[SPAN_NAME - EVENT] message` |
/// | Span opens          | `[SPAN_NAME - START]`         |
/// | Span closes         | `[SPAN_NAME - END]`           |
///
/// ## Usage
///
/// `GCPFormattingLayer` must be paired with [`SpanDataLayer`] so that span fields are
/// available when formatting events emitted inside a span. Always add [`SpanDataLayer`]
/// **before** `GCPFormattingLayer` in the subscriber stack.
///
/// ```no_run
/// use tracing_subscriber::{Registry, layer::SubscriberExt};
/// use tracing_gcp_formatter::{GCPFormattingLayer, SpanDataLayer};
///
/// let subscriber = Registry::default()
///     .with(SpanDataLayer::new())
///     .with(GCPFormattingLayer::new(std::io::stdout));
///
/// tracing::subscriber::set_global_default(subscriber).expect("setting subscriber");
/// ```
///
/// [GCP Structured Logging]: https://cloud.google.com/logging/docs/structured-logging
pub struct GCPFormattingLayer<W: for<'a> MakeWriter<'a> + 'static> {
    make_writer: W,
    pid: u32,
    hostname: Option<String>,
}

impl<W> GCPFormattingLayer<W>
where
    W: for<'a> MakeWriter<'a> + 'static,
{
    /// Creates a new `GCPFormattingLayer` that writes JSON log entries using `make_writer`.
    ///
    /// `make_writer` can be any type implementing
    /// [`MakeWriter`], such as [`std::io::stdout`] or [`std::io::stderr`].
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

        if let Some(span) = ctx.event_span(event) {
            let mut current_span = Some(span);

            while let Some(span) = current_span {
                let extensions = span.extensions();
                if let Some(span_fields) = extensions.get::<SpanFields>() {
                    for (name, value) in span_fields.fields.iter() {
                        visitor.other_fields.push((name.to_owned(), value.clone()));
                    }
                }
                current_span = span.parent().and_then(|i| ctx.span(&i.id()));
            }
        }

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

        let labels = {
            let mut l = Vec::<(String, serde_json::Value)>::new();

            let mut current_span_id = Some(id);

            while let Some(span) = current_span_id.and_then(|i| ctx.span(&i)) {
                let extensions = span.extensions();
                if let Some(span_fields) = extensions.get::<SpanFields>() {
                    for (name, value) in span_fields.fields.iter() {
                        l.push((name.to_owned(), value.clone()));
                    }
                }

                current_span_id = span.parent().map(|i| i.id().clone());
            }

            l
        };

        let entry = SimplifiedLogEntry {
            severity: map_severity(*span.metadata().level()),
            time: Self::now(),
            message: Cow::Borrowed(&format!(
                "[{} - END]",
                span.metadata().name().to_uppercase()
            )),
            labels: map_labels(labels, self.pid, self.hostname.as_deref()),
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
            line: line.map(|i| i.to_string()),
        }),
    }
}

fn map_labels(
    labels: Vec<(String, serde_json::Value)>,
    pid: u32,
    hostname: Option<&str>,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let mut output = serde_json::Map::new();
    output.insert("pid".to_owned(), serde_json::json!(pid));
    if let Some(hostname) = hostname {
        output.insert("hostname".to_owned(), serde_json::json!(hostname));
    }
    for (key, value) in labels {
        output.insert(key, value);
    }
    Some(output)
}

#[derive(Default, Debug)]
struct EventVisitor {
    message: Option<String>,
    other_fields: Vec<(String, serde_json::Value)>,
}

impl Visit for EventVisitor {
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.other_fields
            .push((field.name().to_owned(), value.into()))
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.other_fields
            .push((field.name().to_owned(), value.into()))
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.other_fields
            .push((field.name().to_owned(), value.into()))
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.other_fields
            .push((field.name().to_owned(), value.trim_matches('"').into()))
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.other_fields
            .push((field.name().to_owned(), value.into()))
    }

    fn record_bytes(&mut self, field: &tracing::field::Field, value: &[u8]) {
        self.other_fields
            .push((field.name().to_owned(), value.into()))
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        match field.name() {
            "message" => self.message = Some(format!("{value:?}")),
            name => self
                .other_fields
                .push((name.to_owned(), format!("{value:?}").into())),
        }
    }
}

/// A [`tracing_subscriber::Layer`] that captures span fields and stores them in span
/// extensions so that [`GCPFormattingLayer`] can include them in log entries emitted
/// from within those spans.
///
/// When an event fires inside a span, [`GCPFormattingLayer`] walks the span ancestry and
/// collects all fields recorded by this layer, merging them into the `labels` of the
/// produced JSON log entry. This layer carries no state of its own and is cheap to construct.
///
/// ## Usage
///
/// Always add `SpanDataLayer` **before** [`GCPFormattingLayer`] in the subscriber stack.
/// The ordering matters: `SpanDataLayer` must store the span data before
/// `GCPFormattingLayer` can read it.
///
/// ```no_run
/// use tracing_subscriber::{Registry, layer::SubscriberExt};
/// use tracing_gcp_formatter::{GCPFormattingLayer, SpanDataLayer};
///
/// let subscriber = Registry::default()
///     .with(SpanDataLayer::new())
///     .with(GCPFormattingLayer::new(std::io::stdout));
///
/// tracing::subscriber::set_global_default(subscriber).expect("setting subscriber");
/// ```
pub struct SpanDataLayer {}

impl SpanDataLayer {
    /// Creates a new `SpanDataLayer`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SpanDataLayer {
    fn default() -> Self {
        Self::new()
    }
}

struct SpanFields {
    fields: Vec<(String, serde_json::Value)>,
}

impl<S> Layer<S> for SpanDataLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = EventVisitor::default();
        attrs.record(&mut visitor);

        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();

            extensions.insert(SpanFields {
                fields: visitor
                    .other_fields
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
