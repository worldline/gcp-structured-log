
# tracing-gcp-formatter

A `tracing-subscriber` layer that formats log events and spans as GCP Structured Logging JSON. Each line of output is a self-contained JSON object that Cloud Logging can parse directly.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry"] }
tracing-gcp-formatter = "0.8"
```

## Setup

Register both `SpanDataLayer` and `GCPFormattingLayer` with the tracing subscriber. `SpanDataLayer` must come first — it captures span fields so they are available to `GCPFormattingLayer` when events fire inside a span.

```rust
use tracing_subscriber::{Registry, layer::SubscriberExt};
use tracing_gcp_formatter::{GCPFormattingLayer, SpanDataLayer};

fn main() {
    let subscriber = Registry::default()
        .with(SpanDataLayer::new())
        .with(GCPFormattingLayer::new(std::io::stdout));

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");

    tracing::info!("Application started");
}
```

## Usage

Use the standard `tracing` macros. Extra fields are included in the `logging.googleapis.com/labels` object.

```rust
use std::error::Error;
use tracing::{info, warn, error, instrument};

// Simple log
info!("Processing request");

// Log with structured fields
warn!(user_id = 42, path = "/api/data", "Slow response detected");

// Instrument a function — emits START and END log entries automatically
#[instrument(fields(order_id = %order.id))]
async fn process_order(order: &Order) -> Result<(), Box<dyn Error>> {
    info!("Validating order");
    // ...
    Ok(())
}

#[derive(Debug)]
struct Order {
    id: u32,
}
```

## Output Format

Each log entry is a JSON object on a single line:

```json
{"severity":"INFO","message":"Processing request","time":"2024-06-17T10:23:45.123Z","logging.googleapis.com/labels":{"hostname":"my-pod","pid":1234},"logging.googleapis.com/sourceLocation":{"file":"src/main.rs","line":"12","function":"my_crate::main"}}
```

With structured fields:

```json
{"severity":"WARNING","message":"Slow response detected","time":"2024-06-17T10:23:45.456Z","logging.googleapis.com/labels":{"hostname":"my-pod","path":"/api/data","pid":1234,"user_id":42},"logging.googleapis.com/sourceLocation":{"file":"src/handler.rs","line":"38","function":"my_crate::handler"}}
```

### Severity mapping

| `tracing` level | GCP severity |
|-----------------|--------------|
| `TRACE`         | `DEFAULT`    |
| `DEBUG`         | `DEBUG`      |
| `INFO`          | `INFO`       |
| `WARN`          | `WARNING`    |
| `ERROR`         | `ERROR`      |

### Span lifecycle

When a span opens, a `[SPANNAME - START]` entry is emitted. Events fired inside the span are prefixed with `[SPANNAME - EVENT]` and carry the span's fields in their labels. When the span closes, a `[SPANNAME - END]` entry is emitted.

```json
{"severity":"INFO","message":"[PROCESS_ORDER - START]","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
{"severity":"INFO","message":"[PROCESS_ORDER - EVENT] Validating order","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
{"severity":"INFO","message":"[PROCESS_ORDER - END]","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
```

### Automatic labels

Every log entry includes:

| Label       | Value                                      |
|-------------|--------------------------------------------|
| `pid`       | Operating system process ID                |
| `hostname`  | Machine hostname (requires `hostname` feature, enabled by default) |

## Features

| Feature    | Default | Description                                      |
|------------|---------|--------------------------------------------------|
| `hostname` | yes     | Includes the machine hostname in every log entry |

To disable hostname capture:

```toml
tracing-gcp-formatter = { version = "0.8", default-features = false }
```
