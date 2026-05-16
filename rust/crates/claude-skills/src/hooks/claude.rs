//! Purpose: Describe Claude Code hook expectations for token-saving proxy interception.
//! Caller: hooks module, runner managed-hook payload, doctor checks.
//! Dependencies: Claude Code settings.json hooks schema.
//! Main Functions: host_name, required_feature_flag, pre_tool_matcher, post_tool_matcher, events, settings_file_name, lifecycle_subcommand, status_message.
//! Side Effects: None.

pub const HOST: &str = "claude";

/// Claude Code stores hook configuration inside `settings.json` under a top-level `hooks` key.
pub const SETTINGS_FILE_NAME: &str = "settings.json";

/// Claude Code's managed hook events — the lifecycle breadcrumbs the claude-skills
/// manager subscribes to. Covers all 9 no-matcher events from the official spec at
/// code.claude.com/docs/en/hooks plus the matcher-supported events the manager
/// materially handles (tool rewriting, permission tracking, compaction checkpoints,
/// session and prompt lifecycle, task tracking, failure recording). Order is stable
/// so rendered settings.json entries remain deterministic.
///
/// No-matcher events (UserPromptSubmit, Stop, TaskCreated, TaskCompleted,
/// PostToolBatch, TeammateIdle, WorktreeCreate, WorktreeRemove, CwdChanged) get
/// an empty matcher string — the spec rejects matcher fields for these.
pub const EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PostToolBatch",
    "PermissionRequest",
    "Notification",
    "UserPromptSubmit",
    "Stop",
    "SubagentStop",
    "TaskCreated",
    "TaskCompleted",
    "TeammateIdle",
    "WorktreeCreate",
    "WorktreeRemove",
    "CwdChanged",
    "PreCompact",
    "PostCompact",
    "SessionStart",
    "SessionEnd",
];

pub const fn host_name() -> &'static str {
    HOST
}

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

/// The PostToolUse matcher that scopes our post-shell lifecycle hook to Bash results.
/// Other tool surfaces (Read, Edit, Glob, Grep, Task, etc.) have no shell output to record,
/// so an empty matcher would just spawn the hook handler on every tool call for no benefit.
pub const fn post_tool_matcher() -> &'static str {
    "Bash"
}

/// Map a Claude Code hook event name to the `claude-skills hook <subcommand>` kebab-case slug.
pub fn lifecycle_subcommand(event: &str) -> &'static str {
    match event {
        "PreToolUse" => "pre-tool-use",
        "PostToolUse" => "post-tool-use",
        "PostToolUseFailure" => "post-tool-use-failure",
        "PostToolBatch" => "post-tool-batch",
        "PermissionRequest" => "permission-request",
        "Notification" => "notification",
        "UserPromptSubmit" => "user-prompt-submit",
        "Stop" => "stop",
        "SubagentStop" => "subagent-stop",
        "TaskCreated" => "task-created",
        "TaskCompleted" => "task-completed",
        "TeammateIdle" => "teammate-idle",
        "WorktreeCreate" => "worktree-create",
        "WorktreeRemove" => "worktree-remove",
        "CwdChanged" => "cwd-changed",
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
        "PostToolBatch" => "Recording post-tool batch lifecycle",
        "PermissionRequest" => "Recording permission lifecycle",
        "Notification" => "Recording notification lifecycle",
        "UserPromptSubmit" => "Routing prompt to the right skill",
        "Stop" => "Closing native session state",
        "SubagentStop" => "Closing subagent lifecycle",
        "TaskCreated" => "Recording task creation in workflow ledger",
        "TaskCompleted" => "Recording task completion in workflow ledger",
        "TeammateIdle" => "Recording teammate idle lifecycle",
        "WorktreeCreate" => "Recording worktree creation lifecycle",
        "WorktreeRemove" => "Recording worktree removal lifecycle",
        "CwdChanged" => "Recording working directory change",
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
            "PostToolBatch",
            "PermissionRequest",
            "Notification",
            "UserPromptSubmit",
            "Stop",
            "SubagentStop",
            "TaskCreated",
            "TaskCompleted",
            "TeammateIdle",
            "WorktreeCreate",
            "WorktreeRemove",
            "CwdChanged",
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
    fn no_matcher_events_have_lifecycle_subcommands() {
        for event in [
            "PostToolBatch",
            "TeammateIdle",
            "WorktreeCreate",
            "WorktreeRemove",
            "CwdChanged",
        ] {
            let sub = lifecycle_subcommand(event);
            assert_ne!(sub, "unknown", "no subcommand mapping for {event}");
            assert_ne!(
                status_message(event),
                "Native lifecycle hook",
                "no status_message for {event}"
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
