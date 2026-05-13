//! Purpose: Rust-native command execution, rewrite, and hook-management surfaces.
//! Caller: commands.rs for `run`, `rewrite`, and `hook` command groups.
//! Dependencies: args, json, runtime helpers, std::env, std::fs, std::io, std::path, and std::time.
//! Main Functions: run_run_command, run_rewrite_command, run_hook_command.
//! Side Effects: Spawns requested child commands, writes raw-output recovery logs, and may write or remove Claude Code hook configuration.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::{Map as JsonMap, Value as JsonDocument};

use crate::args::FlagSet;
use crate::json::{write_indented, Value};
use crate::runtime::{display_path, resolve_claude_home, run_command, write_text};

const CLAUDE_HOOK_EVENTS: &[&str] = crate::hooks::claude::EVENTS;
const MANAGED_PRE_TOOL_USE_EVENT: &str = "PreToolUse";
const MANAGED_PRE_TOOL_USE_MATCHER: &str = crate::hooks::claude::pre_tool_matcher();
const MANAGED_PRE_TOOL_USE_COMMAND_SUFFIX: &str = "hook pre-tool-use";
const MANAGED_PRE_TOOL_USE_STATUS: &str =
    "Transparently rewriting noisy commands via claude-skills run";
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn run_run_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    crate::proxy::run::run_proxy(arguments, standard_output, standard_error)
}

pub fn run_rewrite_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("rewrite");
    flag_set.bool_flag("json", false);
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let command = flag_set.positional.join(" ");
    let command = command.trim();
    if command.is_empty() {
        let _ = writeln!(
            standard_error,
            "Usage: claude-skills rewrite [--json] \"<command>\""
        );
        return 1;
    }
    let rewrite = rewrite_command_text(command);
    if flag_set.bool_value("json") {
        let adapter_name = adapter_name_for_rewrite(command);
        let risk = if rewrite.supported {
            "low"
        } else if rewrite.reason.contains("already") {
            "none"
        } else {
            "unsupported"
        };
        let payload = Value::Object(vec![
            (
                "original_command".into(),
                Value::String(command.to_string()),
            ),
            (
                "rewritten_command".into(),
                Value::String(rewrite.rewritten_command.clone()),
            ),
            ("originalCommand".into(), Value::String(command.to_string())),
            (
                "rewrittenCommand".into(),
                Value::String(rewrite.rewritten_command),
            ),
            ("supported".into(), Value::Bool(rewrite.supported)),
            ("reason".into(), Value::String(rewrite.reason)),
            (
                "adapter_name".into(),
                Value::String(adapter_name.to_string()),
            ),
            (
                "adapterName".into(),
                Value::String(adapter_name.to_string()),
            ),
            ("risk".into(), Value::String(risk.to_string())),
        ]);
        let _ = write_indented(standard_output, &payload);
        return if rewrite.supported { 0 } else { 1 };
    }
    if !rewrite.supported {
        let _ = writeln!(standard_error, "{}", rewrite.reason);
        return 1;
    }
    let _ = writeln!(standard_output, "{}", rewrite.rewritten_command);
    0
}

pub fn rewrite_for_doctor(command: &str) -> String {
    rewrite_command_text(command).rewritten_command
}

fn adapter_name_for_rewrite(command: &str) -> &'static str {
    let analysis = analyze_command_text(command);
    let Some(program) = analysis
        .effective_fields
        .first()
        .map(|value| command_base_name(value))
    else {
        return "none";
    };
    match program.as_str() {
        "cargo" => {
            if analysis
                .effective_fields
                .iter()
                .any(|arg| arg == "test" || arg == "nextest")
            {
                "tests"
            } else if analysis
                .effective_fields
                .iter()
                .any(|arg| matches!(arg.as_str(), "build" | "check" | "clippy"))
            {
                if analysis.effective_fields.iter().any(|arg| arg == "clippy") {
                    "lint"
                } else {
                    "build"
                }
            } else {
                "generic"
            }
        }
        "pytest" | "jest" | "vitest" | "playwright" => "tests",
        "deno" if analysis.effective_fields.iter().any(|arg| arg == "test") => "tests",
        "deno" if analysis.effective_fields.iter().any(|arg| arg == "lint") => "lint",
        "deno" if analysis.effective_fields.iter().any(|arg| arg == "fmt") => "build",
        "python"
            if analysis.effective_fields.get(1).map(String::as_str) == Some("-m")
                && analysis.effective_fields.get(2).map(String::as_str) == Some("pytest") =>
        {
            "tests"
        }
        "go" if analysis.effective_fields.iter().any(|arg| arg == "test") => "tests",
        "go" if analysis
            .effective_fields
            .iter()
            .any(|arg| matches!(arg.as_str(), "build" | "vet")) =>
        {
            "build"
        }
        "npm" | "pnpm" | "yarn" | "bun" => package_manager_adapter_name_for_rewrite(
            analysis.effective_fields.iter().skip(1).map(String::as_str),
        ),
        "npx" | "pnpx" | "dlx" => package_manager_adapter_name_for_rewrite(
            analysis.effective_fields.iter().skip(1).map(String::as_str),
        ),
        "git" | "gh" => "git",
        "rg" | "grep" => "search",
        "find" | "cat" | "sed" | "head" | "tail" | "ls" | "tree" | "jq" => "files",
        "ruff" | "eslint" | "mypy" | "biome" | "rubocop" | "flake8" | "golangci-lint" => "lint",
        "tsc" | "prettier" | "webpack" | "vite" | "next" | "black" | "isort" => "build",
        "rspec" | "phpunit" => "tests",
        "rake" if analysis.effective_fields.iter().any(|arg| arg == "test") => "tests",
        "rake" => "build",
        "bundle" | "composer"
            if analysis
                .effective_fields
                .iter()
                .any(|arg| ["install", "update", "add", "require"].contains(&arg.as_str())) =>
        {
            "build"
        }
        "bundle" | "composer" => "build",
        "nx" if analysis.effective_fields.iter().any(|arg| arg == "test") => "tests",
        "nx" if analysis.effective_fields.iter().any(|arg| arg == "lint") => "lint",
        "nx" if analysis.effective_fields.iter().any(|arg| arg == "build") => "build",
        "brew" | "apt" | "apt-get" => "build",
        "docker" | "kubectl" | "terraform" | "aws" => "logs",
        "dotnet" | "mvn" | "gradle"
            if analysis.effective_fields.iter().any(|arg| arg == "test") =>
        {
            "tests"
        }
        "dotnet" | "mvn" | "gradle"
            if analysis
                .effective_fields
                .iter()
                .any(|arg| arg == "build" || arg == "vet") =>
        {
            "build"
        }
        "make" => "build",
        "curl" | "wget" => "logs",
        "pip" | "pip3" => "build",
        "journalctl" | "systemctl" => "logs",
        _ => "generic",
    }
}

fn package_manager_adapter_name_for_rewrite<'a>(
    args: impl Iterator<Item = &'a str>,
) -> &'static str {
    let args: Vec<&str> = args.collect();
    if args
        .iter()
        .any(|arg| matches!(*arg, "test" | "jest" | "vitest") || arg.ends_with(":test"))
    {
        "tests"
    } else if args.contains(&"build")
        || args
            .iter()
            .any(|arg| matches!(*arg, "install" | "i" | "ci" | "add" | "update"))
    {
        "build"
    } else {
        "generic"
    }
}

pub fn run_hook_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        render_hook_help(standard_output);
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "install" => run_hook_install(standard_output, standard_error),
        "uninstall" => run_hook_uninstall(standard_output, standard_error),
        "list" => run_hook_list(standard_output, standard_error),
        "show" => run_hook_list(standard_output, standard_error),
        "instructions" => run_hook_instructions(&arguments[1..], standard_output, standard_error),
        "pre-tool-use" => run_hook_pre_tool_use(standard_output, standard_error),
        "post-tool-use" | "permission-request" | "notification" | "user-prompt-submit" | "stop"
        | "subagent-stop" | "pre-compact" | "post-compact" | "session-start" | "session-end" => {
            run_hook_lifecycle_noop()
        }
        other => {
            let _ = writeln!(standard_error, "Unknown hook command: {other}");
            render_hook_help(standard_output);
            1
        }
    }
}

pub fn run_raw_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.first().map(String::as_str) == Some("list") {
        return run_raw_list(standard_output, standard_error);
    }
    if arguments.first().map(String::as_str) == Some("prune") {
        return run_raw_prune(&arguments[1..], standard_output, standard_error);
    }
    let mut flag_set = FlagSet::new("raw");
    flag_set.bool_flag("path", false);
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let Some(raw_id) = flag_set.positional.first() else {
        let _ = writeln!(
            standard_error,
            "Usage: claude-skills raw [--path] <raw_id> | raw list | raw prune --older-than <Nd>"
        );
        return 1;
    };
    let store = crate::proxy::raw_store::RawStore::new();
    let raw_dir = match store.find_dir(raw_id) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    if flag_set.bool_value("path") {
        let _ = writeln!(standard_output, "{}", display_path(&raw_dir));
        return 0;
    }
    let command = fs::read_to_string(raw_dir.join("command.txt")).unwrap_or_default();
    let stdout = fs::read(raw_dir.join("stdout.log")).unwrap_or_default();
    let stderr = fs::read(raw_dir.join("stderr.log")).unwrap_or_default();
    let _ = writeln!(standard_output, "raw_id: {raw_id}");
    let _ = writeln!(standard_output, "path: {}", display_path(&raw_dir));
    let _ = writeln!(standard_output, "command: {}", command.trim());
    let _ = writeln!(standard_output, "\n[stdout]");
    let _ = standard_output.write_all(&stdout);
    if !stdout.ends_with(b"\n") {
        let _ = writeln!(standard_output);
    }
    let _ = writeln!(standard_output, "\n[stderr]");
    let _ = standard_output.write_all(&stderr);
    if !stderr.ends_with(b"\n") {
        let _ = writeln!(standard_output);
    }
    0
}

pub fn run_replay_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let Some(raw_id) = arguments.first() else {
        let _ = writeln!(standard_error, "Usage: claude-skills replay <raw_id>");
        return 1;
    };
    let store = crate::proxy::raw_store::RawStore::new();
    let meta = match store.load_meta(raw_id) {
        Ok(meta) => meta,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    if !meta.cwd.is_dir() {
        let _ = writeln!(
            standard_error,
            "Saved cwd no longer exists: {}",
            display_path(&meta.cwd)
        );
        return 1;
    }
    let (program, args) = replay_shell_parts(&meta.command);
    match run_command(&program, &args, Some(&meta.cwd)) {
        Ok(result) => {
            let _ = standard_output.write_all(&result.stdout);
            let _ = standard_error.write_all(&result.stderr);
            result.code.clamp(0, 255) as u8
        }
        Err(error) => {
            let _ = writeln!(standard_error, "Unable to replay command: {error}");
            1
        }
    }
}

fn run_raw_list(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let store = crate::proxy::raw_store::RawStore::new();
    let entries = match store.list() {
        Ok(entries) => entries,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let _ = writeln!(standard_output, "raw store: {}", display_path(store.root()));
    if entries.is_empty() {
        let _ = writeln!(standard_output, "no raw outputs found");
        return 0;
    }
    for entry in entries.iter().take(50) {
        let command = entry
            .meta
            .as_ref()
            .map(|meta| meta.command.as_str())
            .unwrap_or("unknown");
        let _ = writeln!(
            standard_output,
            "{} exit={} adapter={} {}",
            entry.raw_id,
            entry.meta.as_ref().map(|meta| meta.exit_code).unwrap_or(0),
            entry
                .meta
                .as_ref()
                .map(|meta| meta.adapter_name.as_str())
                .unwrap_or("unknown"),
            command
        );
    }
    if entries.len() > 50 {
        let _ = writeln!(
            standard_output,
            "omitted {} older entries",
            entries.len() - 50
        );
    }
    0
}

fn run_raw_prune(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("raw prune");
    flag_set.string_flag("older-than", "30d");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let days = parse_days(flag_set.string_value("older-than")).unwrap_or(30);
    let store = crate::proxy::raw_store::RawStore::new();
    match store.prune_older_than(days) {
        Ok(count) => {
            let _ = writeln!(
                standard_output,
                "raw prune: removed {count} entries older than {days}d"
            );
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            1
        }
    }
}

fn parse_days(value: &str) -> Option<u64> {
    value.trim_end_matches('d').parse().ok()
}

fn replay_shell_parts(command: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        (
            "cmd".to_string(),
            vec!["/C".to_string(), command.to_string()],
        )
    } else {
        (
            "bash".to_string(),
            vec!["-lc".to_string(), command.to_string()],
        )
    }
}

fn run_hook_install(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let claude_home = match resolve_claude_home("") {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let hook_path = claude_home.join(crate::hooks::claude::SETTINGS_FILE_NAME);
    let hook_command = match managed_hook_command() {
        Ok(command) => command,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let hook_payload = match build_hooks_payload(&hook_path, &hook_command) {
        Ok(payload) => payload,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    match write_text(&hook_path, &hook_payload) {
        Ok(()) => {
            let _ = writeln!(
                standard_output,
                "Installed Rust claude-skills lifecycle hooks at {}",
                display_path(&hook_path)
            );
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            1
        }
    }
}

fn run_hook_uninstall(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let claude_home = match resolve_claude_home("") {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let hook_path = claude_home.join(crate::hooks::claude::SETTINGS_FILE_NAME);
    match remove_managed_hook_payload(&hook_path) {
        Ok((payload, removed)) => {
            if removed {
                match write_text(&hook_path, &payload) {
                    Ok(()) => {
                        let _ = writeln!(
                            standard_output,
                            "Removed Rust claude-skills hook from {}",
                            display_path(&hook_path)
                        );
                        0
                    }
                    Err(error) => {
                        let _ = writeln!(standard_error, "{error}");
                        1
                    }
                }
            } else {
                let _ = writeln!(
                    standard_output,
                    "No claude-skills hook installed at {}",
                    display_path(&hook_path)
                );
                0
            }
        }
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            1
        }
    }
}

fn run_hook_list(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let claude_home = match resolve_claude_home("") {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let hook_path = claude_home.join(crate::hooks::claude::SETTINGS_FILE_NAME);
    match fs::read_to_string(&hook_path) {
        Ok(text) => {
            let _ = writeln!(standard_output, "{text}");
            0
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let _ = writeln!(
                standard_output,
                "No claude-skills hook installed at {}",
                display_path(&hook_path)
            );
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "read {}: {error}", display_path(&hook_path));
            1
        }
    }
}

fn run_hook_instructions(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("hook instructions");
    flag_set.string_flag("format", "markdown");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    if flag_set.string_value("format") == "json" {
        let payload = Value::Object(vec![
            ("runtime".into(), Value::String("rust".into())),
            (
                "rerunPrefix".into(),
                Value::String("claude-skills run --".into()),
            ),
            (
                "activeHookEvent".into(),
                Value::String(MANAGED_PRE_TOOL_USE_EVENT.into()),
            ),
            (
                "supportedHookEvents".into(),
                Value::Array(
                    CLAUDE_HOOK_EVENTS
                        .iter()
                        .map(|event| Value::String((*event).into()))
                        .collect(),
                ),
            ),
            ("semanticReducers".into(), Value::Bool(true)),
            (
                "streamingMode".into(),
                Value::String(
                    "bounded live output with --stream; full raw recovery always saved".into(),
                ),
            ),
            ("goFallback".into(), Value::Bool(false)),
        ]);
        let _ = write_indented(standard_output, &payload);
        return 0;
    }
    let _ = writeln!(
        standard_output,
        "claude-skills PreToolUse hook transparently rewrites noisy shell commands via `claude-skills run -- <command>`. No manual rerun needed."
    );
    let _ = writeln!(
        standard_output,
        "Claude Code exposes hook events including: {}.",
        CLAUDE_HOOK_EVENTS.join(", ")
    );
    let _ = writeln!(
        standard_output,
        "claude-skills installs managed entries for every supported lifecycle event; `PreToolUse` silently rewrites supported Bash commands with native compaction."
    );
    let _ = writeln!(
        standard_output,
        "The Rust runtime uses native semantic reducers, raw recovery, gain analytics, and no Go or third-party compaction fallback."
    );
    0
}

fn run_hook_pre_tool_use(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let input_text = match std::io::read_to_string(std::io::stdin()) {
        Ok(text) => text,
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Unable to read Claude Code hook input: {error}"
            );
            return 1;
        }
    };
    let input: JsonDocument = match serde_json::from_str(&input_text) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Unable to decode Claude Code hook input: {error}"
            );
            return 1;
        }
    };
    if input
        .get("tool_name")
        .and_then(JsonDocument::as_str)
        .unwrap_or_default()
        != MANAGED_PRE_TOOL_USE_MATCHER
    {
        return 0;
    }
    let command = input
        .get("tool_input")
        .and_then(|tool_input| tool_input.get("command"))
        .and_then(JsonDocument::as_str)
        .unwrap_or_default();
    let rewrite = rewrite_command_text_for_shell(command, RewriteShell::Bash);
    if !rewrite.supported {
        return 0;
    }
    let payload = serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": MANAGED_PRE_TOOL_USE_EVENT,
            "permissionDecision": "allow",
            "updatedInput": {
                "command": rewrite.rewritten_command,
            },
        }
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(rendered) => {
            let _ = writeln!(standard_output, "{rendered}");
            0
        }
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Unable to render Claude Code hook output: {error}"
            );
            1
        }
    }
}

fn run_hook_lifecycle_noop() -> u8 {
    0
}

fn render_hook_help(standard_output: &mut dyn Write) {
    let _ = writeln!(
        standard_output,
        "Usage: claude-skills hook [install|uninstall|list|show|instructions|pre-tool-use|post-tool-use|permission-request|notification|user-prompt-submit|stop|subagent-stop|pre-compact|post-compact|session-start|session-end]"
    );
}

struct RewriteDecision {
    rewritten_command: String,
    supported: bool,
    reason: String,
}

#[derive(Clone, Copy)]
enum RewriteShell {
    Bash,
    PlatformDefault,
}

fn rewrite_command_text(command: &str) -> RewriteDecision {
    rewrite_command_text_for_shell(command, RewriteShell::PlatformDefault)
}

fn rewrite_command_text_for_shell(command: &str, shell: RewriteShell) -> RewriteDecision {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return RewriteDecision {
            rewritten_command: String::new(),
            supported: false,
            reason: "empty command".into(),
        };
    }
    let analysis = analyze_command_text(trimmed);
    if is_already_compaction_wrapped(&analysis.effective_fields) {
        return RewriteDecision {
            rewritten_command: trimmed.to_string(),
            supported: false,
            reason: "command already uses claude-skills run".into(),
        };
    }
    if !is_supported_noisy_command(&analysis.effective_fields) {
        return RewriteDecision {
            rewritten_command: String::new(),
            supported: false,
            reason: "no native claude-skills compaction filter for command".into(),
        };
    }
    let runnable_command = if analysis.requires_shell_wrapper {
        format!("bash -lc {}", shell_quote(trimmed))
    } else {
        trimmed.to_string()
    };
    RewriteDecision {
        rewritten_command: format!("{} {runnable_command}", compaction_command_prefix(shell)),
        supported: true,
        reason: String::new(),
    }
}

struct CommandAnalysis {
    effective_fields: Vec<String>,
    requires_shell_wrapper: bool,
}

fn analyze_command_text(command: &str) -> CommandAnalysis {
    let segments = split_shell_segments(command);
    let words = segments.first().cloned().unwrap_or_default();
    let mut effective_fields = effective_command_fields(&words, 0);
    for segment in segments.iter().skip(1) {
        let candidate = effective_command_fields(segment, 0);
        if is_supported_noisy_command(&candidate) || is_already_compaction_wrapped(&candidate) {
            effective_fields = candidate;
            break;
        }
    }
    let first_effective = effective_fields
        .first()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let first_word = words.first().cloned().unwrap_or_default();
    CommandAnalysis {
        effective_fields,
        requires_shell_wrapper: contains_shell_syntax(command)
            || words
                .first()
                .map(|value| is_env_assignment(value))
                .unwrap_or(false)
            || (!first_word.is_empty()
                && !first_effective.is_empty()
                && command_base_name(&first_word) != command_base_name(&first_effective)
                && !matches!(
                    command_base_name(&first_word).as_str(),
                    "env" | "time" | "command" | "exec"
                )),
    }
}

fn split_shell_segments(command: &str) -> Vec<Vec<String>> {
    let (words, operators) = shell_words_and_operators(command);
    if words.is_empty() {
        return Vec::new();
    }
    let mut segments = Vec::new();
    let mut current = Vec::new();
    let mut operator_index = 0usize;
    for word in words {
        if is_shell_separator_token(&word) {
            if !current.is_empty() {
                segments.push(current);
                current = Vec::new();
            }
            operator_index += 1;
            continue;
        }
        current.push(word);
    }
    if !current.is_empty() {
        segments.push(current);
    }
    if segments.is_empty() && !operators.is_empty() && operator_index == 0 {
        segments.push(Vec::new());
    }
    segments
}

fn shell_words_and_operators(command: &str) -> (Vec<String>, Vec<String>) {
    let mut words = Vec::new();
    let mut operators = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let characters: Vec<char> = command.chars().collect();
    let mut index = 0usize;
    while index < characters.len() {
        let character = characters[index];
        if escaped {
            current.push(character);
            escaped = false;
            index += 1;
            continue;
        }
        if character == '\\' && backslash_starts_escape(quote, characters.get(index + 1).copied()) {
            escaped = true;
            index += 1;
            continue;
        }
        if let Some(quote_character) = quote {
            if character == quote_character {
                quote = None;
            } else {
                current.push(character);
            }
            index += 1;
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            index += 1;
            continue;
        }
        if character.is_whitespace() {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            index += 1;
            continue;
        }
        if is_shell_operator_character(character) {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
            let mut operator = character.to_string();
            if let Some(next) = characters.get(index + 1).copied() {
                if matches!((character, next), ('|', '|') | ('&', '&') | ('>', '>')) {
                    operator.push(next);
                    index += 1;
                }
            }
            operators.push(operator.clone());
            words.push(operator);
            index += 1;
            continue;
        }
        current.push(character);
        index += 1;
    }
    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        words.push(current);
    }
    (words, operators)
}

fn backslash_starts_escape(quote: Option<char>, next: Option<char>) -> bool {
    match quote {
        Some('\'') => false,
        Some('"') => next
            .map(|character| matches!(character, '"' | '\\' | '$' | '`' | '\n'))
            .unwrap_or(false),
        _ => next
            .map(|character| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '\'' | '"' | '\\' | '$' | '`' | '|' | '&' | ';' | '<' | '>' | '(' | ')'
                    )
            })
            .unwrap_or(false),
    }
}

fn is_shell_operator_character(character: char) -> bool {
    matches!(
        character,
        '|' | '&' | ';' | '<' | '>' | '\n' | '`' | '(' | ')'
    )
}

fn is_shell_separator_token(value: &str) -> bool {
    matches!(
        value,
        "|" | "||" | "&&" | ";" | "&" | "<" | ">" | ">>" | "\n" | "`" | "(" | ")"
    )
}

fn contains_shell_syntax(command: &str) -> bool {
    !shell_words_and_operators(command).1.is_empty()
}

fn effective_command_fields(words: &[String], depth: usize) -> Vec<String> {
    if depth > 3 {
        return normalize_command_fields(words);
    }
    let mut index = 0;
    while words
        .get(index)
        .map(|value| is_env_assignment(value))
        .unwrap_or(false)
    {
        index += 1;
    }
    let Some(command) = words.get(index).map(|value| command_base_name(value)) else {
        return Vec::new();
    };
    match command.as_str() {
        "env" => {
            index += 1;
            while let Some(value) = words.get(index) {
                if is_env_assignment(value) {
                    index += 1;
                } else if matches!(value.as_str(), "-u" | "--unset" | "-C" | "--chdir") {
                    index += 2;
                } else if value.starts_with("--ignore-environment")
                    || value == "-i"
                    || value.starts_with('-')
                {
                    index += 1;
                } else {
                    break;
                }
            }
            effective_command_fields(&words[index..], depth + 1)
        }
        "time" | "command" | "exec" | "nohup" => {
            effective_command_fields(&words[index + 1..], depth + 1)
        }
        "sudo" | "doas" | "nice" => {
            index += 1;
            while words
                .get(index)
                .map(|value| value.starts_with('-'))
                .unwrap_or(false)
            {
                index += 1;
            }
            effective_command_fields(&words[index..], depth + 1)
        }
        "bash" | "sh" | "zsh" => {
            for (shell_index, word) in words[index + 1..].iter().enumerate() {
                if word.contains('c') && word.starts_with('-') {
                    if let Some(shell_command) = words.get(index + 1 + shell_index + 1) {
                        let nested_segments = split_shell_segments(shell_command);
                        for segment in &nested_segments {
                            let candidate = effective_command_fields(segment, depth + 1);
                            if is_supported_noisy_command(&candidate)
                                || is_already_compaction_wrapped(&candidate)
                            {
                                return candidate;
                            }
                        }
                        let nested_words = nested_segments.first().cloned().unwrap_or_default();
                        return effective_command_fields(&nested_words, depth + 1);
                    }
                }
            }
            normalize_command_fields(&words[index..])
        }
        _ => normalize_command_fields(&words[index..]),
    }
}

fn normalize_command_fields(words: &[String]) -> Vec<String> {
    words
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn is_env_assignment(value: &str) -> bool {
    let Some((name, _)) = value.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .map(|character| character == '_' || character.is_ascii_alphabetic())
            .unwrap_or(false)
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn bash_command_for_executable_args(path: &Path, arguments: &str) -> String {
    format!("{} {arguments}", shell_quote(&display_path(path)))
}

fn platform_default_command_for_executable_args(path: &Path, arguments: &str) -> String {
    let displayed_path = display_path(path);
    if cfg!(windows) {
        format!("& {} {arguments}", powershell_quote(&displayed_path))
    } else {
        bash_command_for_executable_args(path, arguments)
    }
}

fn hook_command_for_executable_args(path: &Path, arguments: &str) -> String {
    if cfg!(windows) {
        let script = platform_default_command_for_executable_args(path, arguments);
        format!(
            "powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand {}",
            powershell_encoded_command(&script)
        )
    } else {
        bash_command_for_executable_args(path, arguments)
    }
}

fn powershell_encoded_command(script: &str) -> String {
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.push((unit & 0x00ff) as u8);
        bytes.push((unit >> 8) as u8);
    }
    base64_encode(&bytes)
}

fn base64_encode(bytes: &[u8]) -> String {
    let mut rendered = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        rendered.push(BASE64_ALPHABET[(first >> 2) as usize] as char);
        rendered
            .push(BASE64_ALPHABET[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            rendered.push(
                BASE64_ALPHABET[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char,
            );
        } else {
            rendered.push('=');
        }
        if chunk.len() > 2 {
            rendered.push(BASE64_ALPHABET[(third & 0b0011_1111) as usize] as char);
        } else {
            rendered.push('=');
        }
    }
    rendered
}

fn base64_decode(value: &str) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(value.len() / 4 * 3);
    let mut chunk = [0u8; 4];
    let mut chunk_len = 0usize;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        let decoded = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => 64,
            _ => return None,
        };
        chunk[chunk_len] = decoded;
        chunk_len += 1;
        if chunk_len != 4 {
            continue;
        }
        output.push((chunk[0] << 2) | (chunk[1] >> 4));
        if chunk[2] != 64 {
            output.push((chunk[1] << 4) | (chunk[2] >> 2));
        }
        if chunk[3] != 64 {
            output.push((chunk[2] << 6) | chunk[3]);
        }
        chunk_len = 0;
    }
    if chunk_len == 0 {
        Some(output)
    } else {
        None
    }
}

fn decode_powershell_encoded_command(command: &str) -> Option<String> {
    let mut words = command.split_whitespace();
    while let Some(word) = words.next() {
        if !word.eq_ignore_ascii_case("-EncodedCommand") {
            continue;
        }
        let encoded = words.next()?.trim_matches('"').trim_matches('\'');
        let bytes = base64_decode(encoded)?;
        if bytes.len() % 2 != 0 {
            return None;
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        return String::from_utf16(&units).ok();
    }
    None
}

fn is_already_compaction_wrapped(fields: &[String]) -> bool {
    if fields.len() >= 2 && command_base_name(&fields[0]) == "claude-skills" && fields[1] == "run" {
        return true;
    }
    fields.len() >= 2
        && command_base_name(&fields[0]).starts_with("claude-skills")
        && fields[1] == "run"
}

fn is_supported_noisy_command(fields: &[String]) -> bool {
    let Some(command) = fields.first().map(|value| command_base_name(value)) else {
        return false;
    };
    if matches!(
        command.as_str(),
        "cargo"
            | "npm"
            | "pnpm"
            | "yarn"
            | "bun"
            | "npx"
            | "pnpx"
            | "dlx"
            | "pytest"
            | "ruff"
            | "eslint"
            | "tsc"
            | "vitest"
            | "jest"
            | "playwright"
            | "deno"
            | "gradle"
            | "mvn"
            | "make"
            | "go"
            | "dotnet"
            | "docker"
            | "kubectl"
            | "terraform"
            | "aws"
            | "gh"
            | "git"
            | "rg"
            | "grep"
            | "find"
            | "cat"
            | "head"
            | "tail"
            | "sed"
            | "ls"
            | "tree"
            | "jq"
            | "mypy"
            | "prettier"
            | "biome"
            | "curl"
            | "wget"
            | "pip"
            | "pip3"
            | "journalctl"
            | "systemctl"
            | "rake"
            | "rspec"
            | "rubocop"
            | "bundle"
            | "flake8"
            | "black"
            | "isort"
            | "poetry"
            | "webpack"
            | "vite"
            | "nx"
            | "next"
            | "golangci-lint"
            | "composer"
            | "phpunit"
            | "brew"
            | "apt"
            | "apt-get"
    ) {
        return true;
    }
    command == "python"
        && fields.get(1).map(String::as_str) == Some("-m")
        && fields.get(2).map(String::as_str) == Some("pytest")
}

fn command_base_name(command: &str) -> String {
    let normalized = command.replace('\\', "/");
    let base_name = normalized.rsplit('/').next().unwrap_or(command);
    base_name
        .trim_end_matches(".exe")
        .trim_end_matches(".cmd")
        .trim_end_matches(".bat")
        .to_string()
}

fn compaction_command_prefix(shell: RewriteShell) -> String {
    match std::env::current_exe() {
        Ok(path) => match shell {
            RewriteShell::Bash => bash_command_for_executable_args(&path, "run --"),
            RewriteShell::PlatformDefault => {
                platform_default_command_for_executable_args(&path, "run --")
            }
        },
        Err(_) => "claude-skills run --".to_string(),
    }
}

fn managed_hook_command() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| hook_command_for_executable_args(&path, MANAGED_PRE_TOOL_USE_COMMAND_SUFFIX))
        .map_err(|error| format!("resolve current executable: {error}"))
}

fn build_hooks_payload(hook_path: &Path, hook_command: &str) -> Result<String, String> {
    let mut document = read_hooks_document(hook_path)?;
    ensure_hooks_object(&mut document)?;
    remove_managed_hooks(&mut document);
    append_managed_hooks(&mut document, hook_command)?;
    serde_json::to_string_pretty(&document)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|error| format!("render hooks config: {error}"))
}

fn remove_managed_hook_payload(hook_path: &Path) -> Result<(String, bool), String> {
    let mut document = read_hooks_document(hook_path)?;
    let before = serde_json::to_string(&document).unwrap_or_default();
    ensure_hooks_object(&mut document)?;
    remove_managed_hooks(&mut document);
    let after = serde_json::to_string(&document).unwrap_or_default();
    let rendered = serde_json::to_string_pretty(&document)
        .map(|value| format!("{value}\n"))
        .map_err(|error| format!("render hooks config: {error}"))?;
    Ok((rendered, before != after))
}

fn read_hooks_document(hook_path: &Path) -> Result<JsonDocument, String> {
    match fs::read_to_string(hook_path) {
        Ok(text) if text.trim().is_empty() => Ok(serde_json::json!({"hooks": {}})),
        Ok(text) => serde_json::from_str(&text)
            .map_err(|error| format!("parse {}: {error}", display_path(hook_path))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(serde_json::json!({"hooks": {}}))
        }
        Err(error) => Err(format!("read {}: {error}", display_path(hook_path))),
    }
}

fn ensure_hooks_object(document: &mut JsonDocument) -> Result<(), String> {
    if !document.is_object() {
        *document = serde_json::json!({"hooks": {}});
        return Ok(());
    }
    let object = document.as_object_mut().expect("object checked");
    if !object.contains_key("hooks") {
        object.insert("hooks".into(), JsonDocument::Object(JsonMap::new()));
    }
    if !object
        .get("hooks")
        .map(JsonDocument::is_object)
        .unwrap_or(false)
    {
        return Err("settings.json contains a non-object hooks field".into());
    }
    Ok(())
}

fn remove_managed_hooks(document: &mut JsonDocument) {
    let Some(hooks) = document
        .get_mut("hooks")
        .and_then(JsonDocument::as_object_mut)
    else {
        return;
    };
    for (_event_name, event_entries) in hooks.iter_mut() {
        let Some(entries) = event_entries.as_array_mut() else {
            continue;
        };
        for matcher_entry in entries.iter_mut() {
            let Some(commands) = matcher_entry
                .get_mut("hooks")
                .and_then(JsonDocument::as_array_mut)
            else {
                continue;
            };
            commands.retain(|command_entry| {
                !command_entry
                    .get("command")
                    .and_then(JsonDocument::as_str)
                    .map(is_managed_hook_command)
                    .unwrap_or(false)
            });
        }
        entries.retain(|matcher_entry| {
            matcher_entry
                .get("hooks")
                .and_then(JsonDocument::as_array)
                .map(|commands| !commands.is_empty())
                .unwrap_or(true)
        });
    }
}

fn append_managed_hooks(document: &mut JsonDocument, hook_command: &str) -> Result<(), String> {
    let hooks = document
        .get_mut("hooks")
        .and_then(JsonDocument::as_object_mut)
        .ok_or_else(|| "settings.json missing hooks object".to_string())?;
    for event in CLAUDE_HOOK_EVENTS {
        let event_entries = hooks
            .entry((*event).to_string())
            .or_insert_with(|| JsonDocument::Array(Vec::new()));
        let event_array = event_entries
            .as_array_mut()
            .ok_or_else(|| format!("{event} hooks entry is not an array"))?;
        let (matcher, command, status) = managed_hook_entry_for_event(event, hook_command);
        event_array.push(serde_json::json!({
            "matcher": matcher,
            "hooks": [{
                "type": "command",
                "command": command,
                "statusMessage": status
            }]
        }));
    }
    sort_hook_events(hooks);
    Ok(())
}

fn managed_hook_entry_for_event(
    event: &str,
    pre_tool_use_command: &str,
) -> (&'static str, String, &'static str) {
    if event == "PreToolUse" {
        return (
            MANAGED_PRE_TOOL_USE_MATCHER,
            pre_tool_use_command.to_string(),
            MANAGED_PRE_TOOL_USE_STATUS,
        );
    }
    let subcommand = crate::hooks::claude::lifecycle_subcommand(event);
    (
        "",
        managed_lifecycle_command(subcommand),
        crate::hooks::claude::status_message(event),
    )
}

fn managed_lifecycle_command(subcommand: &str) -> String {
    match std::env::current_exe() {
        Ok(path) => hook_command_for_executable_args(&path, &format!("hook {subcommand}")),
        Err(_) => format!("claude-skills hook {subcommand}"),
    }
}

fn sort_hook_events(hooks: &mut JsonMap<String, JsonDocument>) {
    let sorted: BTreeMap<String, JsonDocument> = hooks
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    hooks.clear();
    for (key, value) in sorted {
        hooks.insert(key, value);
    }
}

fn is_managed_hook_command(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase();
    let has_any_lifecycle = CLAUDE_HOOK_EVENTS.iter().any(|event| {
        let subcommand = crate::hooks::claude::lifecycle_subcommand(event);
        normalized.contains(&format!("hook {subcommand}"))
    });
    let plain_managed = normalized.contains("claude-skills")
        && (has_any_lifecycle || normalized.contains("hook instructions --format json"));
    plain_managed
        || decode_powershell_encoded_command(command)
            .map(|decoded| is_managed_hook_command(&decoded))
            .unwrap_or(false)
}

fn is_help_argument(argument: &str) -> bool {
    matches!(argument, "help" | "--help" | "-h")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rewrite_only_wraps_supported_noisy_commands() {
        let cargo = rewrite_command_text("cargo test --workspace");
        assert!(cargo.supported);
        assert!(cargo
            .rewritten_command
            .contains(" run -- cargo test --workspace"));

        let wrapped = rewrite_command_text("claude-skills run -- cargo test --workspace");
        assert!(!wrapped.supported);
        assert_eq!(wrapped.reason, "command already uses claude-skills run");

        let quiet = rewrite_command_text("echo hello");
        assert!(!quiet.supported);
        assert_eq!(
            quiet.reason,
            "no native claude-skills compaction filter for command"
        );
    }

    #[test]
    fn rewrite_adapter_metadata_matches_package_manager_subcommands() {
        assert_eq!(adapter_name_for_rewrite("npm test"), "tests");
        assert_eq!(adapter_name_for_rewrite("pnpm vitest"), "tests");
        assert_eq!(adapter_name_for_rewrite("yarn unit:test"), "tests");
        assert_eq!(adapter_name_for_rewrite("npm run build"), "build");
        assert_eq!(adapter_name_for_rewrite("npm install"), "build");
        assert_eq!(adapter_name_for_rewrite("yarn add vite"), "build");
        assert_eq!(adapter_name_for_rewrite("npm publish"), "generic");
    }

    #[test]
    fn rewrite_adapter_metadata_distinguishes_build_and_lint() {
        assert_eq!(adapter_name_for_rewrite("cargo build"), "build");
        assert_eq!(adapter_name_for_rewrite("cargo clippy"), "lint");
        assert_eq!(adapter_name_for_rewrite("eslint ."), "lint");
        assert_eq!(adapter_name_for_rewrite("tsc --noEmit"), "build");
    }

    #[test]
    fn rewrite_understands_shell_wrappers_and_env_prefixes() {
        let env_prefixed = rewrite_command_text("RUST_LOG=debug cargo test --workspace");
        assert!(env_prefixed.supported);
        assert!(env_prefixed.rewritten_command.contains("bash -lc"));
        assert!(env_prefixed
            .rewritten_command
            .contains("RUST_LOG=debug cargo test --workspace"));

        let shell_wrapped = rewrite_command_text("bash -lc 'cargo test --workspace'");
        assert!(shell_wrapped.supported);
        assert!(shell_wrapped.rewritten_command.contains("bash -lc"));
        assert!(shell_wrapped
            .rewritten_command
            .contains("cargo test --workspace"));

        let pipeline = rewrite_command_text("cargo test --workspace | tee test.log");
        assert!(pipeline.supported);
        assert!(pipeline.rewritten_command.contains("bash -lc"));
    }

    #[test]
    fn rewrite_parser_handles_late_supported_segments_and_windows_paths() {
        let late_supported = rewrite_command_text("printf ok | grep ok && cargo test --workspace");
        assert!(late_supported.supported);
        assert!(late_supported.rewritten_command.contains("bash -lc"));
        assert!(late_supported
            .rewritten_command
            .contains("cargo test --workspace"));

        let windows_path = rewrite_command_text(r#""C:\tools\cargo.exe" test --workspace"#);
        assert!(windows_path.supported);
        assert!(windows_path
            .rewritten_command
            .contains(r#""C:\tools\cargo.exe" test --workspace"#));

        let already_wrapped =
            rewrite_command_text(r#""C:\Users\me\.claude\claude-skills.exe" run -- cargo test"#);
        assert!(!already_wrapped.supported);
        assert_eq!(
            already_wrapped.reason,
            "command already uses claude-skills run"
        );
    }

    #[test]
    fn hook_payload_preserves_unrelated_events_and_replaces_managed_hook() {
        let hook_path = temp_hook_path("claude-skills-hook-payload");
        std::fs::create_dir_all(hook_path.parent().unwrap()).unwrap();
        std::fs::write(
            &hook_path,
            r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/post_write_figma_parity_check.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "claude-skills hook instructions --format json"
          }
        ]
      }
    ]
  }
}
"#,
        )
        .unwrap();

        let rendered = build_hooks_payload(
            &hook_path,
            r#""C:\tools\claude-skills.exe" hook pre-tool-use"#,
        )
        .unwrap();
        assert!(rendered.contains("PostToolUse"));
        assert!(rendered.contains("PermissionRequest"));
        assert!(rendered.contains("Notification"));
        assert!(rendered.contains("PreCompact"));
        assert!(rendered.contains("PostCompact"));
        assert!(rendered.contains("SessionStart"));
        assert!(rendered.contains("SessionEnd"));
        assert!(rendered.contains("UserPromptSubmit"));
        assert!(rendered.contains("SubagentStop"));
        assert!(rendered.contains("Stop"));
        assert!(rendered.contains("post_write_figma_parity_check"));
        assert!(rendered.contains("hook pre-tool-use"));
        assert!(!rendered.contains("hook instructions --format json"));
        let _ = std::fs::remove_dir_all(hook_path.parent().unwrap());
    }

    #[test]
    fn executable_command_prefixes_are_safe_for_their_target_shells() {
        let path = if cfg!(windows) {
            Path::new(r"C:\Users\Example User's Folder\.claude\claude-skills.exe")
        } else {
            Path::new("/home/example user's folder/.claude/claude-skills")
        };
        let bash_command = bash_command_for_executable_args(path, "run --");
        let platform_command = platform_default_command_for_executable_args(path, "run --");
        let hook_command = hook_command_for_executable_args(path, "hook session-start");

        if cfg!(windows) {
            assert_eq!(
                bash_command,
                r#"'C:\Users\Example User'\''s Folder\.claude\claude-skills.exe' run --"#
            );
            assert_eq!(
                platform_command,
                r#"& 'C:\Users\Example User''s Folder\.claude\claude-skills.exe' run --"#
            );
            assert!(hook_command
                .starts_with("powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand "));
            assert!(!hook_command.contains("Example User"));
        } else {
            assert_eq!(
                bash_command,
                r#"'/home/example user'\''s folder/.claude/claude-skills' run --"#
            );
            assert_eq!(platform_command, bash_command);
            assert_eq!(
                hook_command,
                r#"'/home/example user'\''s folder/.claude/claude-skills' hook session-start"#
            );
        }
    }

    #[test]
    fn pre_tool_rerun_command_stays_bash_compatible_on_windows() {
        let rewrite = rewrite_command_text_for_shell("git status --short", RewriteShell::Bash);
        assert!(rewrite.supported);
        assert!(rewrite
            .rewritten_command
            .contains(" run -- git status --short"));
        assert!(
            !rewrite.rewritten_command.starts_with("& "),
            "Bash rerun command must not start with PowerShell call operator: {}",
            rewrite.rewritten_command
        );
    }

    #[test]
    fn manual_rewrite_uses_platform_default_shell() {
        let rewrite = rewrite_command_text("git status --short");
        assert!(rewrite.supported);

        if cfg!(windows) {
            assert!(
                rewrite.rewritten_command.starts_with("& '"),
                "Windows manual rewrite should be PowerShell-compatible: {}",
                rewrite.rewritten_command
            );
        } else {
            assert!(
                !rewrite.rewritten_command.starts_with("& "),
                "POSIX manual rewrite should not use PowerShell call operator: {}",
                rewrite.rewritten_command
            );
        }
    }

    #[test]
    fn hook_payload_uses_exact_managed_commands_for_each_event() {
        let hook_path = temp_hook_path("claude-skills-hook-command-prefix");
        std::fs::create_dir_all(hook_path.parent().unwrap()).unwrap();
        std::fs::write(&hook_path, r#"{"hooks": {}}"#).unwrap();

        let pre_tool_command = managed_hook_command().unwrap();
        let rendered = build_hooks_payload(&hook_path, &pre_tool_command).unwrap();
        let document: JsonDocument = serde_json::from_str(&rendered).unwrap();
        let hooks = document
            .get("hooks")
            .and_then(JsonDocument::as_object)
            .unwrap();

        for event in CLAUDE_HOOK_EVENTS {
            let commands = hooks
                .get(*event)
                .and_then(JsonDocument::as_array)
                .and_then(|entries| entries.first())
                .and_then(|entry| entry.get("hooks"))
                .and_then(JsonDocument::as_array)
                .unwrap();
            let command = commands
                .first()
                .and_then(|hook| hook.get("command"))
                .and_then(JsonDocument::as_str)
                .unwrap();
            let expected = expected_managed_command_for_event(event, &pre_tool_command);
            assert_eq!(command, expected, "unexpected command for {event}");

            if cfg!(windows) {
                assert!(command.starts_with(
                    "powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand "
                ));
            } else {
                assert!(!command.starts_with("& "));
            }
        }

        let _ = std::fs::remove_dir_all(hook_path.parent().unwrap());
    }

    #[test]
    fn managed_hook_detection_handles_encoded_powershell_commands() {
        let path = Path::new(r"C:\Users\Example User\.claude\claude-skills.exe");
        let command = hook_command_for_executable_args(path, "hook session-start");

        assert!(is_managed_hook_command(&command));
        assert!(!is_managed_hook_command(
            "powershell.exe -NoProfile -EncodedCommand SQBuAHYAYQBsAGkAZAA="
        ));
    }

    fn expected_managed_command_for_event(event: &str, pre_tool_command: &str) -> String {
        if event == "PreToolUse" {
            return pre_tool_command.to_string();
        }
        managed_lifecycle_command(crate::hooks::claude::lifecycle_subcommand(event))
    }

    fn temp_hook_path(name: &str) -> PathBuf {
        let unique = format!("{}-{}", name, std::process::id());
        std::env::temp_dir().join(unique).join("settings.json")
    }
}
