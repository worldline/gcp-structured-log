# GCP Structured Logging

A Rust workspace providing two complementary crates for working with [Google Cloud Structured Logging](https://cloud.google.com/logging/docs/structured-logging):

- **[`tracing-gcp-formatter`](#tracing-gcp-formatter)** — a [`tracing`](https://docs.rs/tracing) layer that emits logs in the GCP Simplified Structured Logging JSON format, ready to be ingested by Cloud Logging.
- **[`gcp-log`](#gcp-log)** — a CLI tool that reads GCP log entries from stdin and renders them as a readable, colorized terminal output.

---

## tracing-gcp-formatter

[See crate level README.](./crates/tracing-gcp-formatter/README.md)

---

## gcp-log

[See crate level README.](./crates/gcp-log/README.md)

---

## License

MIT

## Shoutouts

- tracing-bunyan-formatter: https://github.com/LukeMathWalker/tracing-bunyan-formatter
- bunyan: https://github.com/LukeMathWalker/bunyan
- tracing-gcp: https://docs.rs/tracing-gcp/latest/tracing_gcp

