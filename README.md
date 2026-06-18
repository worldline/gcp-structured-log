# GCP Structured Logging

A Rust workspace providing two complementary crates for working with [Google Cloud Structured Logging](https://cloud.google.com/logging/docs/structured-logging):

- **[`tracing-gcp-formatter`](#tracing-gcp-formatter)** — a [`tracing`](https://docs.rs/tracing) layer that emits logs in the GCP Simplified Structured Logging JSON format, ready to be ingested by Cloud Logging.
- **[`gcp-log`](#gcp-log)** — a CLI tool that reads GCP log entries from stdin and renders them as a readable, colorized terminal output.

---

## tracing-gcp-formatter

A `tracing-subscriber` layer that formats log events and spans as GCP Structured Logging JSON. Each line of output is a self-contained JSON object that Cloud Logging can parse directly.

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry"] }
tracing-gcp-formatter = "0.8"
```

### Setup

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

### Usage

Use the standard `tracing` macros. Extra fields are included in the `logging.googleapis.com/labels` object.

```rust
use tracing::{info, warn, error, instrument};

// Simple log
info!("Processing request");

// Log with structured fields
warn!(user_id = 42, path = "/api/data", "Slow response detected");

// Instrument a function — emits START and END log entries automatically
#[instrument(fields(order_id = %order.id))]
async fn process_order(order: &Order) -> Result<(), Error> {
    info!("Validating order");
    // ...
    Ok(())
}
```

### Output Format

Each log entry is a JSON object on a single line:

```json
{"severity":"INFO","message":"Processing request","time":"2024-06-17T10:23:45.123Z","logging.googleapis.com/labels":{"hostname":"my-pod","pid":1234},"logging.googleapis.com/sourceLocation":{"file":"src/main.rs","line":"12","function":"my_crate::main"}}
```

With structured fields:

```json
{"severity":"WARNING","message":"Slow response detected","time":"2024-06-17T10:23:45.456Z","logging.googleapis.com/labels":{"hostname":"my-pod","path":"/api/data","pid":1234,"user_id":42},"logging.googleapis.com/sourceLocation":{"file":"src/handler.rs","line":"38","function":"my_crate::handler"}}
```

#### Severity mapping

| `tracing` level | GCP severity |
|-----------------|--------------|
| `TRACE`         | `DEFAULT`    |
| `DEBUG`         | `DEBUG`      |
| `INFO`          | `INFO`       |
| `WARN`          | `WARNING`    |
| `ERROR`         | `ERROR`      |

#### Span lifecycle

When a span opens, a `[SPANNAME - START]` entry is emitted. Events fired inside the span are prefixed with `[SPANNAME - EVENT]` and carry the span's fields in their labels. When the span closes, a `[SPANNAME - END]` entry is emitted.

```
{"severity":"INFO","message":"[PROCESS_ORDER - START]","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
{"severity":"INFO","message":"[PROCESS_ORDER - EVENT] Validating order","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
{"severity":"INFO","message":"[PROCESS_ORDER - END]","logging.googleapis.com/labels":{"order_id":"ORD-99",...}}
```

#### Automatic labels

Every log entry includes:

| Label       | Value                                      |
|-------------|--------------------------------------------|
| `pid`       | Operating system process ID                |
| `hostname`  | Machine hostname (requires `hostname` feature, enabled by default) |

### Features

| Feature    | Default | Description                                      |
|------------|---------|--------------------------------------------------|
| `hostname` | yes     | Includes the machine hostname in every log entry |

To disable hostname capture:

```toml
tracing-gcp-formatter = { version = "0.8", default-features = false }
```

---

## gcp-log

A CLI tool that reads GCP log JSON from stdin and renders it as a human-friendly, colorized terminal output. It understands both the full GCP Structured Logging format (as returned by `gcloud logging read`) and the simplified format produced by `tracing-gcp-formatter`.

### Installation

```bash
cargo install gcp-log
```

### Usage

```
gcp-log [OPTIONS]

Options:
      --no-color   Disable colored output
      --no-emoji   Disable emoji severity indicators
      --strict     Only print valid log entries; skip lines that cannot be parsed
  -s, --simplified-format
                   Parse GCP Simplified format (output of tracing-gcp-formatter)
                   instead of the default full structured format
```

### Examples

#### Parse logs from Cloud Logging (`gcloud`)

Pipe the JSON output of `gcloud logging read` directly into `gcp-log`:

```bash
gcloud logging read 'resource.labels.container_name="<MY_CONTAINER>"' \
  --limit 5 \
  --project <MY_PROJECT> \
  --format json \
  | gcp-log
```

#### Parse logs from a local `cargo test` run

Use `-s` (simplified format) when reading logs produced by `tracing-gcp-formatter` locally:

```bash
cargo test my_test | gcp-log -s
```

#### Quick smoke test

```bash
printf '{"message":"Trace","time":"2026-05-11T13:32:04Z"}\n{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04Z"}\n{"severity":"info","message":"Info","time":"2026-05-11T13:32:04Z"}\n{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04Z"}\n{"severity":"error","message":"Error","time":"2026-05-11T13:32:04Z"}' | gcp-log -s
```

### Output

Each log entry is rendered on a single line with:

- An emoji indicating severity
- A colored severity label
- The timestamp
- The log message
- Labels as `key=value` pairs (sorted)
- Source location as `(file:line function)`

#### Severity indicators

| Severity          | Emoji | Color   |
|-------------------|-------|---------|
| `DEFAULT`         | 🐾    | white   |
| `DEBUG`           | 🪲    | yellow  |
| `INFO`            | ℹ️    | cyan    |
| `NOTICE`          | 🐼    | white   |
| `WARNING`         | ⚠️    | magenta |
| `ERROR`           | ⛔    | red     |
| `CRITICAL`/`ALERT`/`EMERGENCY` | 💥 | red |

### CI / non-interactive use

Disable color and emoji for plain-text output suitable for CI logs:

```bash
gcloud logging read '...' --format json | gcp-log --no-color --no-emoji
```

---

## License

MIT

## Shoutouts

- tracing-bunyan-formatter: https://github.com/LukeMathWalker/tracing-bunyan-formatter
- bunyan: https://github.com/LukeMathWalker/bunyan
- tracing-gcp: https://docs.rs/tracing-gcp/latest/tracing_gcp
