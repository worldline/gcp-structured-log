// NOTE: This is in a separate file in order to preserve line positions as much as possible
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use tracing::*;
use tracing_subscriber::{Registry, layer::SubscriberExt};

use crate::*;

#[test]
fn info_simple() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    info!("Lorem ipsum");

    assert_eq!(
        r#"{"severity":"INFO","message":"Lorem ipsum","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":20,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn warn_with_fields() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    warn!(foo = "bar", qux = 42, "Lorem ipsum");

    assert_eq!(
        r#"{"severity":"WARNING","message":"Lorem ipsum","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"foo":"bar","hostname":"test-hostname","pid":1,"qux":42},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":36,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn http_event() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    trace!(
        http.method = "POST",
        http.url = "https://www.disney.com",
        r#"Http::connect; scheme=Some("http"), host=Some("127.0.0.1"), port=Some(Port(43059))"#
    );

    assert_eq!(
        r#"{"severity":"DEFAULT","message":"Http::connect; scheme=Some(\"http\"), host=Some(\"127.0.0.1\"), port=Some(Port(43059))","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"hostname":"test-hostname","http.method":"POST","http.url":"https://www.disney.com","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":52,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn source_location() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    debug!("Where is this coming from?");

    assert_eq!(
        r#"{"severity":"DEBUG","message":"Where is this coming from?","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":72,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn span() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    info!("Outside span, before");
    foo();
    // info!("Outside span, after");

    assert_eq!(
        r#"{"severity":"INFO","message":"Outside span, before","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":88,"function":"tracing_gcp_formatter::tests"}}
{"severity":"INFO","message":"[FOO - START]","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"bar":"baz","hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":102,"function":"tracing_gcp_formatter::tests"}}
{"severity":"WARNING","message":"[FOO - EVENT] Inside span","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"bar":"baz","hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":104,"function":"tracing_gcp_formatter::tests"}}
{"severity":"INFO","message":"[FOO - END]","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"bar":"baz","hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":102,"function":"tracing_gcp_formatter::tests"}}"#,
        // {"severity":"INFO","message":"Outside span, after","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"hostname":"test-hostname","pid":1},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":82,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[tracing::instrument(fields(bar = "baz"))]
fn foo() {
    warn!("Inside span");
}

struct TestMakeWriter {
    writer: TestWriter,
}

impl TestMakeWriter {
    fn new() -> (Self, TestWriter) {
        let writer = TestWriter::new();
        (
            Self {
                writer: writer.clone(),
            },
            writer,
        )
    }
}

impl<'a> MakeWriter<'a> for TestMakeWriter {
    type Writer = TestWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.writer.clone()
    }
}

#[derive(Clone)]
struct TestWriter {
    output: Arc<Mutex<Vec<u8>>>,
}

impl TestWriter {
    fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn output(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().expect("Getting lock"))
            .trim_end_matches('\n')
            .to_owned()
    }
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output
            .lock()
            .expect("Getting lock")
            .write(buf)
            .expect("Writing to output");

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
