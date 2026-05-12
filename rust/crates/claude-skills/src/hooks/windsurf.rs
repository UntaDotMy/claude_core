//! Purpose: Reserve Windsurf hook integration as a host-specific module.
//! Caller: hooks module and future Windsurf install paths.
//! Dependencies: Windsurf hook support when implemented.
//! Main Functions: host_name.
//! Side Effects: None.

pub const HOST: &str = "windsurf";

pub fn host_name() -> &'static str {
    HOST
}
