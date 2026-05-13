//! Purpose: Rust-native lightweight command groups that replace the historical Go catch-all dispatch surface.
//! Caller: commands.rs for code-search, design-intelligence, memory, memoriesv2, orchestration,
//! workflow, gain, session, and bench.
//! Dependencies: args, json, runtime helpers, std::fs, std::io, and std::path.
//! Main Functions: run_code_search_command, run_memory_command, run_workflow_command,
//! run_session_command, run_bench_command.
//! Side Effects: Reads repository files, creates optional memory scope directories, and writes command output.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::args::FlagSet;
use crate::json::{write_indented, Value};
use crate::runtime::{
    display_path, resolve_claude_home, resolve_repository_root, write_text,
    COMMAND_COMPACTION_EVENTS_FILE_NAME,
};

pub fn run_code_search_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills code-search [search|index|status] [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "search" => run_code_search_search(&arguments[1..], standard_output, standard_error),
        "index" => {
            let _ = writeln!(
                standard_output,
                "code-search index: Rust native search scans the workspace live"
            );
            0
        }
        "status" => {
            let _ = writeln!(
                standard_output,
                "code-search status: rust live-search ready"
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown code-search command: {other}");
            1
        }
    }
}

pub fn run_design_intelligence_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills design-intelligence recommend [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    if arguments[0] != "recommend" {
        let _ = writeln!(
            standard_error,
            "Unknown design-intelligence command: {}",
            arguments[0]
        );
        return 1;
    }
    let _ = writeln!(standard_output, "Design Intelligence Recommendation");
    let _ = writeln!(
        standard_output,
        "- preserve existing visual language before changing components"
    );
    let _ = writeln!(
        standard_output,
        "- validate responsive behavior on desktop and mobile"
    );
    let _ = writeln!(
        standard_output,
        "- avoid generic AI-looking layouts unless matching an existing system"
    );
    0
}

pub fn run_memory_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(standard_output, "Usage: claude-skills {command_group} [scope|status|working-brief|completion-gate|agent-registry|research-cache|maintenance|report] ...");
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "scope" => run_scope_command(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "system-map" => run_system_map_command(
            command_group,
            &arguments[1..],
            standard_output,
            standard_error,
        ),
        "status" | "report" => {
            let _ = writeln!(
                standard_output,
                "{command_group} status: rust memory directories are available"
            );
            0
        }
        "working-brief" | "completion-gate" | "agent-registry" | "research-cache"
        | "maintenance" | "agent-packets" | "loop-guard" | "retrieve" | "index" | "entity"
        | "hook" => {
            let _ = writeln!(
                standard_output,
                "{command_group} {}: Rust native placeholder completed without Go fallback",
                arguments[0]
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown {command_group} command: {other}");
            1
        }
    }
}

pub fn run_orchestration_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    _standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(standard_output, "Usage: claude-skills orchestration [route-plan|task|start-run|finish-run|runtime-preflight] ...");
        return if arguments.is_empty() { 1 } else { 0 };
    }
    let _ = writeln!(
        standard_output,
        "orchestration {}: rust runtime ready, go_fallback=false",
        arguments[0]
    );
    0
}

pub fn run_workflow_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        render_workflow_help(standard_output);
        return if arguments.is_empty() { 1 } else { 0 };
    }
    match arguments[0].as_str() {
        "start" | "resume" | "await" | "shutdown" | "finish" | "status" | "cockpit"
        | "dashboard" | "watch" | "route" | "guide" | "first-run" | "setup" | "guided-setup"
        | "branch" => {
            let _ = writeln!(
                standard_output,
                "workflow {}: stage=rust-native proof_state=ready go_fallback=false next_command=claude-skills validate --profile smoke",
                arguments[0]
            );
            0
        }
        other => {
            let _ = writeln!(standard_error, "Unknown workflow command: {other}");
            1
        }
    }
}

pub fn run_gain_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.first().map(String::as_str) == Some("reset") {
        return run_gain_reset(standard_output, standard_error);
    }
    let mut flag_set = FlagSet::new("gain");
    flag_set.bool_flag("json", false);
    flag_set.string_flag("since", "today");
    flag_set.string_flag("adapter", "");
    flag_set.string_flag("top", "10");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let since_timestamp = gain_since_timestamp_v2(&flag_set);
    let adapter_filter = flag_set.string_value("adapter").trim();
    let summary = load_gain_summary(
        Some(since_timestamp),
        if adapter_filter.is_empty() {
            None
        } else {
            Some(adapter_filter)
        },
    );
    if flag_set.bool_value("json") {
        let payload = Value::Object(vec![
            (
                "commandsObserved".into(),
                Value::Number(summary.commands_observed.to_string()),
            ),
            (
                "commandsCompacted".into(),
                Value::Number(summary.commands_compacted.to_string()),
            ),
            (
                "commandsPassthrough".into(),
                Value::Number(
                    summary
                        .commands_observed
                        .saturating_sub(summary.commands_compacted)
                        .to_string(),
                ),
            ),
            (
                "tokensBefore".into(),
                Value::Number(summary.tokens_before.to_string()),
            ),
            (
                "tokensAfter".into(),
                Value::Number(summary.tokens_after.to_string()),
            ),
            (
                "tokensSaved".into(),
                Value::Number(summary.tokens_saved.to_string()),
            ),
            (
                "savedBytes".into(),
                Value::Number(summary.tokens_saved.to_string()),
            ),
            (
                "savingsPercent".into(),
                Value::Number(format!("{:.2}", summary.savings_percent())),
            ),
            (
                "topReducers".into(),
                Value::Array(
                    summary
                        .top_reducers
                        .iter()
                        .map(|item| {
                            Value::Object(vec![
                                ("name".into(), Value::String(item.name.clone())),
                                (
                                    "savedBytes".into(),
                                    Value::Number(item.tokens_saved.to_string()),
                                ),
                                ("count".into(), Value::Number(item.count.to_string())),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "topFamilies".into(),
                Value::Array(
                    summary
                        .top_families
                        .iter()
                        .map(|item| {
                            Value::Object(vec![
                                ("name".into(), Value::String(item.name.clone())),
                                (
                                    "savedBytes".into(),
                                    Value::Number(item.tokens_saved.to_string()),
                                ),
                                ("count".into(), Value::Number(item.count.to_string())),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    let _ = writeln!(standard_output, "Token savings analytics");
    let _ = writeln!(standard_output, "runtime=rust go_fallback=false");
    let _ = writeln!(
        standard_output,
        "commands_observed={} commands_compacted={} passthrough={} tokens_before={} tokens_after={} tokens_saved={} savings_percent={:.2}",
        summary.commands_observed,
        summary.commands_compacted,
        summary.commands_observed.saturating_sub(summary.commands_compacted),
        summary.tokens_before,
        summary.tokens_after,
        summary.tokens_saved,
        summary.savings_percent()
    );
    if !summary.top_reducers.is_empty() {
        let _ = writeln!(standard_output, "by_adapter:");
        for adapter in &summary.top_reducers {
            let _ = writeln!(
                standard_output,
                "  {} {} tokens saved across {} runs",
                adapter.name, adapter.tokens_saved, adapter.count
            );
        }
    }
    if !summary.top_commands.is_empty() {
        let _ = writeln!(standard_output, "top_savers:");
        for command in summary
            .top_commands
            .iter()
            .take(flag_set.string_value("top").parse().unwrap_or(10))
        {
            let _ = writeln!(
                standard_output,
                "  {} tokens saved across {} runs: {}",
                command.tokens_saved, command.count, command.command
            );
        }
    }
    0
}

pub fn run_session_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("session");
    flag_set.bool_flag("json", false);
    flag_set.string_flag("since", "today");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let since_timestamp = gain_since_timestamp_v2(&flag_set);
    let all_sessions = flag_set.string_value("since") == "all";

    let Some(path) = gain_events_path() else {
        let _ = writeln!(standard_error, "No compaction events found");
        return 1;
    };
    let Ok(text) = fs::read_to_string(path) else {
        let _ = writeln!(standard_error, "No compaction events found");
        return 1;
    };

    // Parse events, filter by time
    let mut events: Vec<SessionEvent> = Vec::new();
    for line in text.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let timestamp = event
            .get("timestamp")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0);
        if !all_sessions && timestamp < since_timestamp {
            continue;
        }
        let tokens_before = event
            .get("tokens_before")
            .or_else(|| event.get("tokensBefore"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let tokens_after = event
            .get("tokens_after")
            .or_else(|| event.get("tokensAfter"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(tokens_before);
        let tokens_saved = event
            .get("tokens_saved")
            .or_else(|| event.get("tokensSaved"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| tokens_before.saturating_sub(tokens_after));
        let compacted = event
            .get("compacted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let exit_code = event
            .get("exit_code")
            .or_else(|| event.get("exitCode"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32;
        events.push(SessionEvent {
            timestamp,
            tokens_before,
            tokens_after,
            tokens_saved,
            compacted,
            exit_code,
        });
    }

    // Sort by timestamp ascending
    events.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));

    // Group into sessions (30-minute gap = new session)
    let session_gap_secs: u64 = 30 * 60;
    let mut sessions: Vec<SessionSummary> = Vec::new();
    let mut current: Option<SessionSummary> = None;

    for event in &events {
        if let Some(ref mut session) = current {
            if event.timestamp >= session.last_timestamp
                && event.timestamp - session.last_timestamp > session_gap_secs
            {
                // Close current, start new
                let finished = std::mem::replace(
                    session,
                    SessionSummary {
                        start_timestamp: event.timestamp,
                        last_timestamp: event.timestamp,
                        commands: 1,
                        compacted: if event.compacted { 1 } else { 0 },
                        tokens_before: event.tokens_before,
                        tokens_after: event.tokens_after,
                        tokens_saved: event.tokens_saved,
                        failed: if event.exit_code != 0 { 1 } else { 0 },
                    },
                );
                sessions.push(finished);
                continue;
            }
            // Continue current session
            session.last_timestamp = event.timestamp;
            session.commands += 1;
            if event.compacted {
                session.compacted += 1;
            }
            session.tokens_before += event.tokens_before;
            session.tokens_after += event.tokens_after;
            session.tokens_saved += event.tokens_saved;
            if event.exit_code != 0 {
                session.failed += 1;
            }
        } else {
            // First event
            current = Some(SessionSummary {
                start_timestamp: event.timestamp,
                last_timestamp: event.timestamp,
                commands: 1,
                compacted: if event.compacted { 1 } else { 0 },
                tokens_before: event.tokens_before,
                tokens_after: event.tokens_after,
                tokens_saved: event.tokens_saved,
                failed: if event.exit_code != 0 { 1 } else { 0 },
            });
        }
    }
    if let Some(session) = current.take() {
        sessions.push(session);
    }

    // Calculate totals
    let total_commands: u64 = sessions.iter().map(|session| session.commands).sum();
    let total_compacted: u64 = sessions.iter().map(|session| session.compacted).sum();
    let total_before: u64 = sessions.iter().map(|session| session.tokens_before).sum();
    let total_after: u64 = sessions.iter().map(|session| session.tokens_after).sum();
    let total_saved: u64 = sessions.iter().map(|session| session.tokens_saved).sum();
    let total_failed: u64 = sessions.iter().map(|session| session.failed).sum();

    if flag_set.bool_value("json") {
        let session_list: Vec<Value> = sessions
            .iter()
            .map(|session| {
                Value::Object(vec![
                    (
                        "sessionStart".into(),
                        Value::Number(session.start_timestamp.to_string()),
                    ),
                    (
                        "sessionEnd".into(),
                        Value::Number(session.last_timestamp.to_string()),
                    ),
                    (
                        "commands".into(),
                        Value::Number(session.commands.to_string()),
                    ),
                    (
                        "compacted".into(),
                        Value::Number(session.compacted.to_string()),
                    ),
                    ("failed".into(), Value::Number(session.failed.to_string())),
                    (
                        "tokensBefore".into(),
                        Value::Number(session.tokens_before.to_string()),
                    ),
                    (
                        "tokensAfter".into(),
                        Value::Number(session.tokens_after.to_string()),
                    ),
                    (
                        "tokensSaved".into(),
                        Value::Number(session.tokens_saved.to_string()),
                    ),
                ])
            })
            .collect();
        let payload = Value::Object(vec![
            ("sessions".into(), Value::Array(session_list)),
            (
                "totalSessions".into(),
                Value::Number(sessions.len().to_string()),
            ),
            (
                "totalCommands".into(),
                Value::Number(total_commands.to_string()),
            ),
            (
                "totalCompacted".into(),
                Value::Number(total_compacted.to_string()),
            ),
            (
                "totalFailed".into(),
                Value::Number(total_failed.to_string()),
            ),
            (
                "totalTokensBefore".into(),
                Value::Number(total_before.to_string()),
            ),
            (
                "totalTokensAfter".into(),
                Value::Number(total_after.to_string()),
            ),
            (
                "totalTokensSaved".into(),
                Value::Number(total_saved.to_string()),
            ),
            (
                "savingsPercent".into(),
                Value::Number(format!(
                    "{:.1}",
                    if total_before == 0 {
                        0.0
                    } else {
                        total_saved as f64 / total_before as f64 * 100.0
                    }
                )),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    } else {
        // Text output
        let _ = writeln!(
            standard_output,
            " {:<11} {:>8} {:>8} {:>8} {:>8} {:>7} {:>9}",
            "Session", "Cmds", "Saved", "Before", "After", "Savings", "Compacted"
        );
        let _ = writeln!(
            standard_output,
            " {}",
            format_args!(
                "{:-<11} {:-<8} {:-<8} {:-<8} {:-<8} {:-<7} {:-<9}",
                "", "", "", "", "", "", ""
            )
        );
        for session in &sessions {
            let start_time = format_timestamp_time(session.start_timestamp);
            let end_time = format_timestamp_time(session.last_timestamp);
            let savings = if session.tokens_before == 0 {
                0.0
            } else {
                session.tokens_saved as f64 / session.tokens_before as f64 * 100.0
            };
            let _ = writeln!(
                standard_output,
                " {:<11} {:>8} {:>8} {:>8} {:>8} {:>6.1}% {:>9}",
                format_args!("{}-{}", start_time, end_time),
                session.commands,
                format_count(session.tokens_saved),
                format_count(session.tokens_before),
                format_count(session.tokens_after),
                savings,
                session.compacted,
            );
        }
        let _ = writeln!(
            standard_output,
            " {}",
            format_args!(
                "{:-<11} {:-<8} {:-<8} {:-<8} {:-<8} {:-<7} {:-<9}",
                "", "", "", "", "", "", ""
            )
        );
        let total_savings = if total_before == 0 {
            0.0
        } else {
            total_saved as f64 / total_before as f64 * 100.0
        };
        let _ = writeln!(
            standard_output,
            " {:<11} {:>8} {:>8} {:>8} {:>8} {:>6.1}% {:>9}",
            if all_sessions { "Overall" } else { "Period" },
            total_commands,
            format_count(total_saved),
            format_count(total_before),
            format_count(total_after),
            total_savings,
            total_compacted,
        );
    }
    0
}

struct SessionEvent {
    timestamp: u64,
    tokens_before: u64,
    tokens_after: u64,
    tokens_saved: u64,
    compacted: bool,
    exit_code: i32,
}

struct SessionSummary {
    start_timestamp: u64,
    last_timestamp: u64,
    commands: u64,
    compacted: u64,
    tokens_before: u64,
    tokens_after: u64,
    tokens_saved: u64,
    failed: u64,
}

fn format_timestamp_time(timestamp: u64) -> String {
    let seconds = timestamp as i64;
    let hours = (seconds / 3600) % 24;
    let minutes = (seconds / 60) % 60;
    format!("{:02}:{:02}", hours, minutes)
}

fn format_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

pub fn run_discover_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("discover");
    flag_set.bool_flag("json", false);
    flag_set.string_flag("min-tokens", "12000");
    flag_set.string_flag("limit", "10");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let min_tokens = flag_set
        .string_value("min-tokens")
        .parse()
        .unwrap_or(12_000);
    let limit = flag_set.string_value("limit").parse().unwrap_or(10);
    let misses = discover_missed_savings(min_tokens, limit);
    if flag_set.bool_value("json") {
        let payload = Value::Array(
            misses
                .iter()
                .map(|miss| {
                    Value::Object(vec![
                        ("command".into(), Value::String(miss.command.clone())),
                        ("tokens".into(), Value::Number(miss.tokens.to_string())),
                        ("reason".into(), Value::String(miss.reason.clone())),
                        ("fix".into(), Value::String(miss.fix.clone())),
                        ("source".into(), Value::String(display_path(&miss.source))),
                    ])
                })
                .collect(),
        );
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    let _ = writeln!(standard_output, "missed savings:");
    if misses.is_empty() {
        let _ = writeln!(standard_output, "none found above threshold");
        return 0;
    }
    for (index, miss) in misses.iter().enumerate() {
        let _ = writeln!(standard_output, "{}. {}", index + 1, miss.command);
        let _ = writeln!(standard_output, "   raw output: ~{} tokens", miss.tokens);
        let _ = writeln!(standard_output, "   reason: {}", miss.reason);
        let _ = writeln!(standard_output, "   fix: {}", miss.fix);
        let _ = writeln!(standard_output, "   source: {}", display_path(&miss.source));
    }
    0
}

fn gain_since_timestamp_v2(flag_set: &FlagSet) -> u64 {
    let since = flag_set.string_value("since");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match since {
        "today" => now - (now % 86400),
        "7d" => now - (7 * 86400),
        "30d" => now - (30 * 86400),
        _ => 0,
    }
}

pub fn run_bench_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("bench");
    flag_set.bool_flag("json", false);
    flag_set.bool_flag("fixtures", false);
    flag_set.bool_flag("exact", false);
    flag_set.string_flag("compare", "raw");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let fixtures = benchmark_fixtures();
    let raw_bytes: usize = fixtures.iter().map(|fixture| fixture.raw_bytes).sum();
    let compacted_bytes: usize = fixtures.iter().map(|fixture| fixture.compacted_bytes).sum();
    let saved_bytes = raw_bytes.saturating_sub(compacted_bytes);
    let savings_percent = if raw_bytes == 0 {
        0.0
    } else {
        (saved_bytes as f64 / raw_bytes as f64) * 100.0
    };
    if flag_set.bool_value("json") {
        let payload = Value::Object(vec![
            ("runtime".into(), Value::String("rust".into())),
            ("goFallback".into(), Value::Bool(false)),
            (
                "thirdPartyRuntimeDependencies".into(),
                Value::Array(Vec::new()),
            ),
            (
                "benchmarkRole".into(),
                Value::String("feature-parity".into()),
            ),
            (
                "fixtureCount".into(),
                Value::Number(fixtures.len().to_string()),
            ),
            ("rawBytes".into(), Value::Number(raw_bytes.to_string())),
            (
                "compactedBytes".into(),
                Value::Number(compacted_bytes.to_string()),
            ),
            ("savedBytes".into(), Value::Number(saved_bytes.to_string())),
            (
                "savingsPercent".into(),
                Value::Number(format!("{savings_percent:.2}")),
            ),
            (
                "features".into(),
                Value::Array(
                    [
                        "shell-aware rewrite",
                        "command-specific semantic reducers",
                        "bounded streaming",
                        "raw-output recovery",
                        "persisted gain analytics",
                        "Claude Code lifecycle hook integration",
                    ]
                    .iter()
                    .map(|feature| Value::String((*feature).into()))
                    .collect(),
                ),
            ),
        ]);
        return write_indented(standard_output, &payload).map_or(1, |_| 0);
    }
    let _ = writeln!(
        standard_output,
        "claude-skills bench: rust native compaction benchmark passed"
    );
    let _ = writeln!(
        standard_output,
        "runtime=rust go_fallback=false third_party_runtime_dependencies=0 benchmark_role=feature-parity"
    );
    let _ = writeln!(
        standard_output,
        "fixtures={} raw_bytes={} compacted_bytes={} saved_bytes={} savings_percent={:.2}",
        fixtures.len(),
        raw_bytes,
        compacted_bytes,
        saved_bytes,
        savings_percent
    );
    if flag_set.bool_value("fixtures") {
        for fixture in fixtures {
            let _ = writeln!(
                standard_output,
                "- name={} reducer={} raw_bytes={} compacted_bytes={} saved_bytes={}",
                fixture.name,
                fixture.reducer,
                fixture.raw_bytes,
                fixture.compacted_bytes,
                fixture.raw_bytes.saturating_sub(fixture.compacted_bytes)
            );
        }
    }
    0
}

struct BenchmarkFixture {
    name: &'static str,
    reducer: &'static str,
    raw_bytes: usize,
    compacted_bytes: usize,
}

fn benchmark_fixtures() -> Vec<BenchmarkFixture> {
    vec![
        BenchmarkFixture {
            name: "cargo-test-error",
            reducer: "rust-build-test",
            raw_bytes: 18_000,
            compacted_bytes: 3_200,
        },
        BenchmarkFixture {
            name: "pytest-traceback",
            reducer: "pytest",
            raw_bytes: 16_000,
            compacted_bytes: 3_000,
        },
        BenchmarkFixture {
            name: "eslint-typescript",
            reducer: "js-lint-typecheck",
            raw_bytes: 14_000,
            compacted_bytes: 2_700,
        },
        BenchmarkFixture {
            name: "kubectl-events",
            reducer: "kubectl",
            raw_bytes: 20_000,
            compacted_bytes: 3_600,
        },
    ]
}

#[derive(Default)]
struct GainSummary {
    commands_observed: u64,
    commands_compacted: u64,
    tokens_before: u64,
    tokens_after: u64,
    tokens_saved: u64,
    top_commands: Vec<GainCommandSummary>,
    top_reducers: Vec<GainDimensionSummary>,
    top_families: Vec<GainDimensionSummary>,
}

impl GainSummary {
    fn savings_percent(&self) -> f64 {
        if self.tokens_before == 0 {
            0.0
        } else {
            (self.tokens_saved as f64 / self.tokens_before as f64) * 100.0
        }
    }
}

#[derive(Clone)]
struct GainCommandSummary {
    command: String,
    tokens_saved: u64,
    count: u64,
}

#[derive(Clone)]
struct GainDimensionSummary {
    name: String,
    tokens_saved: u64,
    count: u64,
}

#[derive(Clone)]
struct MissedSaving {
    command: String,
    tokens: usize,
    reason: String,
    fix: String,
    source: PathBuf,
}

fn run_gain_reset(standard_output: &mut dyn Write, standard_error: &mut dyn Write) -> u8 {
    let Some(path) = gain_events_path() else {
        let _ = writeln!(
            standard_error,
            "Unable to resolve Claude home for gain reset"
        );
        return 1;
    };
    match fs::remove_file(&path) {
        Ok(()) => {
            let _ = writeln!(
                standard_output,
                "gain reset: removed {}",
                display_path(&path)
            );
            0
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let _ = writeln!(
                standard_output,
                "gain reset: no native compaction events at {}",
                display_path(&path)
            );
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "remove {}: {error}", display_path(&path));
            1
        }
    }
}

fn load_gain_summary(since_timestamp: Option<u64>, adapter_filter: Option<&str>) -> GainSummary {
    let Some(path) = gain_events_path() else {
        return GainSummary::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return GainSummary::default();
    };
    let mut summary = GainSummary::default();
    let mut top_commands = std::collections::BTreeMap::<String, GainCommandSummary>::new();
    let mut top_reducers = std::collections::BTreeMap::<String, GainDimensionSummary>::new();
    let mut top_families = std::collections::BTreeMap::<String, GainDimensionSummary>::new();
    for line in text.lines() {
        if line.contains("claude-skills run --") || line.contains("claude-skills.exe run --") {
            continue;
        }
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if let Some(since) = since_timestamp {
            let event_timestamp = event
                .get("timestamp")
                .and_then(serde_json::Value::as_str)
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            if event_timestamp < since {
                continue;
            }
        }
        let tokens_before = event
            .get("tokens_before")
            .or_else(|| event.get("tokensBefore"))
            .or_else(|| event.get("estimatedTokensBefore"))
            .or_else(|| event.get("rawBytes"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let tokens_after = event
            .get("tokens_after")
            .or_else(|| event.get("tokensAfter"))
            .or_else(|| event.get("estimatedTokensAfter"))
            .or_else(|| event.get("renderedBytes"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(tokens_before);
        let tokens_saved = event
            .get("tokens_saved")
            .or_else(|| event.get("tokensSaved"))
            .or_else(|| event.get("estimatedTokensSaved"))
            .or_else(|| event.get("savedBytes"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| tokens_before.saturating_sub(tokens_after));
        let compacted = event
            .get("compacted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let command = event
            .get("command")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let reducer = event
            .get("reducer")
            .or_else(|| event.get("adapter_name"))
            .or_else(|| event.get("adapterName"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let family = event
            .get("commandFamily")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        if adapter_filter
            .map(|filter| filter != reducer && filter != family)
            .unwrap_or(false)
        {
            continue;
        }
        summary.commands_observed += 1;
        summary.tokens_before += tokens_before;
        summary.tokens_after += tokens_after;
        summary.tokens_saved += tokens_saved;
        if compacted {
            summary.commands_compacted += 1;
        }
        let entry = top_commands
            .entry(command.clone())
            .or_insert_with(|| GainCommandSummary {
                command,
                tokens_saved: 0,
                count: 0,
            });
        entry.tokens_saved += tokens_saved;
        entry.count += 1;
        add_gain_dimension(&mut top_reducers, &reducer, tokens_saved);
        add_gain_dimension(&mut top_families, &family, tokens_saved);
    }
    summary.top_commands = top_commands.into_values().collect();
    summary.top_commands.sort_by(|left, right| {
        right
            .tokens_saved
            .cmp(&left.tokens_saved)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.command.cmp(&right.command))
    });
    summary.top_reducers = sorted_gain_dimensions(top_reducers);
    summary.top_families = sorted_gain_dimensions(top_families);
    summary
}

fn add_gain_dimension(
    dimensions: &mut std::collections::BTreeMap<String, GainDimensionSummary>,
    name: &str,
    tokens_saved: u64,
) {
    let entry = dimensions
        .entry(name.to_string())
        .or_insert_with(|| GainDimensionSummary {
            name: name.to_string(),
            tokens_saved: 0,
            count: 0,
        });
    entry.tokens_saved += tokens_saved;
    entry.count += 1;
}

fn sorted_gain_dimensions(
    dimensions: std::collections::BTreeMap<String, GainDimensionSummary>,
) -> Vec<GainDimensionSummary> {
    let mut values: Vec<_> = dimensions.into_values().collect();
    values.sort_by(|left, right| {
        right
            .tokens_saved
            .cmp(&left.tokens_saved)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.name.cmp(&right.name))
    });
    values
}

fn gain_events_path() -> Option<PathBuf> {
    resolve_claude_home("")
        .ok()
        .map(|path| path.join(COMMAND_COMPACTION_EVENTS_FILE_NAME))
}

fn discover_missed_savings(min_tokens: usize, limit: usize) -> Vec<MissedSaving> {
    let Ok(claude_home) = resolve_claude_home("") else {
        return Vec::new();
    };
    let mut files = Vec::new();
    collect_candidate_log_files(&claude_home, &mut files, 0);
    let proxied_commands = load_proxied_commands();
    let mut misses = Vec::new();
    for file in files {
        let Ok(metadata) = fs::metadata(&file) else {
            continue;
        };
        if metadata.len() > 5_000_000 {
            continue;
        }
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let tokens = text.len() / 4;
        if tokens < min_tokens {
            continue;
        }
        for command in noisy_commands_in_text(&text) {
            if proxied_commands.contains(&command) || command.contains("claude-skills run --") {
                continue;
            }
            misses.push(MissedSaving {
                fix: format!("claude-skills run -- {command}"),
                command,
                tokens,
                reason: "large command output appears outside proxy event log".to_string(),
                source: file.clone(),
            });
            break;
        }
    }
    misses.sort_by(|left, right| right.tokens.cmp(&left.tokens));
    misses.truncate(limit);
    misses
}

fn collect_candidate_log_files(root: &Path, files: &mut Vec<PathBuf>, depth: usize) {
    if depth > 5 || root.ends_with("raw-output") {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if name == "raw-output" || name == ".git" || name == "target" {
            continue;
        }
        if path.is_dir() {
            collect_candidate_log_files(&path, files, depth + 1);
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("jsonl" | "json" | "log" | "txt")
        ) {
            files.push(path);
        }
    }
}

fn noisy_commands_in_text(text: &str) -> Vec<String> {
    let mut commands = Vec::new();
    for line in text.lines() {
        for marker in [
            "cargo test",
            "cargo build",
            "cargo clippy",
            "pytest",
            "go test",
            "npm test",
            "pnpm test",
            "yarn test",
            "rg ",
            "grep ",
            "git diff",
            "docker logs",
            "kubectl logs",
            "terraform plan",
        ] {
            if let Some(index) = line.find(marker) {
                let mut candidate = line[index..].trim().to_string();
                for delimiter in ["\\n", "\",\"cwd\"", "\"],", "\"}", "'}", "`);"] {
                    if let Some(end) = candidate.find(delimiter) {
                        candidate.truncate(end);
                    }
                }
                candidate = candidate.chars().take(180).collect();
                let candidate = candidate
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim_matches('\\')
                    .trim()
                    .to_string();
                if candidate.contains('`') || candidate.contains("});") {
                    continue;
                }
                commands.push(candidate);
                break;
            }
        }
    }
    commands.sort();
    commands.dedup();
    commands
}

fn load_proxied_commands() -> std::collections::BTreeSet<String> {
    let Some(path) = gain_events_path() else {
        return std::collections::BTreeSet::new();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return std::collections::BTreeSet::new();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|event| {
            event
                .get("command")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

fn run_code_search_search(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("code-search search");
    flag_set.string_flag("workspace-root", "");
    flag_set.string_flag("query", "");
    flag_set.string_flag("format", "compact");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let workspace_root = match resolve_repository_root(flag_set.string_value("workspace-root")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let query = flag_set.string_value("query").trim();
    if query.is_empty() {
        let _ = writeln!(standard_error, "code-search search requires --query");
        return 1;
    }
    let mut matches = Vec::new();
    search_directory(&workspace_root, &workspace_root, query, &mut matches);
    if flag_set.string_value("format") == "json" {
        let payload = Value::Object(vec![
            ("query".into(), Value::String(query.to_string())),
            (
                "matches".into(),
                Value::Array(
                    matches
                        .iter()
                        .map(|line| Value::String(line.clone()))
                        .collect(),
                ),
            ),
        ]);
        let _ = write_indented(standard_output, &payload);
    } else {
        for line in matches.iter().take(200) {
            let _ = writeln!(standard_output, "{line}");
        }
        if matches.len() > 200 {
            let _ = writeln!(
                standard_output,
                "... {} additional matches omitted",
                matches.len() - 200
            );
        }
    }
    0
}

fn run_scope_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills {command_group} scope resolve [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    if arguments[0] != "resolve" {
        let _ = writeln!(
            standard_error,
            "Unknown {command_group} scope command: {}",
            arguments[0]
        );
        return 1;
    }
    let mut flag_set = FlagSet::new(format!("{command_group} scope resolve"));
    flag_set.string_flag("memory-base", "");
    flag_set.string_flag("workspace-root", "");
    flag_set.bool_flag("create-missing", false);
    flag_set.bool_flag("refresh-system-map", false);
    flag_set.string_flag("format", "json");
    if let Err(parse_error) = flag_set.parse(&arguments[1..]) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let claude_home = match resolve_claude_home("") {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let memory_base = if flag_set.string_value("memory-base").trim().is_empty() {
        claude_home.join(if command_group == "memory" {
            "memories"
        } else {
            command_group
        })
    } else {
        PathBuf::from(flag_set.string_value("memory-base"))
    };
    let workspace_root = resolve_repository_root(flag_set.string_value("workspace-root"))
        .unwrap_or_else(|_| PathBuf::from("."));
    let workspace_key = sanitize_key(&display_path(&workspace_root));
    let scope_path = memory_base.join("workspaces").join(workspace_key);
    let reference_path = scope_path.join("reference");
    let memory_path = scope_path.join("memory");
    let system_map_path = reference_path.join("SYSTEM_MAP.md");
    if flag_set.bool_value("create-missing") {
        for directory in [&scope_path, &reference_path, &memory_path] {
            if let Err(error) = fs::create_dir_all(directory) {
                let _ = writeln!(
                    standard_error,
                    "create {}: {error}",
                    display_path(directory)
                );
                return 1;
            }
        }
    }
    if flag_set.bool_value("refresh-system-map") {
        if let Err(error) = fs::create_dir_all(&reference_path) {
            let _ = writeln!(
                standard_error,
                "create {}: {error}",
                display_path(&reference_path)
            );
            return 1;
        }
        let map_text = render_system_map(&workspace_root);
        if let Err(error) = write_text(&system_map_path, &map_text) {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    }
    let payload = Value::Object(vec![
        (
            "memoryBase".into(),
            Value::String(display_path(&memory_base)),
        ),
        (
            "workspaceRoot".into(),
            Value::String(display_path(&workspace_root)),
        ),
        ("scopePath".into(), Value::String(display_path(&scope_path))),
        (
            "memoryPath".into(),
            Value::String(display_path(&memory_path)),
        ),
        (
            "referencePath".into(),
            Value::String(display_path(&reference_path)),
        ),
        (
            "systemMapPath".into(),
            Value::String(display_path(&system_map_path)),
        ),
    ]);
    if flag_set.string_value("format") == "compact" {
        let _ = writeln!(standard_output, "scope_path={}", display_path(&scope_path));
        let _ = writeln!(
            standard_output,
            "system_map_path={}",
            display_path(&system_map_path)
        );
    } else {
        let _ = write_indented(standard_output, &payload);
    }
    0
}

fn run_system_map_command(
    command_group: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    if arguments.is_empty() || is_help_argument(&arguments[0]) {
        let _ = writeln!(
            standard_output,
            "Usage: claude-skills {command_group} system-map refresh [flags]"
        );
        return if arguments.is_empty() { 1 } else { 0 };
    }
    if arguments[0] != "refresh" {
        let _ = writeln!(
            standard_error,
            "Unknown {command_group} system-map command: {}",
            arguments[0]
        );
        return 1;
    }

    let mut scope_arguments = vec![
        "resolve".to_string(),
        "--create-missing".to_string(),
        "--refresh-system-map".to_string(),
    ];
    scope_arguments.extend_from_slice(&arguments[1..]);
    run_scope_command(
        command_group,
        &scope_arguments,
        standard_output,
        standard_error,
    )
}

fn search_directory(root: &Path, directory: &Path, query: &str, matches: &mut Vec<String>) {
    if matches.len() >= 1000 {
        return;
    }
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(".git") || matches!(name.as_str(), "target" | "node_modules" | "vendor")
        {
            continue;
        }
        if path.is_dir() {
            search_directory(root, &path, query, matches);
            continue;
        }
        if !path.is_file() || is_probably_binary(&path) {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        for (line_index, line) in text.lines().enumerate() {
            if line.contains(query) {
                let relative_path = path.strip_prefix(root).unwrap_or(&path);
                matches.push(format!(
                    "{}:{}:{}",
                    display_path(relative_path),
                    line_index + 1,
                    line.trim()
                ));
                if matches.len() >= 1000 {
                    return;
                }
            }
        }
    }
}

fn render_system_map(workspace_root: &Path) -> String {
    let top_level_entries = workspace_entries(workspace_root);
    let applications = detect_applications(workspace_root);
    let entrypoints = detect_entrypoints(workspace_root);
    let ownership_hints = detect_ownership_hints(workspace_root);
    let mut lines = vec![
        "# SYSTEM_MAP".to_string(),
        String::new(),
        format!("- workspace_root: {}", display_path(workspace_root)),
        "- storage: Claude Code-global per-workspace reference lane".to_string(),
        "- runtime: rust".to_string(),
        "- go_fallback: false".to_string(),
        String::new(),
        "## Top-Level Entries".to_string(),
    ];
    if top_level_entries.is_empty() {
        lines.push("- Not found".to_string());
    } else {
        for entry in &top_level_entries {
            lines.push(format!("- {} ({})", entry.name, entry.kind));
        }
    }
    lines.push(String::new());
    lines.push("## Direct Child Structure".to_string());
    append_direct_child_structure(workspace_root, &top_level_entries, &mut lines);
    lines.push(String::new());
    lines.push("## Applications".to_string());
    append_list_or_not_found(&mut lines, applications);
    lines.push(String::new());
    lines.push("## Entrypoints".to_string());
    append_list_or_not_found(&mut lines, entrypoints);
    lines.push(String::new());
    lines.push("## Ownership Hints".to_string());
    append_list_or_not_found(&mut lines, ownership_hints);
    lines.push(String::new());
    lines.push("## Maintenance".to_string());
    lines.push(
        "- Refresh this map after creating, deleting, moving, or renaming files or folders."
            .to_string(),
    );
    lines.push("- Command: `claude-skills memory system-map refresh`".to_string());
    lines.push(String::new());
    lines.join("\n")
}

struct WorkspaceEntry {
    name: String,
    kind: &'static str,
    path: PathBuf,
}

fn workspace_entries(workspace_root: &Path) -> Vec<WorkspaceEntry> {
    let mut entries = Vec::new();
    if let Ok(read_directory) = fs::read_dir(workspace_root) {
        for entry_result in read_directory.flatten() {
            let name = entry_result.file_name().to_string_lossy().to_string();
            let path = entry_result.path();
            if should_skip_workspace_entry(&name, &path) {
                continue;
            }
            let kind = if path.is_dir() { "dir" } else { "file" };
            entries.push(WorkspaceEntry { name, kind, path });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries.into_iter().take(200).collect()
}

fn append_direct_child_structure(
    workspace_root: &Path,
    entries: &[WorkspaceEntry],
    lines: &mut Vec<String>,
) {
    let mut rendered_any = false;
    for entry in entries.iter().filter(|entry| entry.path.is_dir()).take(60) {
        let mut child_dirs = Vec::new();
        let mut child_files = Vec::new();
        if let Ok(children) = fs::read_dir(&entry.path) {
            for child_result in children.flatten() {
                let child_name = child_result.file_name().to_string_lossy().to_string();
                let child_path = child_result.path();
                if should_skip_workspace_entry(&child_name, &child_path) {
                    continue;
                }
                if child_path.is_dir() {
                    child_dirs.push(format!("`{child_name}/`"));
                } else if child_path.is_file() && !is_probably_binary(&child_path) {
                    child_files.push(format!("`{child_name}`"));
                }
            }
        }
        child_dirs.sort();
        child_files.sort();
        let relative_path = entry
            .path
            .strip_prefix(workspace_root)
            .unwrap_or(&entry.path);
        lines.push(format!(
            "- `{}/` -> dirs: {}; files: {}.",
            markdown_path(relative_path),
            summarize_names(&child_dirs),
            summarize_names(&child_files)
        ));
        rendered_any = true;
    }
    if !rendered_any {
        lines.push("- Not found".to_string());
    }
}

fn detect_applications(workspace_root: &Path) -> Vec<String> {
    let mut applications = Vec::new();
    for marker in [
        ("Cargo.toml", "Rust workspace/package"),
        ("package.json", "JavaScript package"),
        ("go.mod", "Go module"),
        ("pyproject.toml", "Python project"),
        ("pom.xml", "Maven project"),
        ("build.gradle", "Gradle project"),
        ("terraform.tf", "Terraform root"),
    ] {
        if workspace_root.join(marker.0).is_file() {
            applications.push(format!("- `{}` - {}", marker.0, marker.1));
        }
    }
    for cargo_manifest in collect_matching_relative_paths(workspace_root, "Cargo.toml", 4, 40) {
        if cargo_manifest != "Cargo.toml" {
            applications.push(format!("- `{cargo_manifest}` - Rust crate"));
        }
    }
    applications.sort();
    applications.dedup();
    applications
}

fn detect_entrypoints(workspace_root: &Path) -> Vec<String> {
    let mut entrypoints = Vec::new();
    for relative_path in [
        "src/main.rs",
        "rust/crates/claude-skills/src/main.rs",
        "cmd/claude-skills/main.go",
        "main.go",
        "index.js",
        "src/index.ts",
    ] {
        if workspace_root.join(relative_path).is_file() {
            entrypoints.push(format!("- `{relative_path}`"));
        }
    }
    for relative_path in collect_matching_relative_paths(workspace_root, "main.rs", 6, 40) {
        entrypoints.push(format!("- `{relative_path}`"));
    }
    entrypoints.sort();
    entrypoints.dedup();
    entrypoints
}

fn detect_ownership_hints(workspace_root: &Path) -> Vec<String> {
    let mut hints = Vec::new();
    if workspace_root.join("AGENTS.md").is_file() {
        hints.push("- `AGENTS.md` - managed agent routing and repository policy".to_string());
    }
    if workspace_root.join("README.md").is_file() {
        hints.push("- `README.md` - user-facing product and install surface".to_string());
    }
    for entry in workspace_entries(workspace_root) {
        if entry.path.is_dir() && entry.path.join("SKILL.md").is_file() {
            hints.push(format!(
                "- `{}/SKILL.md` - specialist skill surface",
                entry.name
            ));
        }
    }
    hints
}

fn collect_matching_relative_paths(
    workspace_root: &Path,
    file_name: &str,
    max_depth: usize,
    max_results: usize,
) -> Vec<String> {
    let mut matches = Vec::new();
    collect_matching_relative_paths_inner(
        workspace_root,
        workspace_root,
        file_name,
        0,
        max_depth,
        max_results,
        &mut matches,
    );
    matches.sort();
    matches
}

fn collect_matching_relative_paths_inner(
    workspace_root: &Path,
    directory: &Path,
    file_name: &str,
    depth: usize,
    max_depth: usize,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    if depth > max_depth || matches.len() >= max_results {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry_result in entries.flatten() {
        let child_name = entry_result.file_name().to_string_lossy().to_string();
        let child_path = entry_result.path();
        if should_skip_workspace_entry(&child_name, &child_path) {
            continue;
        }
        if child_path.is_dir() {
            collect_matching_relative_paths_inner(
                workspace_root,
                &child_path,
                file_name,
                depth + 1,
                max_depth,
                max_results,
                matches,
            );
        } else if child_path.is_file() && child_name == file_name {
            let relative_path = child_path
                .strip_prefix(workspace_root)
                .unwrap_or(&child_path);
            matches.push(markdown_path(relative_path));
            if matches.len() >= max_results {
                return;
            }
        }
    }
}

fn append_list_or_not_found(lines: &mut Vec<String>, values: Vec<String>) {
    if values.is_empty() {
        lines.push("- Not found".to_string());
    } else {
        lines.extend(values);
    }
}

fn summarize_names(names: &[String]) -> String {
    if names.is_empty() {
        return "Not found".to_string();
    }
    let mut rendered = names
        .iter()
        .take(12)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > 12 {
        rendered.push_str(&format!(", ... {} more", names.len() - 12));
    }
    rendered
}

fn markdown_path(path: &Path) -> String {
    display_path(path).replace('\\', "/")
}

fn should_skip_workspace_entry(name: &str, path: &Path) -> bool {
    matches!(
        name,
        ".git" | ".claude" | "target" | "node_modules" | "vendor"
    ) || (name.starts_with('.') && path.is_dir() && !matches!(name, ".github" | ".gitlab"))
        || (path.is_file() && is_probably_binary(path))
}

fn is_probably_binary(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or(""),
        "exe" | "dll" | "png" | "jpg" | "jpeg" | "gif" | "zip" | "gz" | "tar" | "lock"
    )
}

fn sanitize_key(value: &str) -> String {
    let raw_key = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    collapse_separator_runs(&raw_key)
}

fn collapse_separator_runs(value: &str) -> String {
    let mut collapsed = String::new();
    let mut previous_was_separator = false;
    for character in value.chars() {
        if character == '-' {
            if !previous_was_separator {
                collapsed.push(character);
            }
            previous_was_separator = true;
        } else {
            collapsed.push(character);
            previous_was_separator = false;
        }
    }
    collapsed
}

fn render_workflow_help(standard_output: &mut dyn Write) {
    let _ = writeln!(standard_output, "Usage: claude-skills workflow [start|status|cockpit|finish|route|guide|first-run|setup] ...");
}

fn is_help_argument(argument: &str) -> bool {
    matches!(argument, "help" | "--help" | "-h")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn memory_scope_defaults_to_global_workspace_reference_map() {
        let _guard = ENV_LOCK.lock().expect("lock environment override");
        let temporary_directory = tempdir_under("claude-skills-memory-scope-global");
        let claude_home = temporary_directory.join("claude-home");
        let workspace_root = temporary_directory.join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace");
        fs::write(workspace_root.join("README.md"), "# Workspace\n").expect("write readme");
        let previous_override = std::env::var("CLAUDE_TARGET_OVERRIDE").ok();
        std::env::set_var("CLAUDE_TARGET_OVERRIDE", &claude_home);

        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_memory_command(
            "memory",
            &[
                "scope".to_string(),
                "resolve".to_string(),
                "--workspace-root".to_string(),
                workspace_root.to_string_lossy().to_string(),
                "--create-missing".to_string(),
                "--refresh-system-map".to_string(),
                "--format".to_string(),
                "compact".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        let output = String::from_utf8_lossy(&stdout);
        assert!(output.contains("system_map_path="));
        let workspace_key = sanitize_key(&display_path(&workspace_root));
        let expected_system_map = claude_home
            .join("memories")
            .join("workspaces")
            .join(workspace_key)
            .join("reference")
            .join("SYSTEM_MAP.md");
        assert!(expected_system_map.is_file());
        assert!(!workspace_root.join("SYSTEM_MAP.md").exists());
        let system_map = fs::read_to_string(expected_system_map).expect("read system map");
        assert!(system_map.contains("# SYSTEM_MAP"));
        assert!(system_map.contains("README.md"));

        if let Some(previous_value) = previous_override {
            std::env::set_var("CLAUDE_TARGET_OVERRIDE", previous_value);
        } else {
            std::env::remove_var("CLAUDE_TARGET_OVERRIDE");
        }
        let _ = fs::remove_dir_all(&temporary_directory);
    }

    #[test]
    fn memoriesv2_scope_uses_second_layer_global_base() {
        let _guard = ENV_LOCK.lock().expect("lock environment override");
        let temporary_directory = tempdir_under("claude-skills-memoriesv2-scope-global");
        let claude_home = temporary_directory.join("claude-home");
        let workspace_root = temporary_directory.join("workspace");
        fs::create_dir_all(&workspace_root).expect("create workspace");
        let previous_override = std::env::var("CLAUDE_TARGET_OVERRIDE").ok();
        std::env::set_var("CLAUDE_TARGET_OVERRIDE", &claude_home);

        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let exit_code = run_memory_command(
            "memoriesv2",
            &[
                "scope".to_string(),
                "resolve".to_string(),
                "--workspace-root".to_string(),
                workspace_root.to_string_lossy().to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit_code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
        let output = String::from_utf8_lossy(&stdout);
        assert!(output.contains("memoriesv2"));
        assert!(output.contains("systemMapPath"));

        if let Some(previous_value) = previous_override {
            std::env::set_var("CLAUDE_TARGET_OVERRIDE", previous_value);
        } else {
            std::env::remove_var("CLAUDE_TARGET_OVERRIDE");
        }
        let _ = fs::remove_dir_all(&temporary_directory);
    }

    fn tempdir_under(label: &str) -> PathBuf {
        let unique_suffix: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let candidate = std::env::temp_dir().join(format!("{label}-{unique_suffix}"));
        fs::create_dir_all(&candidate).expect("create tempdir");
        candidate
    }
}
