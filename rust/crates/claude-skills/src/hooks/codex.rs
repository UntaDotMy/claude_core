//! Purpose: Reserve Codex hook identity for cross-host parity; Claude Code is the primary host.
//! Caller: hooks::supported_hosts.
//! Dependencies: none.
//! Main Functions: required_feature_flag, pre_tool_matcher.
//! Side Effects: None.

pub const HOST: &str = "codex";

#[allow(dead_code)]
pub fn required_feature_flag() -> &'static str {
    "codex_hooks = true"
}

#[allow(dead_code)]
pub fn pre_tool_matcher() -> &'static str {
    "Bash"
}
