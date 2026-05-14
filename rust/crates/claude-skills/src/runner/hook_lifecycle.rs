//! Purpose: Claude Code hook lifecycle management, installation, and removal.
//! Caller: runner/mod.rs for hook command group.
//! Dependencies: std::collections::BTreeMap, std::fs, std::path, serde_json, crate::runtime.
//! Main Functions: run_hook_command, build_hooks_payload, remove_managed_hooks.
//! Side Effects: Reads and writes Claude Code hooks.json configuration.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Map as JsonMap, Value as JsonDocument};

use crate::args::FlagSet;
use crate::json::{write_indented, Value};
use crate::runner::shell_rewrite::{
    bash_command_for_executable_args, platform_default_command_for_executable_args,
    rewrite_command_text_for_shell, RewriteShell,
};
use crate::runtime::{display_path, resolve_claude_home, write_text};
use crate::utility;

const CLAUDE_HOOK_EVENTS: &[&str] = crate::hooks::claude::EVENTS;
const MANAGED_PRE_TOOL_USE_EVENT: &str = "PreToolUse";
const MANAGED_PRE_TOOL_USE_MATCHER: &str = crate::hooks::claude::pre_tool_matcher();
const MANAGED_PRE_TOOL_USE_COMMAND_SUFFIX: &str = "hook pre-tool-use";
const MANAGED_PRE_TOOL_USE_STATUS: &str =
    "Transparently rewriting noisy commands via claude-skills run";
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

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
            run_hook_lifecycle(arguments[0].as_str(), standard_output, standard_error)
        }

        other => {
            let _ = writeln!(standard_error, "Unknown hook command: {other}");

            render_hook_help(standard_output);

            1
        }
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

fn run_hook_lifecycle(
    subcommand: &str,

    standard_output: &mut dyn Write,

    standard_error: &mut dyn Write,
) -> u8 {
    let event_name = hook_event_name_from_subcommand(subcommand);

    if event_name == "SessionStart" {
        let _ = refresh_memory_scope_for_current_directory(standard_error);
    }

    let context = lifecycle_additional_context(subcommand);

    if context.trim().is_empty() {
        return 0;
    }

    // Only PreToolUse, UserPromptSubmit, PostToolUse, and PostToolBatch

    // support hookSpecificOutput in Claude Code's hook schema. Other events

    // (Stop, SessionStart, etc.) must use top-level fields only.

    let supports_hook_specific = matches!(
        event_name,
        "PreToolUse" | "UserPromptSubmit" | "PostToolUse" | "PostToolBatch"
    );

    let payload = if supports_hook_specific {
        serde_json::json!({

            "hookSpecificOutput": {

                "hookEventName": event_name,

                "additionalContext": context,

            },

            "suppressOutput": true,

        })
    } else {
        serde_json::json!({

            "systemMessage": context,

            "suppressOutput": true,

        })
    };

    match serde_json::to_string_pretty(&payload) {
        Ok(rendered) => {
            let _ = writeln!(standard_output, "{rendered}");

            0
        }

        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Unable to render Claude Code lifecycle hook output: {error}"
            );

            1
        }
    }
}

fn hook_event_name_from_subcommand(subcommand: &str) -> &'static str {
    match subcommand {
        "post-tool-use" => "PostToolUse",

        "permission-request" => "PermissionRequest",

        "notification" => "Notification",

        "user-prompt-submit" => "UserPromptSubmit",

        "stop" => "Stop",

        "subagent-stop" => "SubagentStop",

        "pre-compact" => "PreCompact",

        "post-compact" => "PostCompact",

        "session-start" => "SessionStart",

        "session-end" => "SessionEnd",

        _ => "SessionStart",
    }
}

fn lifecycle_additional_context(subcommand: &str) -> String {
    match subcommand {
        "session-start" => session_start_context(),

        "user-prompt-submit" => user_prompt_submit_context(),

        "post-tool-use" => post_tool_use_context(),

        "pre-compact" => pre_compact_context(),

        "post-compact" => post_compact_context(),

        "stop" | "subagent-stop" | "session-end" => closeout_context(),

        _ => String::new(),
    }
}

fn session_start_context() -> String {
    let scope = memory_scope_summary();

    format!(
        "claude-skills automatic operating contract is active.\n\n\
        Skills: route domain work through installed skills in ~/.claude/skills; for existing-source edits, apply preserve-existing-flow before patching and reviewer before closeout.\n\
        Workflow: start substantial work with claude-skills workflow route/start, keep cockpit/proof state current, and finish only after validation evidence is present.\n\
        Memory: read the workspace system map and relevant memory before recommendations; save durable user/project/reference/feedback facts when they will matter later; avoid saving ephemeral task state.\n\
        Review: before final completion, run claude-skills review pre-pr and claude-skills review gates check when code or delivery artifacts changed.\n\
        Commands: noisy Bash commands are automatically rewritten through claude-skills run --; use direct tools for file reads/searches when they are better.\n\n{scope}"
    )
}

fn user_prompt_submit_context() -> String {
    format!(
        "Before answering this prompt, apply claude-skills automatically.\n\
        1. Route specialist work: use the relevant installed skill guidance; preserve-existing-flow is mandatory before existing-source edits; reviewer is mandatory before closing work.\n\
        2. Memory first: consult the workspace system map and relevant memories before acting on repo structure or user/project preferences; save durable facts learned from the user.\n\
        3. Workflow proof: for non-trivial work, keep workflow/proof state current and run the narrowest useful validation.\n\
        4. Review closeout: do not present code work as complete until review/pre-pr gates and validation evidence are current.\n\n{}",
        memory_scope_summary()
    )
}

fn post_tool_use_context() -> String {
    "After each tool result, update claude-skills proof state mentally: if files changed, preserve-existing-flow evidence and review gates must still be satisfied before closeout; if the result introduced durable user/project/reference/feedback knowledge, save it to memory.".to_string()
}

fn pre_compact_context() -> String {
    "Before compaction, preserve claude-skills continuity: summarize active workflow stage, files changed, validation evidence, unresolved blockers, memory facts to save, and next review gate.".to_string()
}

fn post_compact_context() -> String {
    format!(

        "After compaction, resume using claude-skills automatically: reload workspace memory/system map, re-establish workflow proof state, and run review gates before final closeout.\n\n{}",

        memory_scope_summary()

    )
}

fn closeout_context() -> String {
    "Before final response or session close, enforce claude-skills closeout: complete or leave pending task tracking, save durable memory facts, run reviewer/pre-pr gates for changed code, report validation evidence, and do not claim done with in-progress work or failing checks.".to_string()
}

fn refresh_memory_scope_for_current_directory(standard_error: &mut dyn Write) -> Option<PathBuf> {
    let workspace_root = std::env::current_dir().ok()?;

    let mut stdout = Vec::new();

    let mut stderr = Vec::new();

    let arguments = vec![
        "scope".to_string(),
        "resolve".to_string(),
        "--workspace-root".to_string(),
        display_path(&workspace_root),
        "--create-missing".to_string(),
        "--refresh-system-map".to_string(),
        "--format".to_string(),
        "compact".to_string(),
    ];

    let code = utility::run_memory_command("memory", &arguments, &mut stdout, &mut stderr);

    if code != 0 {
        let _ = writeln!(
            standard_error,
            "claude-skills lifecycle memory refresh failed: {}",
            String::from_utf8_lossy(&stderr).trim()
        );

        return None;
    }

    memory_system_map_path_for_workspace(&workspace_root)
}

fn memory_scope_summary() -> String {
    match std::env::current_dir()

        .ok()

        .and_then(|workspace_root| memory_system_map_path_for_workspace(&workspace_root))

    {

        Some(path) => format!(

            "Workspace memory system map: {}. Read it before making repo-structure claims; refresh happened automatically at session start when possible.",

            display_path(&path)

        ),

        None => "Workspace memory system map: unavailable; create it with claude-skills memory scope resolve --create-missing --refresh-system-map before repo-structure claims.".to_string(),

    }
}

fn memory_system_map_path_for_workspace(workspace_root: &Path) -> Option<PathBuf> {
    let claude_home = resolve_claude_home("").ok()?;

    let workspace_key = sanitize_memory_key(&display_path(workspace_root));

    Some(
        claude_home
            .join("memories")
            .join("workspaces")
            .join(workspace_key)
            .join("reference")
            .join("SYSTEM_MAP.md"),
    )
}

fn sanitize_memory_key(value: &str) -> String {
    let mut key = String::new();

    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            key.push(character.to_ascii_lowercase());

            previous_dash = false;
        } else if !previous_dash {
            key.push('-');

            previous_dash = true;
        }
    }

    let trimmed = key.trim_matches('-').to_string();

    if trimmed.is_empty() {
        "workspace".to_string()
    } else {
        trimmed
    }
}

fn render_hook_help(standard_output: &mut dyn Write) {
    let _ = writeln!(

        standard_output,

        "Usage: claude-skills hook [install|uninstall|list|show|instructions|pre-tool-use|post-tool-use|post-tool-use-failure|permission-request|notification|user-prompt-submit|stop|subagent-stop|task-created|task-completed|pre-compact|post-compact|session-start|session-end]"

    );
}

fn is_help_argument(argument: &str) -> bool {
    matches!(argument, "help" | "--help" | "-h")
}

pub fn managed_hook_command() -> Result<String, String> {
    std::env::current_exe()
        .map(|path| hook_command_for_executable_args(&path, MANAGED_PRE_TOOL_USE_COMMAND_SUFFIX))
        .map_err(|error| format!("resolve current executable: {error}"))
}

pub fn build_hooks_payload(hook_path: &Path, hook_command: &str) -> Result<String, String> {
    let mut document = read_hooks_document(hook_path)?;

    ensure_hooks_object(&mut document)?;

    remove_managed_hooks(&mut document);

    append_managed_hooks(&mut document, hook_command)?;

    serde_json::to_string_pretty(&document)
        .map(|rendered| format!("{rendered}\n"))
        .map_err(|error| format!("render hooks config: {error}"))
}

pub fn remove_managed_hook_payload(hook_path: &Path) -> Result<(String, bool), String> {
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

pub fn read_hooks_document(hook_path: &Path) -> Result<JsonDocument, String> {
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

pub fn remove_managed_hooks(document: &mut JsonDocument) {
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

pub fn is_managed_hook_command(command: &str) -> bool {
    is_managed_hook_command_with_depth(command, 0)
}

fn is_managed_hook_command_with_depth(command: &str, depth: usize) -> bool {
    const MAX_DECODE_DEPTH: usize = 2;

    let normalized = command.to_ascii_lowercase();

    let has_any_lifecycle = CLAUDE_HOOK_EVENTS.iter().any(|event| {
        let subcommand = crate::hooks::claude::lifecycle_subcommand(event);

        normalized.contains(&format!("hook {subcommand}"))
    });

    let plain_managed = normalized.contains("claude-skills")
        && (has_any_lifecycle || normalized.contains("hook instructions --format json"));

    if plain_managed {
        return true;
    }

    if depth >= MAX_DECODE_DEPTH {
        return false;
    }

    decode_powershell_encoded_command(command)
        .map(|decoded| is_managed_hook_command_with_depth(&decoded, depth + 1))
        .unwrap_or(false)
}

pub fn managed_lifecycle_command(subcommand: &str) -> String {
    match std::env::current_exe() {
        Ok(path) => hook_command_for_executable_args(&path, &format!("hook {subcommand}")),

        Err(_) => format!("claude-skills hook {subcommand}"),
    }
}

pub fn hook_command_for_executable_args(path: &Path, arguments: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    #[test]
    fn lifecycle_hooks_emit_additional_context() {
        let mut stdout = Vec::new();

        let mut stderr = Vec::new();

        let code = run_hook_lifecycle("user-prompt-submit", &mut stdout, &mut stderr);

        assert_eq!(code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));

        let output: JsonDocument = serde_json::from_slice(&stdout).unwrap();

        let hook_output = output.get("hookSpecificOutput").unwrap();

        assert_eq!(
            hook_output
                .get("hookEventName")
                .and_then(JsonDocument::as_str),
            Some("UserPromptSubmit")
        );

        let context = hook_output
            .get("additionalContext")
            .and_then(JsonDocument::as_str)
            .unwrap();

        assert!(context.contains("preserve-existing-flow"));

        assert!(context.contains("Memory first"));

        assert!(context.contains("review"));
    }

    #[test]
    fn session_start_context_mentions_system_map() {
        let context = session_start_context();

        assert!(context.contains("claude-skills automatic operating contract"));

        assert!(context.contains("Workspace memory system map"));

        assert!(context.contains("review pre-pr"));
    }

    #[test]
    fn memory_key_sanitization_matches_scope_command_shape() {
        let key = sanitize_memory_key(r#"C:\Users\riezh\OneDrive\Documents\test\claude_core"#);

        assert_eq!(key, "c-users-riezh-onedrive-documents-test-claude-core");
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
