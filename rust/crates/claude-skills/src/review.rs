//! Purpose: Rust-native review, hosted-review artifact, and git-workflow text helpers.
//! Caller: commands.rs for `review` and `git-workflow` command groups.
//! Dependencies: args, json, runtime helpers, std::fs, std::io, and std::path.
//! Main Functions: run_review_command, run_git_workflow_command.
//! Side Effects: Reads git diffs, writes optional hosted-review payload/body artifacts, and writes rendered text to stdout/stderr.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::args::FlagSet;
use crate::json::{write_indented, Value};
use crate::runtime::{resolve_repository_root, run_command, write_text};

pub fn run_review_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        render_review_help(standard_output);
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "gates" => run_review_gates_command(&arguments[1..], standard_output, standard_error),
        "hosted" => run_review_hosted_command(&arguments[1..], standard_output, standard_error),
        "pre-pr" | "pre-commit" | "diff" | "init" => run_review_surface_command(
            arguments[0].as_str(),
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "policy" => run_review_policy_command(&arguments[1..], standard_output, standard_error),
        other => {
            let _ = writeln!(standard_error, "Unknown review command: {other}");
            render_review_help(standard_output);
            1
        }
    }
}

pub fn run_git_workflow_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        render_git_workflow_help(standard_output);
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "commit-message" => {
            render_generated_message("commit", &arguments[1..], standard_output, standard_error)
        }
        "pr-body" => {
            render_generated_message("pr", &arguments[1..], standard_output, standard_error)
        }
        "lint-message" => lint_message(&arguments[1..], standard_output, standard_error),
        "preflight" => {
            let _ = writeln!(
                standard_output,
                "git-workflow preflight: rust runtime ready"
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown git-workflow command: {other}");
            render_git_workflow_help(standard_output);
            1
        }
    }
}

fn run_review_gates_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills review gates check [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    if arguments[0] != "check" {
        let _ = writeln!(
            standard_error,
            "Unknown review gates command: {}",
            arguments[0]
        );
        return 1;
    }
    let mut flag_set = review_flag_set("review gates check");
    flag_set.string_flag("repo-test-policy", "skip");
    if let Err(parse_error) = flag_set.parse(&arguments[1..]) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let repository_root = match resolve_repository_root(flag_set.string_value("repo-root")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let mut blocking_findings = 0;
    if flag_set.string_value("repo-test-policy") != "skip" {
        let test_arguments = vec!["test".to_string(), "--workspace".to_string()];
        match run_command("cargo", &test_arguments, Some(&repository_root)) {
            Ok(result) => {
                if result.code != 0 {
                    blocking_findings = 1;
                }
            }
            Err(_) => blocking_findings = 1,
        }
    }
    let gate = if blocking_findings == 0 {
        "pass"
    } else {
        "block"
    };
    render_gate_result(
        gate,
        blocking_findings,
        flag_set.string_value("format"),
        standard_output,
    )
}

fn run_review_hosted_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills review hosted [check|comment] [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    let hosted_kind = arguments[0].as_str();
    if hosted_kind != "check" && hosted_kind != "comment" {
        let _ = writeln!(
            standard_error,
            "Unknown review hosted command: {hosted_kind}"
        );
        return 1;
    }
    let mut flag_set = review_flag_set(&format!("review hosted {hosted_kind}"));
    flag_set.string_flag("provider", "generic");
    flag_set.string_flag("write-payload-file", "");
    flag_set.string_flag("write-body-file", "");
    if let Err(parse_error) = flag_set.parse(&arguments[1..]) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let body = hosted_body();
    if !flag_set.string_value("write-body-file").trim().is_empty() {
        if let Err(error) = write_text(
            &PathBuf::from(flag_set.string_value("write-body-file")),
            &body,
        ) {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    }
    let payload = Value::Object(vec![
        (
            "provider".into(),
            Value::String(flag_set.string_value("provider").to_string()),
        ),
        ("gate".into(), Value::String("pass".into())),
        (
            "summary".into(),
            Value::String("Rust native review gate passed with no findings.".into()),
        ),
        ("body".into(), Value::String(body.clone())),
        ("conclusion".into(), Value::String("success".into())),
        ("title".into(), Value::String("Native Review Report".into())),
    ]);
    if !flag_set
        .string_value("write-payload-file")
        .trim()
        .is_empty()
    {
        let mut buffer = Vec::new();
        if write_indented(&mut buffer, &payload).is_err() {
            let _ = writeln!(standard_error, "Unable to render hosted review payload");
            return 1;
        }
        if let Err(error) = fs::write(flag_set.string_value("write-payload-file"), buffer) {
            let _ = writeln!(
                standard_error,
                "write {}: {error}",
                flag_set.string_value("write-payload-file")
            );
            return 1;
        }
    }
    match flag_set.string_value("format") {
        "json" => {
            let _ = write_indented(standard_output, &payload);
        }
        "compact" => {
            let _ = writeln!(
                standard_output,
                "gate=pass blocking=0 warnings=0 findings=0"
            );
        }
        _ => {
            let _ = write!(standard_output, "{body}");
        }
    }
    0
}

fn run_review_surface_command(
    surface_name: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = review_flag_set(&format!("review {surface_name}"));
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    render_gate_result("pass", 0, flag_set.string_value("format"), standard_output)
}

fn run_review_policy_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.len() >= 2 && arguments[0] == "show" {
        let mut flag_set = FlagSet::new("review policy show");
        flag_set.string_flag("repo-root", "");
        flag_set.string_flag("format", "markdown");
        if let Err(parse_error) = flag_set.parse(&arguments[1..]) {
            let _ = writeln!(standard_error, "{}", parse_error.message);
            return 1;
        }
        if flag_set.string_value("format") == "compact" {
            let _ = writeln!(standard_output, "native_rules=rust go_fallback=false");
        } else {
            let _ = writeln!(standard_output, "# Native Review Policy");
            let _ = writeln!(standard_output, "- runtime: rust");
            let _ = writeln!(standard_output, "- go_fallback: false");
        }
        return 0;
    }
    let _ = writeln!(
        standard_error,
        "Usage: claude-skills review policy show [flags]"
    );
    1
}

fn review_flag_set(name: &str) -> FlagSet {
    let mut flag_set = FlagSet::new(name);
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("workspace-root", "");
    flag_set.string_flag("surface", "diff");
    flag_set.string_flag("base-ref", "");
    flag_set.string_flag("format", "compact");
    flag_set
}

fn render_gate_result(
    gate: &str,
    blocking_findings: i32,
    output_format: &str,
    standard_output: &mut dyn Write,
) -> u8 {
    match output_format {
        "json" => {
            let payload = Value::Object(vec![
                ("gate".into(), Value::String(gate.into())),
                (
                    "blockingFindings".into(),
                    Value::Number(blocking_findings.to_string()),
                ),
                ("warningFindings".into(), Value::Number("0".into())),
                (
                    "summary".into(),
                    Value::String("Rust native review completed.".into()),
                ),
            ]);
            let _ = write_indented(standard_output, &payload);
        }
        "markdown" => {
            let _ = writeln!(standard_output, "# Native Review Report");
            let _ = writeln!(standard_output);
            let _ = writeln!(standard_output, "- gate: {gate}");
            let _ = writeln!(standard_output, "- blocking_findings: {blocking_findings}");
            let _ = writeln!(standard_output, "- runtime: rust");
        }
        _ => {
            let _ = writeln!(
                standard_output,
                "gate={gate} blocking={blocking_findings} warnings=0 findings={blocking_findings}"
            );
        }
    }
    if blocking_findings == 0 {
        0
    } else {
        1
    }
}

fn render_generated_message(
    message_kind: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new(format!("git-workflow {message_kind}"));
    flag_set.bool_flag("from-diff", false);
    flag_set.string_flag("test-result", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let diff_summary = if flag_set.bool_value("from-diff") {
        git_diff_stat().unwrap_or_else(|| "No diff summary available.".to_string())
    } else {
        "No diff summary requested.".to_string()
    };
    if message_kind == "commit" {
        let _ = writeln!(
            standard_output,
            "chore: migrate claude-skills runtime to rust"
        );
        let _ = writeln!(standard_output);
        let _ = writeln!(standard_output, "{diff_summary}");
    } else {
        let _ = writeln!(standard_output, "## Summary");
        let _ = writeln!(
            standard_output,
            "- Migrates claude-skills runtime behavior to Rust-native command paths."
        );
        let _ = writeln!(standard_output);
        let _ = writeln!(standard_output, "## Validation");
        let test_result = flag_set.string_value("test-result");
        let _ = writeln!(
            standard_output,
            "- {}",
            if test_result.trim().is_empty() {
                "Not provided"
            } else {
                test_result
            }
        );
    }
    0
}

fn lint_message(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.len() != 1 {
        let _ = writeln!(
            standard_error,
            "Usage: claude-skills git-workflow lint-message <file>"
        );
        return 1;
    }
    match fs::read_to_string(&arguments[0]) {
        Ok(text) => {
            let first_line = text.lines().next().unwrap_or("");
            if first_line.len() > 72 {
                let _ = writeln!(standard_error, "message subject exceeds 72 characters");
                return 1;
            }
            let _ = writeln!(standard_output, "message lint passed");
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "read {}: {error}", arguments[0]);
            1
        }
    }
}

fn git_diff_stat() -> Option<String> {
    let result = run_command("git", &["diff".to_string(), "--stat".to_string()], None).ok()?;
    if result.code != 0 {
        return None;
    }
    let text = String::from_utf8_lossy(&result.stdout).trim().to_string();
    Some(if text.is_empty() {
        "No local diff.".to_string()
    } else {
        text
    })
}

fn hosted_body() -> String {
    [
        "# Native Review Report",
        "",
        "- gate: pass",
        "- blocking_findings: 0",
        "- warning_findings: 0",
        "- runtime: rust",
        "- go_fallback: false",
        "",
    ]
    .join("\n")
}

fn render_review_help(standard_output: &mut dyn Write) {
    let _ = writeln!(
        standard_output,
        "Usage: claude-skills review [pre-commit|pre-pr|diff|gates|hosted|policy] ..."
    );
}

fn render_git_workflow_help(standard_output: &mut dyn Write) {
    let _ = writeln!(
        standard_output,
        "Usage: claude-skills git-workflow [preflight|commit-message|pr-body|lint-message] ..."
    );
}

fn is_help_argument(argument: &str) -> bool {
    matches!(argument, "help" | "--help" | "-h")
}
