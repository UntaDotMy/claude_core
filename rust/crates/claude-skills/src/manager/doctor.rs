//! Purpose: Doctor and hook probe logic for claude-skills manager.
//! Caller: commands.rs via run_doctor_command.
//! Dependencies: std::fs, std::io, std::path, std::process, crate::runtime.
//! Main Functions: run_doctor_command, hook_blocks_raw_command, hook_accepts_wrapped_command, run_hook_probe, write_doctor_check, find_on_path.
//! Side Effects: Runs hook probe commands, writes doctor check output.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::args::FlagSet;
use crate::runtime::{
    discover_repository_layout, display_path, resolve_claude_home, resolve_repository_root,
};

pub fn run_doctor_command(
    _build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("doctor");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let repository_root = match resolve_repository_root(flag_set.string_value("repo-root")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let claude_home = match resolve_claude_home(flag_set.string_value("claude-home")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let layout = match discover_repository_layout(&repository_root) {
        Ok(layout) => layout,
        Err(error) => {
            let _ = writeln!(standard_error, "discover repository layout: {error}");
            return 1;
        }
    };
    let _ = writeln!(standard_output, "Claude Code Skill Pack Doctor");
    let _ = writeln!(standard_output);
    write_doctor_check(
        standard_output,
        "Repository layout",
        &format!(
            "{} skills, {} agents",
            layout.skills.len(),
            layout.agent_names.len()
        ),
    );
    write_doctor_check(standard_output, "Claude home", &display_path(&claude_home));
    let cargo_path = find_on_path("cargo");
    write_doctor_check(
        standard_output,
        "cargo",
        &cargo_path.unwrap_or_else(|| "not found".to_string()),
    );
    let git_path = find_on_path("git");
    write_doctor_check(
        standard_output,
        "git",
        &git_path.unwrap_or_else(|| "not found".to_string()),
    );
    let _ = writeln!(standard_output);
    let _ = writeln!(standard_output, "Hook probe:");
    let raw_blocked = hook_blocks_raw_command();
    let wrapped_accepted = hook_accepts_wrapped_command();
    write_doctor_check(
        standard_output,
        "Raw command blocked",
        if raw_blocked { "yes" } else { "no" },
    );
    write_doctor_check(
        standard_output,
        "Wrapped command accepted",
        if wrapped_accepted { "yes" } else { "no" },
    );
    0
}

fn hook_blocks_raw_command() -> bool {
    run_hook_probe("echo test").is_err()
}

fn hook_accepts_wrapped_command() -> bool {
    run_hook_probe("claude-skills run -- echo test").is_ok()
}

fn run_hook_probe(command: &str) -> Result<(), String> {
    let output = Command::new("sh")
        .args(["-c", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|error| format!("run hook probe: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err("hook probe failed".to_string())
    }
}

fn write_doctor_check(output: &mut dyn Write, label: &str, value: &str) {
    let _ = writeln!(output, "  {}: {}", label, value);
}

fn find_on_path(executable: &str) -> Option<String> {
    let path_var = std::env::var("PATH").ok()?;
    for directory in path_var.split(if cfg!(windows) { ';' } else { ':' }) {
        let candidate = PathBuf::from(directory).join(executable);
        if candidate.is_file() {
            return Some(display_path(&candidate));
        }
        if cfg!(windows) {
            let candidate_exe = candidate.with_extension("exe");
            if candidate_exe.is_file() {
                return Some(display_path(&candidate_exe));
            }
        }
    }
    None
}
