# gcp-log

A CLI tool for structured logging on GCP.

Supports coloring and emoji-ing logs that are using the Google Strcutred Logging Simplified format (see https://docs.cloud.google.com/logging/docs/structured-logging)

## Installation

```bash
cargo install gcp-log
```

## Usage

Just pipe the output into `gcp-log`.

If you want to do a quick test, run this:

```
printf '{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}' | gcp-log
```
