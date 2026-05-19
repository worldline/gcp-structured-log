use std::{
    fmt::{Display, Formatter},
    io::{BufRead, Write, stdout},
};

use anyhow::Context;
use chrono::SecondsFormat;
use clap::Parser;
use colored::Colorize;

use crate::models::{SimplifiedLogEntry, SourceLocation};

mod models;

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
    /// Use simplified format; useful for development
    #[arg(name = "simplified", short = 's', long = "simplified")]
    simplified: bool,
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
    let message = if color {
        entry.message.cyan()
    } else {
        entry.message.normal()
    };

    format!(
        "[{}] {}: {}{}{}",
        entry.time.to_rfc3339_opts(SecondsFormat::Millis, true),
        Severity::new(&entry.severity, color, emoji),
        message,
        Labels(entry.labels),
        Sources(entry.source_location),
    )
}

struct Severity<'a> {
    severity: &'a models::Severity,
    emoji: bool,
    color: bool,
}

impl<'a> Severity<'a> {
    fn new(severity: &'a models::Severity, color: bool, emoji: bool) -> Self {
        Self {
            severity,
            color,
            emoji,
        }
    }
}

impl<'a> Display for Severity<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn severity_emoji(severity: &models::Severity, emoji: bool) -> &'static str {
            match (severity, emoji) {
                (models::Severity::Default, true) => "🐾 ",
                (models::Severity::Debug, true) => "🪲 ",
                (models::Severity::Info, true) => "ℹ️  ",
                (models::Severity::Warning, true) => "⚠️  ",
                (models::Severity::Error, true) => "⛔ ",
                (_, true) => "🐼 ",
                _ => "",
            }
        }

        fn color_severity(severity: &models::Severity, color: bool) -> String {
            let base = format!("{severity}");
            let base = match (severity, color) {
                (models::Severity::Default, true) => base.white(),
                (models::Severity::Debug, true) => base.yellow(),
                (models::Severity::Info, true) => base.cyan(),
                (models::Severity::Warning, true) => base.magenta(),
                (models::Severity::Error, true) => base.red(),
                _ => base.normal(),
            };
            base.to_string()
        }

        write!(
            f,
            "{}{}",
            severity_emoji(self.severity, self.emoji),
            color_severity(self.severity, self.color)
        )
    }
}

struct Labels(Option<serde_json::Map<String, serde_json::Value>>);

impl Display for Labels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(labels) = &self.0
            && !labels.is_empty()
        {
            write!(f, " (")?;
            let mut first = true;

            for (key, value) in labels {
                if !first {
                    write!(f, ", ")?;
                }

                let val = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };

                write!(f, "{}={}", key, val)?;
                first = false;
            }

            write!(f, ")")?;
        }

        Ok(())
    }
}

struct Sources(Option<SourceLocation>);

impl Display for Sources {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(source_location) => {
                let mut parts = Vec::new();

                if let Some(file) = &source_location.file {
                    parts.push(format!("file={}", file));
                }
                if let Some(line) = &source_location.line {
                    parts.push(format!("line={}", line));
                }
                if let Some(function) = &source_location.function {
                    parts.push(format!("function={}", function));
                }

                if parts.is_empty() {
                    write!(f, "")
                } else {
                    write!(f, " ({})", parts.join(", "))
                }
            }
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn minimal_content_trace_is_default() {
        let input = r#"{"message":"My message","time":"2026-05-11T08:23:17.404670507Z"}"#;

        let result = parse(input).unwrap();
        let output = print_line(result, false, true);

        assert_eq!("[2026-05-11T08:23:17.404Z] 🐾 TRACE: My message", output,);
    }

    #[test]
    fn message_with_escaped_double_quotes() {
        let input =
            r#"{"message":"This is \"quoted\" content","time":"2026-05-11T08:23:17.404670507Z"}"#;

        let result = parse(input).unwrap();
        let output = print_line(result, false, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 TRACE: This is \"quoted\" content",
            output,
        );
    }

    #[test]
    fn levels_color_emoji_not_strict() {
        if std::env::var("CI").is_ok() {
            return;
        }

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
            "[2026-05-11T13:32:04.598Z] \u{1f43e} \u{1b}[37mTRACE\u{1b}[0m: \u{1b}[36mTrace\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{1fab2} \u{1b}[33mDEBUG\u{1b}[0m: \u{1b}[36mDebug\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{2139}\u{fe0f}  \u{1b}[36m INFO\u{1b}[0m: \u{1b}[36mInfo\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{26a0}\u{fe0f}  \u{1b}[35m WARN\u{1b}[0m: \u{1b}[36mWarn\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{26d4} \u{1b}[31mERROR\u{1b}[0m: \u{1b}[36mError\u{1b}[0m
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
        if std::env::var("CI").is_ok() {
            return;
        }

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
            "[2026-05-11T13:32:04.598Z] \u{1b}[37mTRACE\u{1b}[0m: \u{1b}[36mTrace\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{1b}[33mDEBUG\u{1b}[0m: \u{1b}[36mDebug\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{1b}[36m INFO\u{1b}[0m: \u{1b}[36mInfo\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{1b}[35m WARN\u{1b}[0m: \u{1b}[36mWarn\u{1b}[0m
[2026-05-11T13:32:04.598Z] \u{1b}[31mERROR\u{1b}[0m: \u{1b}[36mError\u{1b}[0m
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
        let output = print_line(result, false, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 TRACE: My message (baz=qux, foo=bar)",
            output,
        );
    }
}
