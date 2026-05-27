use std::{
    fmt::{Display, Formatter},
    io::{BufRead, Write, stdin, stdout},
};

use chrono::{DateTime, SecondsFormat};
use clap::Parser;
use colored::Colorize;

use crate::models::{LogEntry, SimplifiedLogEntry, SourceLocation};

mod models;

#[derive(Parser)]
#[command(
    version = env!("CARGO_PKG_VERSION"),
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
    /// Use Simplified format: this is the format used by most applications to produce logs, so
    /// this is usefull to read logs produced locally, i.e. for debugging
    #[arg(name = "simplified-format", long, short)]
    simplified_format: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.simplified_format {
        process_lines_in_simplified_format(
            stdin().lock().lines().map_while(Result::ok),
            &mut stdout().lock(),
            !cli.no_color,
            !cli.no_emoji,
            cli.strict,
        );
    } else {
        process_lines_in_structured_format(
            stdin().lock().lines().map_while(Result::ok),
            &mut stdout().lock(),
            !cli.no_color,
            !cli.no_emoji,
            cli.strict,
        );
    }
}

fn process_lines_in_simplified_format<T: Iterator<Item = impl ToString + Display>, W: Write>(
    lines: T,
    writer: &mut W,
    color: bool,
    emoji: bool,
    strict: bool,
) {
    for line in lines {
        if let Ok(entry) = serde_json::from_str::<SimplifiedLogEntry>(&line.to_string()) {
            let _ = writeln!(writer, "{}", entry.print(color, emoji));
        } else if !strict {
            let _ = writeln!(writer, "{line}");
        }
    }
}

fn process_lines_in_structured_format<T: Iterator<Item = impl ToString + Display>, W: Write>(
    lines: T,
    writer: &mut W,
    color: bool,
    emoji: bool,
    strict: bool,
) {
    let mut current = String::new();

    for line in lines {
        let trimmed = line.to_string().trim().to_owned();

        if trimmed == "[" || trimmed == "]" || trimmed.is_empty() || trimmed == "[]" {
            continue;
        }

        current.push_str(&trimmed);

        match serde_json::from_str(current.trim_end_matches(',')) {
            Ok(obj) => {
                match serde_json::from_value::<LogEntry>(obj) {
                    Ok(entry) => {
                        let _ = writeln!(writer, "{}", entry.print(color, emoji));
                    }
                    Err(_) => {
                        if !strict {
                            let _ = writeln!(writer, "# {current}");
                        }
                    }
                }
                current.clear();
            }
            Err(_) => {
                current.push(' ');
            }
        }
    }
}

trait PrintableLogLine {
    fn print(self, color: bool, emoji: bool) -> String;
}

impl PrintableLogLine for LogEntry {
    fn print(self, color: bool, emoji: bool) -> String {
        let message = {
            let payload = match (self.text_payload, self.json_payload) {
                (Some(text), _) => text,
                (_, Some(json)) => json.to_string(),
                _ => "[empty]".to_owned(),
            };

            if color {
                payload.cyan()
            } else {
                payload.normal()
            }
        };

        format!(
            "[{}] {}: {}{}{}",
            //TODO Cleanup
            // self.timestamp.map(DateTime::parse_from_rfc3339).and_then(|i| i)
            DateTime::parse_from_rfc3339(&self.timestamp.unwrap())
                .unwrap()
                .to_rfc3339_opts(SecondsFormat::Millis, true),
            Severity::new(&self.severity.unwrap_or_default(), color, emoji),
            message,
            Labels(self.labels.map(|i| {
                let mut l = i.into_iter().collect::<Vec<_>>();
                l.sort_by(|a, b| a.0.cmp(&b.0));
                l
            })),
            Sources(self.source_location),
        )
    }
}

impl PrintableLogLine for SimplifiedLogEntry<'_> {
    fn print(self, color: bool, emoji: bool) -> String {
        let message = if color {
            self.message.cyan()
        } else {
            self.message.normal()
        };

        let labels = self.labels.map(|i| {
            let mut l = i
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        },
                    )
                })
                .collect::<Vec<_>>();
            l.sort_by(|a, b| a.0.cmp(&b.0));
            l
        });

        format!(
            "[{}] {}: {}{}{}",
            self.time.to_rfc3339_opts(SecondsFormat::Millis, true),
            Severity::new(&self.severity, color, emoji),
            message,
            Labels(labels),
            Sources(self.source_location),
        )
    }
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

struct Labels(Option<Vec<(String, String)>>);

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

                write!(f, "{}={}", key, value)?;
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
    fn simplified_format_minimal_content_trace_is_default() {
        let input = [r#"{"message":"My message","time":"2026-05-11T08:23:17.404670507Z"}"#];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, true, true, false);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 \u{1b}[37mTRACE\u{1b}[0m: \u{1b}[36mMy message\u{1b}[0m\n",
            String::from_utf8_lossy(&output)
        );
    }

    #[test]
    fn simplified_format_message_with_escaped_double_quotes() {
        let input =
            [r#"{"message":"This is \"quoted\" content","time":"2026-05-11T08:23:17.404670507Z"}"#];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, false, true, false);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 TRACE: This is \"quoted\" content\n",
            String::from_utf8_lossy(&output),
        );
    }

    #[test]
    fn simplified_format_levels_color_emoji_not_strict() {
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
        process_lines_in_simplified_format(input.iter(), &mut output, true, true, false);

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
    fn simplified_format_levels_no_color_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, false, true, false);

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
    fn simplified_format_levels_color_no_emoji_not_strict() {
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
        process_lines_in_simplified_format(input.iter(), &mut output, true, false, false);

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
    fn simplified_format_levels_no_color_no_emoji_not_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, false, false, false);

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
    fn simplified_format_levels_no_color_no_emoji_strict() {
        let input = [
            r#"{"message":"Trace","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"debug","message":"Debug","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"info","message":"Info","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"warning","message":"Warn","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"{"severity":"error","message":"Error","time":"2026-05-11T13:32:04.598656833Z"}"#,
            r#"The quick brown fox"#,
        ];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, false, false, true);

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
    fn simplified_format_labels() {
        let input = [
            r#"{"message":"My message","time":"2026-05-11T08:23:17.404670507Z", "logging.googleapis.com/labels":{"foo": "bar", "baz": "qux"}}"#,
        ];
        let mut output = Vec::new();
        process_lines_in_simplified_format(input.iter(), &mut output, false, true, true);

        assert_eq!(
            "[2026-05-11T08:23:17.404Z] 🐾 TRACE: My message (baz=qux, foo=bar)\n",
            String::from_utf8_lossy(&output),
        );
    }

    #[test]
    fn structured_format_simple() {
        let input = [
            r#"["#,
            r#"  {"#,
            r#"    "insertId": "ekwdppfyq77v9a1i","#,
            r#"    "labels": {"#,
            r#"      "compute.googleapis.com/resource_name": "gke-cluster-node-pool20251104-52cdb89b-83zw","#,
            r#"      "k8s-pod/app": "my-pod","#,
            r#"      "k8s-pod/pod-template-hash": "65b465d4bb","#,
            r#"      "logging.gke.io/top_level_controller_name": "my-pod","#,
            r#"      "logging.gke.io/top_level_controller_type": "Deployment""#,
            r#"    },"#,
            r#"    "logName": "projects/my-project/logs/stdout","#,
            r#"    "receiveTimestamp": "2026-05-26T21:11:49.319620805Z","#,
            r#"    "resource": {"#,
            r#"      "labels": {"#,
            r#"        "cluster_name": "gke-cluster","#,
            r#"        "container_name": "my-container","#,
            r#"        "location": "europe-west1","#,
            r#"        "namespace_name": "my-namespace","#,
            r#"        "pod_name": "my-pod-123","#,
            r#"        "project_id": "my-project""#,
            r#"      },"#,
            r#"      "type": "k8s_container""#,
            r#"    },"#,
            r#"    "severity": "INFO","#,
            r#"    "sourceLocation": {"#,
            r#"      "file": "src/lib.rs","#,
            r#"      "function": "lib::main","#,
            r#"      "line": "24""#,
            r#"    },"#,
            r#"    "textPayload": "Some text payload","#,
            r#"    "timestamp": "2026-05-26T21:11:44.677623573Z""#,
            r#"  }"#,
            r#"]"#,
        ];

        let mut output = Vec::new();
        process_lines_in_structured_format(input.iter(), &mut output, false, false, true);

        assert_eq!(
            "[2026-05-26T21:11:44.677Z]  INFO: Some text payload (compute.googleapis.com/resource_name=gke-cluster-node-pool20251104-52cdb89b-83zw, k8s-pod/app=my-pod, k8s-pod/pod-template-hash=65b465d4bb, logging.gke.io/top_level_controller_name=my-pod, logging.gke.io/top_level_controller_type=Deployment) (file=src/lib.rs, line=24, function=lib::main)\n",
            String::from_utf8_lossy(&output),
        );
    }

    #[test]
    fn structured_format_two_entries() {
        let input = [
            r#"["#,
            r#"  {"#,
            r#"    "insertId": "ekwdppfyq77v9a1i","#,
            r#"    "labels": {"#,
            r#"      "compute.googleapis.com/resource_name": "gke-cluster-node-pool20251104-52cdb89b-83zw","#,
            r#"      "k8s-pod/app": "my-pod","#,
            r#"      "k8s-pod/pod-template-hash": "65b465d4bb","#,
            r#"      "logging.gke.io/top_level_controller_name": "my-pod","#,
            r#"      "logging.gke.io/top_level_controller_type": "Deployment""#,
            r#"    },"#,
            r#"    "logName": "projects/my-project/logs/stdout","#,
            r#"    "receiveTimestamp": "2026-05-26T21:11:49.319620805Z","#,
            r#"    "resource": {"#,
            r#"      "labels": {"#,
            r#"        "cluster_name": "gke-cluster","#,
            r#"        "container_name": "my-container","#,
            r#"        "location": "europe-west1","#,
            r#"        "namespace_name": "my-namespace","#,
            r#"        "pod_name": "my-pod-123","#,
            r#"        "project_id": "my-project""#,
            r#"      },"#,
            r#"      "type": "k8s_container""#,
            r#"    },"#,
            r#"    "severity": "INFO","#,
            r#"    "sourceLocation": {"#,
            r#"      "file": "src/lib.rs","#,
            r#"      "function": "lib::main","#,
            r#"      "line": "24""#,
            r#"    },"#,
            r#"    "textPayload": "Some text payload","#,
            r#"    "timestamp": "2026-05-26T21:11:44.677623573Z""#,
            r#"  },"#,
            r#"  {"#,
            r#"    "insertId": "8v9p7rien9spfltm","#,
            r#"    "labels": {"#,
            r#"      "compute.googleapis.com/resource_name": "gke-cluster-node-pool20251104-52cdb89b-83zw","#,
            r#"      "hostname": "my-pod-987","#,
            r#"      "k8s-pod/app": "my-pod","#,
            r#"      "k8s-pod/pod-template-hash": "65b465d4bb","#,
            r#"      "logging.gke.io/top_level_controller_name": "my-pod","#,
            r#"      "logging.gke.io/top_level_controller_type": "Deployment""#,
            r#"    },"#,
            r#"    "logName": "projects/my-project/logs/stdout","#,
            r#"    "receiveTimestamp": "2026-05-26T21:11:49.319620805Z","#,
            r#"    "resource": {"#,
            r#"      "labels": {"#,
            r#"        "cluster_name": "gke-cluster","#,
            r#"        "container_name": "my-container","#,
            r#"        "location": "europe-west1","#,
            r#"        "namespace_name": "my-namespace","#,
            r#"        "pod_name": "my-pod-123","#,
            r#"        "project_id": "my-project""#,
            r#"      },"#,
            r#"      "type": "k8s_container""#,
            r#"    },"#,
            r#"    "severity": "DEBUG","#,
            r#"    "sourceLocation": {"#,
            r#"      "file": "src/lib.rs","#,
            r#"      "function": "lib::main","#,
            r#"      "line": "176""#,
            r#"    },"#,
            r#"    "textPayload": "Some other text payload","#,
            r#"    "timestamp": "2026-05-26T21:11:44.677591073Z""#,
            r#"  }"#,
            r#"]"#,
        ];

        let mut output = Vec::new();
        process_lines_in_structured_format(input.iter(), &mut output, false, false, true);

        assert_eq!(
            "[2026-05-26T21:11:44.677Z]  INFO: Some text payload (compute.googleapis.com/resource_name=gke-cluster-node-pool20251104-52cdb89b-83zw, k8s-pod/app=my-pod, k8s-pod/pod-template-hash=65b465d4bb, logging.gke.io/top_level_controller_name=my-pod, logging.gke.io/top_level_controller_type=Deployment) (file=src/lib.rs, line=24, function=lib::main)
[2026-05-26T21:11:44.677Z] DEBUG: Some other text payload (compute.googleapis.com/resource_name=gke-cluster-node-pool20251104-52cdb89b-83zw, hostname=my-pod-987, k8s-pod/app=my-pod, k8s-pod/pod-template-hash=65b465d4bb, logging.gke.io/top_level_controller_name=my-pod, logging.gke.io/top_level_controller_type=Deployment) (file=src/lib.rs, line=176, function=lib::main)\n",
            String::from_utf8_lossy(&output),
        );
    }

    #[test]
    fn structured_format_empty() {
        let input = ["[]"];

        let mut output = Vec::new();
        process_lines_in_structured_format(input.iter(), &mut output, false, false, true);

        assert_eq!("", String::from_utf8_lossy(&output));
    }
}
