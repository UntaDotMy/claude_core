//! Purpose: Separate host-specific hook ownership from workflow, memory, and proxy code.
//! Caller: manager doctor and future hook install/update paths.
//! Dependencies: Host-specific modules for Claude Code, Cursor, and Windsurf.
//! Main Functions: supported_hosts.
//! Side Effects: None.

pub mod claude;
pub mod cursor;
pub mod windsurf;

pub fn supported_hosts() -> [&'static str; 3] {
    [
        claude::host_name(),
        cursor::host_name(),
        windsurf::host_name(),
    ]
}
