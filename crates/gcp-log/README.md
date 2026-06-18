
# gcp-log

A CLI tool that reads GCP log JSON from stdin and renders it as a human-friendly, colorized terminal output. It understands both the full GCP Structured Logging format (as returned by `gcloud logging read`) and the simplified format produced by `tracing-gcp-formatter`.

## Installation

```bash
cargo install gcp-log
```

## Usage

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

## Examples

### Parse logs from Cloud Logging (`gcloud`)

Pipe the JSON output of `gcloud logging read` directly into `gcp-log`:

```bash
gcloud logging read 'resource.labels.container_name="<MY_CONTAINER>"' \
  --limit 5 \
  --project <MY_PROJECT> \
  --format json \
  | gcp-log
```

### Parse logs from a local `cargo test` run

Use `-s` (simplified format) when reading logs produced by `tracing-gcp-formatter` locally:

```bash
cargo test my_test | gcp-log -s
```

### Quick smoke test

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

## CI / non-interactive use

Disable color and emoji for plain-text output suitable for CI logs:

```bash
gcloud logging read '...' --format json | gcp-log --no-color --no-emoji
```
