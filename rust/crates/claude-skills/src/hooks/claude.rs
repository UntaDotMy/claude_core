//! Purpose: Describe Claude Code hook expectations for token-saving proxy interception.
//! Caller: hooks module, runner managed-hook payload, doctor checks.
//! Dependencies: Claude Code settings.json hooks schema.
//! Main Functions: required_feature_flag, pre_tool_matcher, events, settings_file_name, lifecycle_subcommand, status_message.
//! Side Effects: None.

/// Claude Code stores hook configuration inside `settings.json` under a top-level `hooks` key.
pub const SETTINGS_FILE_NAME: &str = "settings.json";

/// Claude Code's managed hook events — the lifecycle breadcrumbs the claude-skills
/// manager subscribes to. This is a curated subset of the full event set documented
/// at code.claude.com/docs/en/hooks; we wire up the ones that materially affect the
/// manager's behavior (tool rewriting, permission tracking, compaction checkpoints,
/// session and prompt lifecycle, task tracking, failure recording). Order is stable
/// so rendered settings.json entries remain deterministic.
///
/// Note: 9 events in the official spec have no matcher support
/// (UserPromptSubmit, Stop, TaskCreated, TaskCompleted, PostToolBatch, TeammateIdle,
/// WorktreeCreate, WorktreeRemove, CwdChanged). Do not add a matcher for those.
pub const EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "Notification",
    "UserPromptSubmit",
    "Stop",
    "SubagentStop",
    "TaskCreated",
    "TaskCompleted",
    "PreCompact",
    "PostCompact",
    "SessionStart",
    "SessionEnd",
];

/// Claude Code uses no dedicated feature flag; hooks are active whenever settings.json is loaded.
#[allow(dead_code)]
pub const fn required_feature_flag() -> &'static str {
    ""
}

/// The PreToolUse matcher that scopes our command-rewriting hook to shell invocations.
/// Claude Code uses the exact tool name `Bash` for its built-in shell tool.
pub const fn pre_tool_matcher() -> &'static str {
    "Bash"
}

/// Map a Claude Code hook event name to the `claude-skills hook <subcommand>` kebab-case slug.
pub fn lifecycle_subcommand(event: &str) -> &'static str {
    match event {
        "PreToolUse" => "pre-tool-use",
        "PostToolUse" => "post-tool-use",
        "PostToolUseFailure" => "post-tool-use-failure",
        "PermissionRequest" => "permission-request",
        "Notification" => "notification",
        "UserPromptSubmit" => "user-prompt-submit",
        "Stop" => "stop",
        "SubagentStop" => "subagent-stop",
        "TaskCreated" => "task-created",
        "TaskCompleted" => "task-completed",
        "PreCompact" => "pre-compact",
        "PostCompact" => "post-compact",
        "SessionStart" => "session-start",
        "SessionEnd" => "session-end",
        _ => "unknown",
    }
}

/// Human-readable status message surfaced in Claude Code's hook feedback UI.
pub fn status_message(event: &str) -> &'static str {
    match event {
        "PreToolUse" => "Transparently rewriting noisy commands via claude-skills run",
        "PostToolUse" => "Recording post-tool lifecycle",
        "PostToolUseFailure" => "Recording tool failure for routing and recovery",
        "PermissionRequest" => "Recording permission lifecycle",
        "Notification" => "Recording notification lifecycle",
        "UserPromptSubmit" => "Routing prompt to the right skill",
        "Stop" => "Closing native session state",
        "SubagentStop" => "Closing subagent lifecycle",
        "TaskCreated" => "Recording task creation in workflow ledger",
        "TaskCompleted" => "Recording task completion in workflow ledger",
        "PreCompact" => "Checkpointing before compaction",
        "PostCompact" => "Resuming after compaction",
        "SessionStart" => "Preparing native session state",
        "SessionEnd" => "Recording session end",
        _ => "Native lifecycle hook",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_are_claude_code_canonical_set() {
        for expected in [
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
            "PermissionRequest",
            "Notification",
            "UserPromptSubmit",
            "Stop",
            "SubagentStop",
            "TaskCreated",
            "TaskCompleted",
            "PreCompact",
            "PostCompact",
            "SessionStart",
            "SessionEnd",
        ] {
            assert!(
                EVENTS.contains(&expected),
                "missing canonical event {expected}"
            );
        }
    }

    #[test]
    fn lifecycle_subcommand_maps_every_event() {
        for event in EVENTS {
            let sub = lifecycle_subcommand(event);
            assert!(!sub.is_empty());
            assert!(sub.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
    }

    #[test]
    fn settings_file_name_is_claude_code_convention() {
        assert_eq!(SETTINGS_FILE_NAME, "settings.json");
    }
}
