//! Purpose: Reserve Cursor hook integration as a host-specific module.
//! Caller: hooks module and future Cursor install paths.
//! Dependencies: Cursor hook support when implemented.
//! Main Functions: host_name.
//! Side Effects: None.

pub const HOST: &str = "cursor";

pub fn host_name() -> &'static str {
    HOST
}
