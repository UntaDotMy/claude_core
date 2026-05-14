//! Purpose: Separate host-specific hook ownership from workflow, memory, and proxy code.
//! Caller: manager doctor and future hook install/update paths.
//! Dependencies: Host-specific modules for Claude Code, Cursor, and Windsurf.
//! Main Functions: None (host-specific constants only).
//! Side Effects: None.

pub mod claude;
pub mod cursor;
pub mod windsurf;
