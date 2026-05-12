# GCP Structured Logging

## tracing-gcp-formatter

## gcp-log

CLI to format GCP logs

### Quick & Dirty Test

```
printf '{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}\n{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}' | cargo run -p gcp-log
```
