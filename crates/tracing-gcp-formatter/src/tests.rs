// NOTE: This is in a separate file in order to minimize line positions changes that brake all
// tests
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
    let sub = Registry::default().with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    info!("Lorem ipsum");

    assert_eq!(
        r#"{"severity":"INFO","message":"Lorem ipsum","time":"1970-01-01T00:00:00Z","logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":19,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn warn_with_fields() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default().with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    warn!(foo = "bar", qux = 42, "Lorem ipsum");

    assert_eq!(
        r#"{"severity":"WARNING","message":"Lorem ipsum","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"foo":"bar","qux":"42"},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":33,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn http_event() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default().with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    trace!(
        http.method = "POST",
        http.url = "https://www.disney.com",
        r#"Http::connect; scheme=Some("http"), host=Some("127.0.0.1"), port=Some(Port(43059))"#
    );

    assert_eq!(
        r#"{"severity":"DEFAULT","message":"Http::connect; scheme=Some(\"http\"), host=Some(\"127.0.0.1\"), port=Some(Port(43059))","time":"1970-01-01T00:00:00Z","logging.googleapis.com/labels":{"http.method":"POST","http.url":"https://www.disney.com"},"logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":47,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
}

#[test]
fn source_location() {
    let (make_writer, writer) = TestMakeWriter::new();
    let sub = Registry::default().with(GCPFormattingLayer::new(make_writer));
    let _guard = tracing::subscriber::set_default(sub);

    tracing::debug!("Where is this coming from?");

    assert_eq!(
        r#"{"severity":"DEBUG","message":"Where is this coming from?","time":"1970-01-01T00:00:00Z","logging.googleapis.com/sourceLocation":{"file":"crates/tracing-gcp-formatter/src/tests.rs","line":65,"function":"tracing_gcp_formatter::tests"}}"#,
        writer.output()
    );
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
