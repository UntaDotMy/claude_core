//! Purpose: Rust-native manager commands for install, status, validate, verify, update, doctor, all, menu, and uninstall.
//! Caller: commands.rs top-level command dispatch.
//! Dependencies: args, runtime helpers, claude-skills-platform, std::fs, std::io, std::path, and std::time.
//! Main Functions: run_install_command, run_status_command, run_validate_command, run_verify_command, run_uninstall_command.
//! Side Effects: Copies managed skill-pack files, writes Claude home config/state, publishes the Rust binary, runs cargo/git validation commands, and removes managed files during uninstall.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use claude_skills_platform::detect_current_target;

use crate::args::FlagSet;
use crate::runtime::{
    agent_profiles_directory, agents_directory, config_path, discover_repository_layout,
    display_path, forward_process_result, git_short_head, installed_executable_path,
    read_text_if_exists, remove_path_if_exists, repository_layout_is_complete, resolve_claude_home,
    resolve_repository_root, run_command, skills_directory, state_directory, write_lines,
    write_text, AgentConfig, RepositoryLayout, SkillDefinition, SKILL_SYNC_DIRECTORIES,
};

pub fn run_install_command(
    build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("install");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    match install_from_flags(build_version, &flag_set) {
        Ok(summary) => {
            write_install_summary(&summary, standard_output);
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "Native Rust install failed: {error}");
            1
        }
    }
}

pub fn run_status_command(
    build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("status");
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
    let installed_skill_count = count_installed_skills(&claude_home);
    let metadata = read_text_if_exists(&install_metadata_path(&claude_home)).unwrap_or_default();
    let layout = discover_repository_layout(&repository_root);
    let source_skill_count = layout
        .as_ref()
        .ok()
        .map(|value| value.skills.len())
        .or_else(|| count_managed_skills(&claude_home));
    let source_display = if layout.is_ok() {
        display_path(&repository_root)
    } else if source_skill_count.is_some() {
        format!(
            "installed inventory (source unavailable from {})",
            display_path(&repository_root)
        )
    } else {
        format!("unavailable from {}", display_path(&repository_root))
    };
    let repo_version = if layout.is_ok() {
        repo_version_for_source(build_version, &repository_root)
    } else {
        repo_version_from_metadata_or_build(&metadata, build_version)
            .unwrap_or_else(|| "unavailable".to_string())
    };
    let update_status = match source_skill_count {
        Some(expected_count) if installed_skill_count == expected_count => "current",
        Some(_) => "refresh recommended",
        None if installed_skill_count == 0 => "not installed",
        None => "source unavailable",
    };
    let synced_skills = match source_skill_count {
        Some(expected_count) => format!("{installed_skill_count}/{expected_count}"),
        None => format!("{installed_skill_count}/unknown"),
    };
    let target = detect_current_target()
        .map(|value| value.directory_name())
        .unwrap_or_else(|error| format!("unknown ({error})"));
    let _ = writeln!(standard_output, "Codex Skill Pack Status");
    let _ = writeln!(standard_output);
    let _ = writeln!(standard_output, "Summary:");
    let _ = writeln!(standard_output, "  Manager version: {build_version}");
    let _ = writeln!(standard_output, "  Repo version: {}", repo_version);
    let _ = writeln!(
        standard_output,
        "  Installed version: {}",
        metadata_value(&metadata, "manager_version").unwrap_or("not installed")
    );
    let _ = writeln!(standard_output, "  Install source: Rust-native manager");
    let _ = writeln!(
        standard_output,
        "  Skill pack update status: {}",
        update_status
    );
    let _ = writeln!(standard_output);
    let _ = writeln!(standard_output, "Codex Skills:");
    let _ = writeln!(standard_output, "  Source: {}", source_display);
    let _ = writeln!(
        standard_output,
        "  Target: {}",
        display_path(&skills_directory(&claude_home))
    );
    let _ = writeln!(standard_output, "  Platform: {target}");
    let _ = writeln!(standard_output, "  Synced skills: {synced_skills}");
    let _ = writeln!(standard_output);
    let _ = writeln!(standard_output, "Runtime:");
    let _ = writeln!(standard_output, "  implementation: rust");
    0
}

pub fn run_doctor_command(
    build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let status_code = run_status_command(build_version, arguments, standard_output, standard_error);
    if status_code != 0 {
        return status_code;
    }
    let claude_home = match resolve_claude_home("") {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let _ = config_path(&claude_home);
    let hooks_path = claude_home.join(crate::hooks::claude::SETTINGS_FILE_NAME);
    let hooks_text = fs::read_to_string(&hooks_path).unwrap_or_default();
    let claude_binary = find_on_path(if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    });
    let _ = writeln!(standard_output, "Doctor:");
    let _ = writeln!(
        standard_output,
        "[ok] binary: {}",
        display_path(&std::env::current_exe().unwrap_or_else(|_| PathBuf::from("claude-skills")))
    );
    let raw_store = crate::proxy::raw_store::RawStore::new();
    let raw_writable = fs::create_dir_all(raw_store.root())
        .and_then(|_| {
            let probe = raw_store.root().join(".doctor-write-probe");
            fs::write(&probe, b"ok").and_then(|_| fs::remove_file(probe))
        })
        .is_ok();
    write_doctor_check(
        standard_output,
        raw_writable,
        &format!("raw store writable: {}", display_path(raw_store.root())),
    );
    let event_path = claude_home.join(crate::runtime::COMMAND_COMPACTION_EVENTS_FILE_NAME);
    let event_writable = fs::create_dir_all(&claude_home)
        .and_then(|_| {
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&event_path)
        })
        .is_ok();
    write_doctor_check(
        standard_output,
        event_writable,
        &format!("event log writable: {}", display_path(&event_path)),
    );
    let _ = writeln!(
        standard_output,
        "[ok] adapters: {}",
        crate::proxy::adapters::adapter_names()
    );
    let rewrite_probe = crate::runner::rewrite_for_doctor("cargo test");
    write_doctor_check(
        standard_output,
        rewrite_probe.contains("run -- cargo test"),
        "rewrite: cargo test -> claude-skills run -- cargo test",
    );
    write_doctor_check(
        standard_output,
        claude_binary.is_some(),
        "claude binary found",
    );
    write_doctor_check(
        standard_output,
        hooks_path.exists(),
        "~/.claude/settings.json exists",
    );
    write_doctor_check(
        standard_output,
        hooks_text.contains("PreToolUse")
            && hooks_text.contains(crate::hooks::claude::pre_tool_matcher()),
        "PreToolUse Bash matcher installed",
    );
    let dry_run_blocks = codex_hook_blocks_raw_command();
    write_doctor_check(
        standard_output,
        dry_run_blocks,
        "dry-run raw command is blocked with rerun guidance",
    );
    write_doctor_check(
        standard_output,
        codex_hook_accepts_wrapped_command() && installed_executable_path(&claude_home).exists(),
        "rerun wrapper command is accepted",
    );
    let _ = writeln!(
        standard_output,
        "[warn] unified_exec interception incomplete in current Codex"
    );
    let _ = writeln!(
        standard_output,
        "hook hosts: {}",
        crate::hooks::supported_hosts().join(", ")
    );
    let _ = writeln!(
        standard_output,
        "Run `claude-skills validate --profile smoke` for local proof."
    );
    0
}

fn codex_hook_blocks_raw_command() -> bool {
    run_hook_probe("cargo test --workspace")
        .map(|output| output.contains("permissionDecision") && output.contains("Rerun that as:"))
        .unwrap_or(false)
}

fn codex_hook_accepts_wrapped_command() -> bool {
    let executable = std::env::current_exe()
        .map(|path| display_path(&path))
        .unwrap_or_else(|_| "claude-skills".to_string());
    let command = format!("{executable} run -- cargo test --workspace");
    run_hook_probe(&command)
        .map(|output| !output.contains("permissionDecision"))
        .unwrap_or(false)
}

fn run_hook_probe(command: &str) -> Option<String> {
    let executable = std::env::current_exe().ok()?;
    let mut child = Command::new(executable)
        .args(["hook", "pre-tool-use"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let input = serde_json::json!({
        "tool_name": "Bash",
        "tool_input": {
            "command": command
        }
    });
    if let Some(mut stdin) = child.stdin.take() {
        let _ = write!(stdin, "{}", input);
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn write_doctor_check(standard_output: &mut dyn Write, ok: bool, message: &str) {
    let status = if ok { "[ok]" } else { "[warn]" };
    let _ = writeln!(standard_output, "{status} {message}");
}

fn find_on_path(executable: &str) -> Option<PathBuf> {
    let path_value = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path_value) {
        let candidate = directory.join(executable);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(windows) && !executable.ends_with(".exe") {
            let candidate = directory.join(format!("{executable}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn run_update_command(
    build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("update");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    flag_set.bool_flag("no-build", false);
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let claude_home = match resolve_claude_home(flag_set.string_value("claude-home")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let repository_root = match resolve_update_repository_root(&flag_set, &claude_home) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    match current_git_branch(&repository_root) {
        Ok(Some(_branch_name)) => {}
        Ok(None) => {
            let _ = writeln!(
                standard_output,
                "Rust update detected detached HEAD; skipping git pull and reinstalling current checkout."
            );
            let install_arguments = vec![
                "--repo-root".to_string(),
                display_path(&repository_root),
                "--claude-home".to_string(),
                display_path(&claude_home),
            ];
            return run_install_command(
                build_version,
                &install_arguments,
                standard_output,
                standard_error,
            );
        }
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Rust update could not inspect git checkout: {error}"
            );
            return 1;
        }
    }
    let pull_arguments = vec!["pull".to_string(), "--ff-only".to_string()];
    match run_command("git", &pull_arguments, Some(&repository_root)) {
        Ok(result) => {
            forward_process_result(&result, standard_output, standard_error);
            if result.code != 0 {
                return result.code.clamp(1, 255) as u8;
            }
        }
        Err(error) => {
            let _ = writeln!(
                standard_error,
                "Rust update could not refresh git checkout: {error}"
            );
            return 1;
        }
    }
    let built_release = !flag_set.bool_value("no-build");
    if built_release {
        let build_arguments = vec![
            "build".to_string(),
            "--release".to_string(),
            "--bin".to_string(),
            "claude-skills".to_string(),
        ];
        match run_command("cargo", &build_arguments, Some(&repository_root)) {
            Ok(result) => {
                forward_process_result(&result, standard_output, standard_error);
                if result.code != 0 {
                    return result.code.clamp(1, 255) as u8;
                }
            }
            Err(error) => {
                let _ = writeln!(
                    standard_error,
                    "Rust update could not build release binary: {error}"
                );
                return 1;
            }
        }
    }
    let release_executable = repository_root
        .join("target")
        .join("release")
        .join(executable_file_name());
    let mut publish_during_install = true;
    let mut release_executable_action = String::new();
    if release_executable.is_file() {
        match publish_specific_executable(&release_executable, &claude_home) {
            Ok(action) => {
                release_executable_action = action;
                publish_during_install = false;
            }
            Err(error) => {
                let _ = writeln!(
                    standard_error,
                    "Rust update could not publish built executable: {error}"
                );
                return 1;
            }
        }
    } else if built_release {
        let _ = writeln!(
            standard_error,
            "Rust update built successfully but release executable is missing at {}",
            display_path(&release_executable)
        );
        return 1;
    }
    match install_from_paths(
        build_version,
        &repository_root,
        &claude_home,
        publish_during_install,
    ) {
        Ok(mut summary) => {
            if !release_executable_action.is_empty() {
                summary.executable_action = release_executable_action;
            }
            write_install_summary(&summary, standard_output);
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "Native Rust update failed: {error}");
            1
        }
    }
}

fn resolve_update_repository_root(
    flag_set: &FlagSet,
    claude_home: &Path,
) -> Result<PathBuf, String> {
    if !flag_set.string_value("repo-root").trim().is_empty() {
        return resolve_repository_root(flag_set.string_value("repo-root"));
    }
    let current = resolve_repository_root("")?;
    if repository_layout_is_complete(&current) {
        return Ok(current);
    }
    let metadata = read_text_if_exists(&install_metadata_path(claude_home))?;
    if let Some(repository_root) = metadata_value(&metadata, "repository_root") {
        let path = PathBuf::from(repository_root);
        if repository_layout_is_complete(&path) {
            return Ok(path);
        }
    }
    Err("Rust update needs a claude_skills checkout. Run from the checkout, pass --repo-root, or install once from a GitHub-cloned checkout so metadata can remember it.".to_string())
}

fn current_git_branch(repository_root: &Path) -> Result<Option<String>, String> {
    let branch_arguments = vec![
        "rev-parse".to_string(),
        "--abbrev-ref".to_string(),
        "HEAD".to_string(),
    ];
    let result = run_command("git", &branch_arguments, Some(repository_root))?;
    if result.code != 0 {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }
    let branch_name = String::from_utf8_lossy(&result.stdout).trim().to_string();
    if branch_name == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(branch_name))
    }
}

pub fn run_verify_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("verify");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    if flag_set.positional.len() > 1 {
        let _ = writeln!(
            standard_error,
            "Native Rust verify accepts at most one optional skill name, got {} arguments",
            flag_set.positional.len()
        );
        return 1;
    }
    match verify_install(
        &flag_set,
        flag_set.positional.first().map(String::as_str),
        standard_output,
    ) {
        Ok(()) => 0,
        Err(error) => {
            let _ = writeln!(standard_error, "Native Rust verify failed: {error}");
            1
        }
    }
}

pub fn run_uninstall_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("uninstall");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let claude_home = match resolve_claude_home(flag_set.string_value("claude-home")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let state = state_directory(&claude_home);
    for name in read_inventory_lines(&state.join("managed-skills.txt")) {
        if let Err(error) = remove_path_if_exists(&skills_directory(&claude_home).join(&name)) {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    }
    for name in read_inventory_lines(&state.join("managed-home-agents.txt")) {
        let agent_name = name.split('|').nth(1).unwrap_or(&name);
        let _ = remove_path_if_exists(
            &agents_directory(&claude_home).join(format!("{agent_name}.toml")),
        );
    }
    for name in read_inventory_lines(&state.join("managed-agent-profiles.txt")) {
        let _ = remove_path_if_exists(
            &agent_profiles_directory(&claude_home).join(format!("{name}.toml")),
        );
    }
    for relative_path in crate::runtime::ROOT_GUIDANCE_RELATIVE_PATHS {
        let _ = remove_path_if_exists(&claude_home.join(relative_path));
    }
    let _ = remove_path_if_exists(&installed_executable_path(&claude_home));
    let _ = remove_path_if_exists(&state);
    let _ = writeln!(
        standard_output,
        "Native Rust uninstall removed managed files from {}",
        display_path(&claude_home)
    );
    0
}

pub fn run_validate_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("validate");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("profile", "full");
    flag_set.string_flag("review-surface", "");
    flag_set.string_flag("review-base-ref", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    if !flag_set.positional.is_empty() {
        let _ = writeln!(
            standard_error,
            "Native Rust validate does not accept positional arguments, got {}",
            flag_set.positional.len()
        );
        return 1;
    }
    let repository_root = match resolve_repository_root(flag_set.string_value("repo-root")) {
        Ok(path) => path,
        Err(error) => {
            let _ = writeln!(standard_error, "{error}");
            return 1;
        }
    };
    let profile = flag_set.string_value("profile").trim();
    let mut commands: Vec<(&str, Vec<String>)> = vec![
        (
            "cargo",
            vec![
                "fmt".to_string(),
                "--all".to_string(),
                "--check".to_string(),
            ],
        ),
        ("cargo", vec!["test".to_string(), "--workspace".to_string()]),
    ];
    if profile != "smoke" {
        commands.push(("cargo", vec!["build".to_string(), "--release".to_string()]));
    }
    commands.push(("git", vec!["diff".to_string(), "--check".to_string()]));

    for (program, command_arguments) in commands {
        let _ = writeln!(
            standard_output,
            "[validate] {} {}",
            program,
            command_arguments.join(" ")
        );
        match run_command(program, &command_arguments, Some(&repository_root)) {
            Ok(result) => {
                forward_process_result(&result, standard_output, standard_error);
                if result.code != 0 {
                    let _ = writeln!(standard_error, "Native Rust validate failed in {program}");
                    return result.code.clamp(1, 255) as u8;
                }
            }
            Err(error) => {
                let _ = writeln!(standard_error, "{error}");
                return 1;
            }
        }
    }
    let _ = writeln!(standard_output, "Native Rust validate passed");
    0
}

pub fn run_all_command(
    build_version: &str,
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let validate_code = run_validate_command(arguments, standard_output, standard_error);
    if validate_code != 0 {
        return validate_code;
    }
    let mut flag_set = FlagSet::new("all");
    flag_set.string_flag("repo-root", "");
    flag_set.string_flag("claude-home", "");
    flag_set.string_flag("profile", "full");
    flag_set.string_flag("review-surface", "");
    flag_set.string_flag("review-base-ref", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let mut install_arguments = Vec::new();
    if !flag_set.string_value("repo-root").trim().is_empty() {
        install_arguments.push("--repo-root".to_string());
        install_arguments.push(flag_set.string_value("repo-root").to_string());
    }
    if !flag_set.string_value("claude-home").trim().is_empty() {
        install_arguments.push("--claude-home".to_string());
        install_arguments.push(flag_set.string_value("claude-home").to_string());
    }
    run_install_command(
        build_version,
        &install_arguments,
        standard_output,
        standard_error,
    )
}

pub fn run_menu_command(standard_output: &mut dyn Write) -> u8 {
    let _ = writeln!(standard_output, "Codex Skill Manager");
    let _ = writeln!(
        standard_output,
        "  [1] install  - install or refresh managed Rust-native files"
    );
    let _ = writeln!(
        standard_output,
        "  [2] update   - fast-forward checkout, then install"
    );
    let _ = writeln!(
        standard_output,
        "  [3] status   - show install and runtime state"
    );
    let _ = writeln!(standard_output, "  [4] doctor   - diagnose install state");
    let _ = writeln!(standard_output);
    let _ = writeln!(
        standard_output,
        "Run the named command directly, for example `claude-skills status`."
    );
    0
}

struct InstallSummary {
    claude_home: PathBuf,
    skill_count: usize,
    copied_files: usize,
    skipped_files: usize,
    removed_paths: usize,
    executable_action: String,
}

#[derive(Default)]
struct SyncStats {
    copied_files: usize,
    skipped_files: usize,
    removed_paths: usize,
    executable_action: String,
}

fn install_from_flags(build_version: &str, flag_set: &FlagSet) -> Result<InstallSummary, String> {
    let repository_root = resolve_install_repository_root(flag_set.string_value("repo-root"))?;
    let claude_home = resolve_claude_home(flag_set.string_value("claude-home"))?;
    install_from_paths(build_version, &repository_root, &claude_home, true)
}

fn resolve_install_repository_root(requested_repository_root: &str) -> Result<PathBuf, String> {
    let trimmed = requested_repository_root.trim();
    if !trimmed.is_empty() {
        return resolve_repository_root(trimmed);
    }
    let mut candidates = Vec::new();
    if let Ok(current_directory_root) = resolve_repository_root("") {
        candidates.push(current_directory_root);
    }
    if let Ok(executable_path) = std::env::current_exe() {
        if let Some(parent) = executable_path.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    resolve_install_repository_root_from_candidates(candidates).ok_or_else(|| {
        "claude-skills install could not find the source or extracted release bundle. Run it from the extracted claude-skills folder, or pass --repo-root <path> for advanced installs.".to_string()
    })
}

fn resolve_install_repository_root_from_candidates(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    for candidate in candidates {
        let cleaned = crate::runtime::clean_path(&candidate);
        if repository_layout_is_complete(&cleaned) {
            return Some(cleaned);
        }
        for ancestor in cleaned.ancestors().skip(1).take(4) {
            if repository_layout_is_complete(ancestor) {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}

fn install_from_paths(
    build_version: &str,
    repository_root: &Path,
    claude_home: &Path,
    publish_executable: bool,
) -> Result<InstallSummary, String> {
    let layout = discover_repository_layout(&repository_root)?;
    let mut stats = SyncStats::default();
    ensure_claude_home_directories(&claude_home)?;
    remove_stale_managed_files(&layout, &claude_home, &mut stats)?;
    sync_root_files(&layout, &claude_home, &mut stats)?;
    sync_skills(&layout, &claude_home, &mut stats)?;
    sync_agents(&layout, &claude_home, &mut stats)?;
    write_managed_config(&layout, &claude_home)?;
    if publish_executable {
        publish_native_executable(&claude_home, &mut stats)?;
    }
    write_install_metadata(build_version, &layout, &claude_home)?;
    write_inventories(&layout, &claude_home)?;
    Ok(InstallSummary {
        claude_home: claude_home.to_path_buf(),
        skill_count: layout.skills.len(),
        copied_files: stats.copied_files,
        skipped_files: stats.skipped_files,
        removed_paths: stats.removed_paths,
        executable_action: if stats.executable_action.is_empty() {
            "unchanged".to_string()
        } else {
            stats.executable_action
        },
    })
}

fn write_install_summary(summary: &InstallSummary, standard_output: &mut dyn Write) {
    let _ = writeln!(
        standard_output,
        "Native Rust install synced {} skills to {}",
        summary.skill_count,
        display_path(&summary.claude_home)
    );
    let _ = writeln!(
        standard_output,
        "Delta sync: copied={} unchanged={} removed={} executable={}",
        summary.copied_files,
        summary.skipped_files,
        summary.removed_paths,
        summary.executable_action
    );
}

fn ensure_claude_home_directories(claude_home: &Path) -> Result<(), String> {
    for directory in [
        claude_home.to_path_buf(),
        skills_directory(claude_home),
        agents_directory(claude_home),
        agent_profiles_directory(claude_home),
        claude_home.join("memories"),
        claude_home.join("memories/workspaces"),
        claude_home.join("memories/agents"),
        claude_home.join("memories/research_cache"),
        claude_home.join("memories/archive"),
        claude_home.join("memories/reports"),
        claude_home.join("memoriesv2"),
        claude_home.join("memoriesv2/global"),
        claude_home.join("memoriesv2/workspaces"),
        state_directory(claude_home).join("manifests/source"),
        state_directory(claude_home).join("manifests/target"),
    ] {
        fs::create_dir_all(&directory)
            .map_err(|error| format!("create {}: {error}", display_path(&directory)))?;
    }
    Ok(())
}

fn remove_stale_managed_files(
    layout: &RepositoryLayout,
    claude_home: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    let current_skills: std::collections::BTreeSet<String> = layout
        .skills
        .iter()
        .map(|skill| skill.name.clone())
        .collect();
    for old_skill in read_inventory_lines(&state_directory(claude_home).join("managed-skills.txt"))
    {
        if !current_skills.contains(&old_skill) {
            remove_path_if_exists_counted(&skills_directory(claude_home).join(old_skill), stats)?;
        }
    }

    let current_agents: std::collections::BTreeSet<String> =
        layout.agent_names.iter().cloned().collect();
    for old_agent_line in
        read_inventory_lines(&state_directory(claude_home).join("managed-home-agents.txt"))
    {
        let old_agent = old_agent_line.split('|').nth(1).unwrap_or(&old_agent_line);
        if !current_agents.contains(old_agent) {
            remove_path_if_exists_counted(
                &agents_directory(claude_home).join(format!("{old_agent}.toml")),
                stats,
            )?;
        }
    }
    for old_profile in
        read_inventory_lines(&state_directory(claude_home).join("managed-agent-profiles.txt"))
    {
        if !current_agents.contains(&old_profile) {
            remove_path_if_exists_counted(
                &agent_profiles_directory(claude_home).join(format!("{old_profile}.toml")),
                stats,
            )?;
        }
    }

    for obsolete_launcher in [
        "sync-skills.sh",
        "sync-skills.ps1",
        "scripts/command-compaction-profile.sh",
        "scripts/command-compaction-profile.ps1",
        "scripts/preflight-mr.sh",
        "scripts/preflight-mr.ps1",
        "scripts/build-all.sh",
        "scripts/build-all.ps1",
    ] {
        remove_path_if_exists_counted(&claude_home.join(obsolete_launcher), stats)?;
    }
    remove_deprecated_config_keys(claude_home, stats)?;
    Ok(())
}

fn remove_deprecated_config_keys(claude_home: &Path, stats: &mut SyncStats) -> Result<(), String> {
    let path = config_path(claude_home);
    let config_text = read_text_if_exists(&path)?;
    let mut changed = false;
    let mut kept_lines = Vec::new();
    let mut in_features = false;

    for line in config_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_features = trimmed == "[features]";
        }
        if in_features && trimmed.starts_with("use_legacy_landlock") {
            changed = true;
            continue;
        }
        kept_lines.push(line);
    }

    if changed {
        let mut updated = kept_lines.join("\n");
        if config_text.ends_with('\n') {
            updated.push('\n');
        }
        write_text(&path, &updated)?;
        stats.copied_files += 1;
    }
    Ok(())
}

fn copy_file_if_changed(
    source_path: &Path,
    target_path: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    let source_bytes = fs::read(source_path)
        .map_err(|error| format!("read source {}: {error}", display_path(source_path)))?;
    if let Ok(target_bytes) = fs::read(target_path) {
        if source_bytes == target_bytes {
            stats.skipped_files += 1;
            return Ok(());
        }
    }
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", display_path(parent)))?;
    }
    fs::write(target_path, source_bytes)
        .map_err(|error| format!("write {}: {error}", display_path(target_path)))?;
    stats.copied_files += 1;
    Ok(())
}

fn write_text_if_changed(
    target_path: &Path,
    text: &str,
    stats: &mut SyncStats,
) -> Result<(), String> {
    if read_text_if_exists(target_path)? == text {
        stats.skipped_files += 1;
        return Ok(());
    }
    write_text(target_path, text)?;
    stats.copied_files += 1;
    Ok(())
}

fn sync_directory_delta(
    source_directory: &Path,
    target_directory: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    fs::create_dir_all(target_directory)
        .map_err(|error| format!("create {}: {error}", display_path(target_directory)))?;
    let mut source_names = std::collections::BTreeSet::new();
    for entry_result in fs::read_dir(source_directory)
        .map_err(|error| format!("read {}: {error}", display_path(source_directory)))?
    {
        let entry = entry_result.map_err(|error| format!("read directory entry: {error}"))?;
        let file_name = entry.file_name();
        source_names.insert(file_name.clone());
        let source_path = entry.path();
        let target_path = target_directory.join(&file_name);
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read type for {}: {error}", display_path(&source_path)))?;
        if file_type.is_dir() {
            sync_directory_delta(&source_path, &target_path, stats)?;
        } else if file_type.is_file() {
            copy_file_if_changed(&source_path, &target_path, stats)?;
        }
    }

    if target_directory.is_dir() {
        for entry_result in fs::read_dir(target_directory)
            .map_err(|error| format!("read {}: {error}", display_path(target_directory)))?
        {
            let entry = entry_result.map_err(|error| format!("read directory entry: {error}"))?;
            if !source_names.contains(&entry.file_name()) {
                remove_path_if_exists_counted(&entry.path(), stats)?;
            }
        }
    }
    Ok(())
}

fn remove_path_if_exists_counted(path: &Path, stats: &mut SyncStats) -> Result<(), String> {
    if path.exists() {
        remove_path_if_exists(path)?;
        stats.removed_paths += 1;
    }
    Ok(())
}

fn sync_root_files(
    layout: &RepositoryLayout,
    claude_home: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    for relative_path in &layout.root_files {
        copy_file_if_changed(
            &layout.root_path.join(relative_path),
            &claude_home.join(relative_path),
            stats,
        )?;
    }
    Ok(())
}

fn sync_skills(
    layout: &RepositoryLayout,
    claude_home: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    for skill in &layout.skills {
        let target_skill_directory = skills_directory(claude_home).join(&skill.name);
        fs::create_dir_all(&target_skill_directory).map_err(|error| {
            format!("create {}: {error}", display_path(&target_skill_directory))
        })?;
        copy_file_if_changed(
            &skill.skill_path.join("SKILL.md"),
            &target_skill_directory.join("SKILL.md"),
            stats,
        )?;
        for relative_directory in SKILL_SYNC_DIRECTORIES {
            let source_directory = skill.skill_path.join(relative_directory);
            let target_directory = target_skill_directory.join(relative_directory);
            if source_directory.is_dir() {
                sync_directory_delta(&source_directory, &target_directory, stats)?;
            } else if target_directory.exists() {
                remove_path_if_exists_counted(&target_directory, stats)?;
            }
        }
    }
    Ok(())
}

fn sync_agents(
    layout: &RepositoryLayout,
    claude_home: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    for skill in &layout.skills {
        for agent_config in &skill.agent_configs {
            let parsed_config = parse_agent_config(agent_config)?;
            let rendered_toml = render_agent_toml(&parsed_config, &agent_config.agent_name)?;
            write_text_if_changed(
                &agents_directory(claude_home).join(format!("{}.toml", agent_config.agent_name)),
                &rendered_toml,
                stats,
            )?;
            write_text_if_changed(
                &agent_profiles_directory(claude_home)
                    .join(format!("{}.toml", agent_config.agent_name)),
                &rendered_toml,
                stats,
            )?;
        }
    }
    Ok(())
}

fn write_managed_config(layout: &RepositoryLayout, claude_home: &Path) -> Result<(), String> {
    let path = config_path(claude_home);
    let mut config_text = read_text_if_exists(&path)?;
    config_text = remove_managed_block(&config_text, "CLAUDE SKILLS RUST MANAGED ROUTING");
    config_text = remove_managed_block(&config_text, "CLAUDE SKILLS RUST MANAGED AGENTS");

    let routing_block = [
        "# BEGIN CLAUDE SKILLS RUST MANAGED ROUTING",
        "# This block is maintained by the Rust claude-skills manager.",
        "# Route specialist work through the installed skill pack and preserve existing flow before brownfield edits.",
        "# END CLAUDE SKILLS RUST MANAGED ROUTING",
        "",
    ]
    .join("\n");

    let mut agent_block_lines = vec![
        "# BEGIN CLAUDE SKILLS RUST MANAGED AGENTS".to_string(),
        "# Agent profiles are installed under agent-profiles/ and home agents under agents/."
            .to_string(),
    ];
    for agent_name in &layout.agent_names {
        agent_block_lines.push(format!("# managed_agent = \"{agent_name}\""));
    }
    agent_block_lines.push("# END CLAUDE SKILLS RUST MANAGED AGENTS".to_string());
    agent_block_lines.push(String::new());

    if !config_text.trim().is_empty() {
        config_text = config_text.trim_end().to_string();
        config_text.push_str("\n\n");
    }
    config_text.push_str(&routing_block);
    config_text.push_str(&agent_block_lines.join("\n"));
    write_text(&path, &config_text)
}

fn remove_managed_block(config_text: &str, marker_name: &str) -> String {
    let begin_marker = format!("# BEGIN {marker_name}");
    let end_marker = format!("# END {marker_name}");
    let mut kept_lines = Vec::new();
    let mut skipping = false;
    for line in config_text.lines() {
        if line.trim() == begin_marker {
            skipping = true;
            continue;
        }
        if skipping && line.trim() == end_marker {
            skipping = false;
            continue;
        }
        if !skipping {
            kept_lines.push(line);
        }
    }
    kept_lines.join("\n")
}

fn publish_native_executable(claude_home: &Path, stats: &mut SyncStats) -> Result<(), String> {
    let current_executable =
        std::env::current_exe().map_err(|error| format!("resolve current executable: {error}"))?;
    let installed_executable = installed_executable_path(claude_home);
    let current_canonical =
        fs::canonicalize(&current_executable).unwrap_or(current_executable.clone());
    let installed_canonical =
        fs::canonicalize(&installed_executable).unwrap_or(installed_executable.clone());
    if current_canonical == installed_canonical {
        stats.executable_action = "running-installed".to_string();
        return Ok(());
    }
    if let Ok(current_bytes) = fs::read(&current_executable) {
        if let Ok(installed_bytes) = fs::read(&installed_executable) {
            if current_bytes == installed_bytes {
                stats.executable_action = "unchanged".to_string();
                stats.skipped_files += 1;
                return Ok(());
            }
        }
    }
    atomic_copy_executable(&current_executable, &installed_executable)?;
    stats.executable_action = "updated".to_string();
    stats.copied_files += 1;
    Ok(())
}

fn atomic_copy_executable(source: &Path, target: &Path) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", display_path(parent)))?;
    }
    let temporary_target = target.with_extension(format!(
        "{}tmp",
        target
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!("{value}."))
            .unwrap_or_default()
    ));
    fs::copy(source, &temporary_target).map_err(|error| {
        format!(
            "copy executable {} to {}: {error}",
            display_path(source),
            display_path(&temporary_target)
        )
    })?;
    if target.exists() {
        let _ = fs::remove_file(target);
    }
    fs::rename(&temporary_target, target)
        .or_else(|_| {
            fs::copy(&temporary_target, target)?;
            fs::remove_file(&temporary_target)
        })
        .map_err(|error| format!("publish executable {}: {error}", display_path(target)))?;
    Ok(())
}

fn publish_specific_executable(source: &Path, claude_home: &Path) -> Result<String, String> {
    let installed_executable = installed_executable_path(claude_home);
    if let Ok(source_bytes) = fs::read(source) {
        if let Ok(installed_bytes) = fs::read(&installed_executable) {
            if source_bytes == installed_bytes {
                return Ok("unchanged".to_string());
            }
        }
    }
    let current_executable =
        std::env::current_exe().map_err(|error| format!("resolve current executable: {error}"))?;
    let current_canonical =
        fs::canonicalize(&current_executable).unwrap_or(current_executable.clone());
    let installed_canonical =
        fs::canonicalize(&installed_executable).unwrap_or(installed_executable.clone());
    if current_canonical == installed_canonical {
        spawn_self_replace(source, &installed_executable)?;
        return Ok("staged-self-replace".to_string());
    }
    atomic_copy_executable(source, &installed_executable)?;
    Ok("updated".to_string())
}

fn spawn_self_replace(source: &Path, target: &Path) -> Result<(), String> {
    Command::new(source)
        .arg("__self-replace")
        .arg("--source")
        .arg(source)
        .arg("--target")
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("spawn native self replacement: {error}"))
}

pub fn run_self_replace_command(arguments: &[String], standard_error: &mut dyn Write) -> u8 {
    let mut flag_set = FlagSet::new("__self-replace");
    flag_set.string_flag("source", "");
    flag_set.string_flag("target", "");
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let source = PathBuf::from(flag_set.string_value("source"));
    let target = PathBuf::from(flag_set.string_value("target"));
    if !source.is_file() || target.as_os_str().is_empty() {
        let _ = writeln!(
            standard_error,
            "__self-replace requires --source and --target"
        );
        return 1;
    }
    for _ in 0..60 {
        match atomic_copy_executable(&source, &target) {
            Ok(()) => return 0,
            Err(_) => thread::sleep(Duration::from_millis(250)),
        }
    }
    let _ = writeln!(
        standard_error,
        "unable to replace running executable at {}",
        display_path(&target)
    );
    1
}

fn executable_file_name() -> String {
    detect_current_target()
        .map(|target| target.executable_name().to_string())
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "claude-skills.exe".to_string()
            } else {
                "claude-skills".to_string()
            }
        })
}

fn write_install_metadata(
    build_version: &str,
    layout: &RepositoryLayout,
    claude_home: &Path,
) -> Result<(), String> {
    let target = detect_current_target()
        .map(|value| value.operating_system)
        .unwrap_or_else(|_| "unknown".to_string());
    let lines = vec![
        format!(
            "repo_version={}",
            repo_version_for_source(build_version, &layout.root_path)
        ),
        format!("manager_version={build_version}"),
        "install_channel=rust-source".to_string(),
        "install_source=Rust-native repository checkout".to_string(),
        format!("repository_root={}", display_path(&layout.root_path)),
        format!("updated_at={}", unix_timestamp()),
        format!("platform={target}"),
        format!("target={}", display_path(claude_home)),
    ];
    write_lines(&install_metadata_path(claude_home), &lines)
}

fn repo_version_for_source(build_version: &str, repository_root: &Path) -> String {
    meaningful_repo_version(&git_short_head(repository_root))
        .or_else(|| repo_version_from_build_version(build_version))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn repo_version_from_metadata_or_build(metadata: &str, build_version: &str) -> Option<String> {
    metadata_value(metadata, "repo_version")
        .and_then(meaningful_repo_version)
        .or_else(|| {
            metadata_value(metadata, "manager_version").and_then(repo_version_from_build_version)
        })
        .or_else(|| repo_version_from_build_version(build_version))
}

fn meaningful_repo_version(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "unknown" || trimmed == "unavailable" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn repo_version_from_build_version(build_version: &str) -> Option<String> {
    let release_commit = build_version.trim().strip_prefix("bootstrap-")?;
    let short_commit: String = release_commit
        .chars()
        .take_while(|character| character.is_ascii_hexdigit())
        .take(7)
        .collect();
    if short_commit.len() == 7 {
        Some(short_commit)
    } else {
        None
    }
}

fn write_inventories(layout: &RepositoryLayout, claude_home: &Path) -> Result<(), String> {
    let state = state_directory(claude_home);
    let skill_names: Vec<String> = layout
        .skills
        .iter()
        .map(|skill| skill.name.clone())
        .collect();
    write_lines(&state.join("managed-skills.txt"), &skill_names)?;

    let mut home_agents = Vec::new();
    for skill in &layout.skills {
        for agent in &skill.agent_configs {
            home_agents.push(format!("{}|{}", skill.name, agent.agent_name));
        }
    }
    home_agents.sort();
    write_lines(&state.join("managed-home-agents.txt"), &home_agents)?;
    write_lines(
        &state.join("managed-agent-profiles.txt"),
        &layout.agent_names,
    )
}

fn verify_install(
    flag_set: &FlagSet,
    requested_skill_name: Option<&str>,
    standard_output: &mut dyn Write,
) -> Result<(), String> {
    let repository_root = resolve_repository_root(flag_set.string_value("repo-root"))?;
    let claude_home = resolve_claude_home(flag_set.string_value("claude-home"))?;
    let layout = discover_repository_layout(&repository_root)?;
    if !skills_directory(&claude_home).is_dir() {
        return Err(format!(
            "Codex skill pack is not installed in Claude home: {}",
            display_path(&claude_home)
        ));
    }
    for relative_path in &layout.root_files {
        compare_file_bytes(
            &layout.root_path.join(relative_path),
            &claude_home.join(relative_path),
        )?;
        let _ = writeln!(standard_output, "Content verified for {relative_path}");
    }
    let selected_skills: Vec<&SkillDefinition> = match requested_skill_name {
        Some(skill_name) if !skill_name.trim().is_empty() => layout
            .skills
            .iter()
            .filter(|skill| skill.name == skill_name.trim())
            .collect(),
        _ => layout.skills.iter().collect(),
    };
    if requested_skill_name.is_some() && selected_skills.is_empty() {
        return Err(format!(
            "Skill does not exist in repo: {}",
            requested_skill_name.unwrap_or_default()
        ));
    }
    for skill in selected_skills {
        compare_skill(skill, &claude_home)?;
        let _ = writeln!(
            standard_output,
            "Content verified for skill: {}",
            skill.name
        );
    }
    for agent_name in &layout.agent_names {
        if !agent_profiles_directory(&claude_home)
            .join(format!("{agent_name}.toml"))
            .is_file()
        {
            return Err(format!(
                "Agent profile is not installed in Claude home: {agent_name}"
            ));
        }
    }
    if !installed_executable_path(&claude_home).is_file() {
        return Err(format!(
            "installed Rust executable is missing {}",
            display_path(&installed_executable_path(&claude_home))
        ));
    }
    let _ = writeln!(standard_output, "All Rust verification checks passed");
    Ok(())
}

fn compare_skill(skill: &SkillDefinition, claude_home: &Path) -> Result<(), String> {
    compare_file_bytes(
        &skill.skill_path.join("SKILL.md"),
        &skills_directory(claude_home)
            .join(&skill.name)
            .join("SKILL.md"),
    )?;
    for relative_directory in SKILL_SYNC_DIRECTORIES {
        compare_directory_subset(
            &skill.skill_path.join(relative_directory),
            &skills_directory(claude_home)
                .join(&skill.name)
                .join(relative_directory),
        )?;
    }
    Ok(())
}

fn compare_directory_subset(
    source_directory: &Path,
    target_directory: &Path,
) -> Result<(), String> {
    if !source_directory.is_dir() {
        return Ok(());
    }
    for entry_result in fs::read_dir(source_directory)
        .map_err(|error| format!("read {}: {error}", display_path(source_directory)))?
    {
        let entry = entry_result.map_err(|error| format!("read directory entry: {error}"))?;
        let source_path = entry.path();
        let target_path = target_directory.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            format!("read file type for {}: {error}", display_path(&source_path))
        })?;
        if file_type.is_dir() {
            compare_directory_subset(&source_path, &target_path)?;
        } else if file_type.is_file() {
            compare_file_bytes(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn compare_file_bytes(source_path: &Path, target_path: &Path) -> Result<(), String> {
    let source_bytes = fs::read(source_path)
        .map_err(|error| format!("read source {}: {error}", display_path(source_path)))?;
    let target_bytes = fs::read(target_path)
        .map_err(|error| format!("read target {}: {error}", display_path(target_path)))?;
    if source_bytes != target_bytes {
        return Err(format!(
            "content mismatch for {}",
            display_path(source_path)
        ));
    }
    Ok(())
}

fn count_installed_skills(claude_home: &Path) -> usize {
    fs::read_dir(skills_directory(claude_home))
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| entry.path().join("SKILL.md").is_file())
        .count()
}

fn count_managed_skills(claude_home: &Path) -> Option<usize> {
    let inventory_path = state_directory(claude_home).join("managed-skills.txt");
    if !inventory_path.is_file() {
        return None;
    }
    Some(read_inventory_lines(&inventory_path).len())
}

fn install_metadata_path(claude_home: &Path) -> PathBuf {
    state_directory(claude_home).join("install-metadata.txt")
}

fn read_inventory_lines(path: &Path) -> Vec<String> {
    read_text_if_exists(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn metadata_value<'a>(metadata: &'a str, key: &str) -> Option<&'a str> {
    for line in metadata.lines() {
        let (line_key, line_value) = line.split_once('=')?;
        if line_key == key {
            return Some(line_value);
        }
    }
    None
}

#[derive(Default)]
struct ParsedAgentConfig {
    reasoning_effort: String,
    short_description: String,
    default_prompt: String,
}

fn parse_agent_config(agent_config: &AgentConfig) -> Result<ParsedAgentConfig, String> {
    let text = fs::read_to_string(&agent_config.config_path)
        .map_err(|error| format!("read {}: {error}", display_path(&agent_config.config_path)))?;
    Ok(ParsedAgentConfig {
        reasoning_effort: extract_quoted_yaml_value(&text, "reasoning_effort")
            .unwrap_or_else(|| "high".to_string()),
        short_description: extract_quoted_yaml_value(&text, "short_description")
            .unwrap_or_default(),
        default_prompt: extract_quoted_yaml_value(&text, "default_prompt").ok_or_else(|| {
            format!(
                "missing default_prompt in {}",
                display_path(&agent_config.config_path)
            )
        })?,
    })
}

fn extract_quoted_yaml_value(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(&prefix) {
            continue;
        }
        let value = trimmed[prefix.len()..].trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return decode_basic_json_string(value);
        }
        return Some(value.to_string());
    }
    None
}

fn decode_basic_json_string(value: &str) -> Option<String> {
    let mut bytes = value.as_bytes();
    if bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return None;
    }
    bytes = &bytes[1..bytes.len() - 1];
    let mut decoded = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' {
            decoded.push(bytes[index] as char);
            index += 1;
            continue;
        }
        index += 1;
        if index >= bytes.len() {
            return None;
        }
        match bytes[index] {
            b'"' => decoded.push('"'),
            b'\\' => decoded.push('\\'),
            b'n' => decoded.push('\n'),
            b'r' => decoded.push('\r'),
            b't' => decoded.push('\t'),
            other => decoded.push(other as char),
        }
        index += 1;
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_root_inference_prefers_complete_candidate() {
        let root = create_minimal_layout("claude-skills-install-root");
        let nested = root.join("docs").join("nested");
        fs::create_dir_all(&nested).unwrap();

        let resolved = resolve_install_repository_root_from_candidates(vec![nested]).unwrap();

        assert_eq!(resolved, root);
    }

    #[test]
    fn install_root_inference_uses_executable_parent_style_candidate() {
        let root = create_minimal_layout("claude-skills-release-root");
        let outside =
            std::env::temp_dir().join(format!("claude-skills-outside-{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();

        let resolved =
            resolve_install_repository_root_from_candidates(vec![outside.clone(), root.clone()])
                .unwrap();

        assert_eq!(resolved, root);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn agent_profile_render_omits_repo_model_pin() {
        let root = create_minimal_layout("claude-skills-agent-profile");
        let config_path = root.join("reviewer").join("agents").join("openai.yaml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(
            &config_path,
            format!(
                "{}: \"repo-value\"\nreasoning_effort: \"high\"\nshort_description: \"Reviewer\"\ndefault_prompt: \"Review the change\"\n",
                "model"
            ),
        )
        .unwrap();

        let parsed = parse_agent_config(&AgentConfig {
            agent_name: "reviewer".to_string(),
            config_path,
        })
        .unwrap();
        let rendered = render_agent_toml(&parsed, "reviewer").unwrap();

        assert!(!rendered.lines().any(|line| line.starts_with("model = ")));
        assert!(rendered.contains("name = \"reviewer\""));
        assert!(rendered.contains("model_reasoning_effort = \"high\""));
        assert!(rendered.contains("description = \"Reviewer\""));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn install_removes_deprecated_legacy_landlock_config_key() {
        let claude_home =
            std::env::temp_dir().join(format!("claude-skills-config-clean-{}", std::process::id()));
        let _ = fs::remove_dir_all(&claude_home);
        fs::create_dir_all(&claude_home).unwrap();
        fs::write(
            claude_home.join("config.toml"),
            "model = \"gpt-5\"\n\n[features]\nhooks = true\nuse_legacy_landlock = true\nsqlite = false\n\n[profiles.default]\nmodel = \"gpt-5\"\n",
        )
        .unwrap();

        let mut stats = SyncStats::default();
        remove_deprecated_config_keys(&claude_home, &mut stats).unwrap();
        let updated = fs::read_to_string(claude_home.join("config.toml")).unwrap();

        assert!(!updated.contains("use_legacy_landlock"));
        assert!(updated.contains("hooks = true"));
        assert!(updated.contains("[profiles.default]"));
        assert_eq!(stats.copied_files, 1);
        let _ = fs::remove_dir_all(claude_home);
    }

    #[test]
    fn repo_version_recovers_bootstrap_commit_from_build_version() {
        assert_eq!(
            repo_version_from_build_version("bootstrap-8c0eb1cf6c20").as_deref(),
            Some("8c0eb1c")
        );
        assert_eq!(repo_version_from_build_version("dev"), None);
        assert_eq!(
            repo_version_from_build_version("bootstrap-sample-release"),
            None
        );
    }

    #[test]
    fn repo_version_recovers_bootstrap_commit_from_installed_metadata() {
        let metadata = "repo_version=unknown\nmanager_version=bootstrap-8c0eb1cf6c20\n";
        assert_eq!(
            repo_version_from_metadata_or_build(metadata, "dev").as_deref(),
            Some("8c0eb1c")
        );
    }

    fn create_minimal_layout(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("reviewer")).unwrap();
        fs::write(root.join("AGENTS.md"), "").unwrap();
        fs::write(root.join("README.md"), "").unwrap();
        fs::write(root.join("00-skill-routing-and-escalation.md"), "").unwrap();
        fs::write(root.join("reviewer").join("SKILL.md"), "").unwrap();
        root
    }
}

fn render_agent_toml(config: &ParsedAgentConfig, agent_name: &str) -> Result<String, String> {
    if config.default_prompt.contains("'''") {
        return Err(format!(
            "triple single quotes are not supported inside developer_instructions for {agent_name}"
        ));
    }
    let mut lines = Vec::new();
    lines.push(format!("name = \"{}\"", escape_toml_string(agent_name)));
    lines.push(format!(
        "model_reasoning_effort = \"{}\"",
        escape_toml_string(&config.reasoning_effort)
    ));
    if !config.short_description.trim().is_empty() {
        lines.push(format!(
            "description = \"{}\"",
            escape_toml_string(&config.short_description)
        ));
    }
    lines.push("developer_instructions = '''".to_string());
    lines.push(config.default_prompt.clone());
    lines.push("'''".to_string());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
