use std::{
    fmt::Display,
    io::{BufRead, Write, stdout},
};

use anyhow::Context;
use chrono::SecondsFormat;
use clap::Parser;
use colored::Colorize;

use log_entry::{Severity, SimplifiedLogEntry};

#[derive(Parser)]
#[command(
    version = "0.1",
    author = "Philippe Vlérick <philippe.vlerick@worldline.com>"
)]
struct Cli {
    /// No coloring in output.
    #[arg(name = "no-color", long = "no-color")]
    no_color: bool,
    /// No emoji in output :sad_panda:.
    #[arg(name = "no-emoji", long = "no-emoji")]
    no_emoji: bool,
    /// Do not print lines that are not valid
    #[arg(long)]
    strict: bool,
}

fn main() {
    let cli = Cli::parse();

    let stdin = std::io::stdin();

    process_lines(
        stdin.lock().lines().map_while(Result::ok),
        &mut stdout().lock(),
        !cli.no_color,
        !cli.no_emoji,
        cli.strict,
    );
}

fn process_lines<T: Iterator<Item = impl ToString + Display>, W: Write>(
    lines: T,
    writer: &mut W,
    color: bool,
    emoji: bool,
    strict: bool,
) {
    for line in lines {
        let _ = match parse(&line.to_string()) {
            Ok(line) => writeln!(writer, "{}", print_line(line, color, emoji)),
            Err(_) => {
                if !strict {
                    writeln!(writer, "{line}")
                } else {
                    Ok(())
                }
            }
        };
    }
}

fn parse(line: &str) -> anyhow::Result<SimplifiedLogEntry<'_>> {
    serde_json::from_str::<SimplifiedLogEntry>(line).context("Parsing log line")
}

fn print_line(entry: SimplifiedLogEntry, color: bool, emoji: bool) -> String {
    format!(
        "[{}] {}: {}{}",
        entry.time.to_rfc3339_opts(SecondsFormat::Millis, true),
        format_severity(&entry.severity, color, emoji),
        entry.message,
        format_labels(entry.labels),
    )
}

fn format_severity(severity: &Severity, color: bool, emoji: bool) -> String {
    fn severity_emoji(severity: &Severity, emoji: bool) -> &'static str {
        match (severity, emoji) {
            (Severity::Default, true) => "🐾 ",
            (Severity::Debug, true) => "🪲 ",
            (Severity::Info, true) => "ℹ️  ",
            (Severity::Warning, true) => "⚠️  ",
            (Severity::Error, true) => "⛔ ",
            (_, true) => "🐼 ",
            _ => "",
        }
    }

    fn color_severity(severity: &Severity, color: bool) -> String {
        let base = format!("{severity}");
        let base = match (severity, color) {
            (Severity::Default, true) => base.white(),
            (Severity::Debug, true) => base.yellow(),
            (Severity::Info, true) => base.cyan(),
            (Severity::Warning, true) => base.magenta(),
            (Severity::Error, true) => base.red(),
            _ => base.normal(),
        };
        base.to_string()
    }

    format!(
        "{}{}",
        severity_emoji(severity, emoji),
        color_severity(severity, color)
    )
}

fn format_labels(labels: Option<serde_json::Map<String, serde_json::Value>>) -> String {
    match labels {
        Some(labels) if !labels.is_empty() => format!(
            " ({})",
            labels
                .iter()
                .map(|i| {
                    format!(
                        "{}={}",
                        i.0,
                        i.1.as_str().expect("json value is not a string").to_owned(),
                    )
                })
                .collect::<Vec<String>>()
                .join(", ")
        ),
        _ => "".to_owned(),
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn minimal_content_trace_is_default() {
        let input = r#"{"message":"My message","time":"2026-05-11T08:23:17.404670507Z"}"#;

        let result = parse(input).unwrap();
        let output = print_line(result, true, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 \u{1b}[37mTRACE\u{1b}[0m: My message",
            output,
        );
    }

    #[test]
    fn message_with_escaped_double_quotes() {
        let input =
            r#"{"message":"This is \"quoted\" content","time":"2026-05-11T08:23:17.404670507Z"}"#;

        let result = parse(input).unwrap();
        let output = print_line(result, true, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 \u{1b}[37mTRACE\u{1b}[0m: This is \"quoted\" content",
            output,
        );
    }

    #[test]
    fn levels_color_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines(input.iter(), &mut output, true, true, false);

        assert_eq!(
            "[2026-05-11T13:32:04.598Z] \u{1f43e} \u{1b}[37mTRACE\u{1b}[0m: Trace
[2026-05-11T13:32:04.598Z] \u{1fab2} \u{1b}[33mDEBUG\u{1b}[0m: Debug
[2026-05-11T13:32:04.598Z] \u{2139}\u{fe0f}  \u{1b}[36m INFO\u{1b}[0m: Info
[2026-05-11T13:32:04.598Z] \u{26a0}\u{fe0f}  \u{1b}[35m WARN\u{1b}[0m: Warn
[2026-05-11T13:32:04.598Z] \u{26d4} \u{1b}[31mERROR\u{1b}[0m: Error
The quick brown fox
",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn levels_no_color_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines(input.iter(), &mut output, false, true, false);

        assert_eq!(
            "[2026-05-11T13:32:04.598Z] \u{1f43e} TRACE: Trace
[2026-05-11T13:32:04.598Z] \u{1fab2} DEBUG: Debug
[2026-05-11T13:32:04.598Z] \u{2139}\u{fe0f}   INFO: Info
[2026-05-11T13:32:04.598Z] \u{26a0}\u{fe0f}   WARN: Warn
[2026-05-11T13:32:04.598Z] \u{26d4} ERROR: Error
The quick brown fox
",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn levels_color_no_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines(input.iter(), &mut output, true, false, false);

        assert_eq!(
            "[2026-05-11T13:32:04.598Z] \u{1b}[37mTRACE\u{1b}[0m: Trace
[2026-05-11T13:32:04.598Z] \u{1b}[33mDEBUG\u{1b}[0m: Debug
[2026-05-11T13:32:04.598Z] \u{1b}[36m INFO\u{1b}[0m: Info
[2026-05-11T13:32:04.598Z] \u{1b}[35m WARN\u{1b}[0m: Warn
[2026-05-11T13:32:04.598Z] \u{1b}[31mERROR\u{1b}[0m: Error
The quick brown fox
",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn levels_no_color_no_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines(input.iter(), &mut output, false, false, false);

        assert_eq!(
            "[2026-05-11T13:32:04.598Z] TRACE: Trace
[2026-05-11T13:32:04.598Z] DEBUG: Debug
[2026-05-11T13:32:04.598Z]  INFO: Info
[2026-05-11T13:32:04.598Z]  WARN: Warn
[2026-05-11T13:32:04.598Z] ERROR: Error
The quick brown fox
",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn levels_no_color_no_emoji_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines(input.iter(), &mut output, false, false, true);

        assert_eq!(
            "[2026-05-11T13:32:04.598Z] TRACE: Trace
[2026-05-11T13:32:04.598Z] DEBUG: Debug
[2026-05-11T13:32:04.598Z]  INFO: Info
[2026-05-11T13:32:04.598Z]  WARN: Warn
[2026-05-11T13:32:04.598Z] ERROR: Error
",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn labels() {
        let input = r#"{"message":"My message","time":"2026-05-11T08:23:17.404670507Z", "logging.googleapis.com/labels":{"foo": "bar", "baz": "qux"}}"#;

        let result = parse(input).unwrap();
        let output = print_line(result, true, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 \u{1b}[37mTRACE\u{1b}[0m: My message (baz=qux, foo=bar)",
            output,
        );
    }
}
