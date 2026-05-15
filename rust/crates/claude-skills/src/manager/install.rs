//! Purpose: Install, sync, update, and uninstall logic for claude-skills manager.
//! Caller: commands.rs via run_install_command, run_update_command, run_uninstall_command.
//! Dependencies: std::fs, std::io, std::path, std::process, std::thread, std::time, claude_skills_platform, crate::args, crate::runtime.
//! Main Functions: install_from_flags, install_from_paths, sync_root_files, sync_skills, sync_agents, publish_native_executable, run_update_command, run_uninstall_command.
//! Side Effects: Copies managed skill-pack files, writes Claude home config/state, publishes the Rust binary, runs git commands, and removes managed files during uninstall.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use claude_skills_platform::detect_current_target;

use crate::args::FlagSet;
use crate::runtime::{
    agent_profiles_directory, agents_directory, config_path, discover_repository_layout,
    display_path, git_short_head, installed_executable_path, read_text_if_exists,
    remove_path_if_exists, repository_layout_is_complete, resolve_claude_home,
    resolve_repository_root, run_command, skills_directory, state_directory, write_lines,
    write_text, RepositoryLayout, SKILL_SYNC_DIRECTORIES,
};

use super::agent_config::{parse_agent_config, render_agent_toml, unix_timestamp};

#[derive(Default)]
pub struct InstallSummary {
    pub synced_skills: usize,
    pub synced_agents: usize,
    pub synced_root_files: usize,
    pub removed_stale_files: usize,
    pub published_executable: bool,
}

#[derive(Default)]
struct SyncStats {
    created: usize,
    updated: usize,
    unchanged: usize,
    #[allow(dead_code)]
    removed: usize,
}

pub fn install_from_flags(
    build_version: &str,
    flag_set: &FlagSet,
) -> Result<InstallSummary, String> {
    let repository_root = resolve_install_repository_root(flag_set.string_value("repo-root"))?;
    let claude_home = resolve_claude_home(flag_set.string_value("claude-home"))?;
    install_from_paths(build_version, &repository_root, &claude_home)
}

pub fn resolve_install_repository_root(flag_value: &str) -> Result<PathBuf, String> {
    if !flag_value.trim().is_empty() {
        return resolve_repository_root(flag_value);
    }
    let candidates = [
        std::env::current_dir().ok(),
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf)),
    ];
    resolve_install_repository_root_from_candidates(&candidates)
}

pub fn resolve_install_repository_root_from_candidates(
    candidates: &[Option<PathBuf>],
) -> Result<PathBuf, String> {
    for candidate in candidates.iter().flatten() {
        if repository_layout_is_complete(candidate) {
            return Ok(candidate.clone());
        }
    }
    Err("Repository root not found. Use --repo-root to specify the path.".to_string())
}

pub fn install_from_paths(
    build_version: &str,
    repository_root: &Path,
    claude_home: &Path,
) -> Result<InstallSummary, String> {
    let layout = discover_repository_layout(repository_root)?;
    ensure_claude_home_directories(claude_home)?;
    let removed_stale_files = remove_stale_managed_files(claude_home)?;
    remove_deprecated_config_keys(claude_home)?;
    let synced_root_files = sync_root_files(&layout, claude_home)?;
    let synced_skills = sync_skills(&layout, claude_home)?;
    let synced_agents = sync_agents(&layout, claude_home)?;
    write_managed_config(claude_home)?;
    let published_executable = publish_native_executable(repository_root, claude_home)?;
    write_install_metadata(build_version, repository_root, claude_home)?;
    write_inventories(&layout, claude_home)?;
    Ok(InstallSummary {
        synced_skills,
        synced_agents,
        synced_root_files,
        removed_stale_files,
        published_executable,
    })
}

pub fn write_install_summary(summary: &InstallSummary, output: &mut dyn Write) {
    let _ = writeln!(output, "Native Rust install complete");
    let _ = writeln!(output);
    let _ = writeln!(output, "Summary:");
    let _ = writeln!(output, "  Synced skills: {}", summary.synced_skills);
    let _ = writeln!(output, "  Synced agents: {}", summary.synced_agents);
    let _ = writeln!(output, "  Synced root files: {}", summary.synced_root_files);
    let _ = writeln!(
        output,
        "  Removed stale files: {}",
        summary.removed_stale_files
    );
    let _ = writeln!(
        output,
        "  Published executable: {}",
        summary.published_executable
    );
}

fn ensure_claude_home_directories(claude_home: &Path) -> Result<(), String> {
    for directory in [
        claude_home,
        &skills_directory(claude_home),
        &agents_directory(claude_home),
        &agent_profiles_directory(claude_home),
        &state_directory(claude_home),
    ] {
        fs::create_dir_all(directory)
            .map_err(|error| format!("create {}: {error}", display_path(directory)))?;
    }
    Ok(())
}

fn remove_stale_managed_files(claude_home: &Path) -> Result<usize, String> {
    let mut removed_count = 0;
    let inventory_path = state_directory(claude_home).join("managed-skills.txt");
    let installed_skills: Vec<String> = super::verify::read_inventory_lines(&inventory_path);
    for skill_name in installed_skills {
        let skill_path = skills_directory(claude_home).join(&skill_name);
        if !skill_path.is_dir() {
            continue;
        }
        removed_count += remove_path_if_exists_counted(&skill_path)?;
    }
    let inventory_path = state_directory(claude_home).join("managed-agents.txt");
    let installed_agents: Vec<String> = super::verify::read_inventory_lines(&inventory_path);
    for agent_name in installed_agents {
        let agent_path = agents_directory(claude_home).join(&agent_name);
        removed_count += remove_path_if_exists_counted(&agent_path)?;
        let profile_path = agent_profiles_directory(claude_home).join(format!("{agent_name}.toml"));
        removed_count += remove_path_if_exists_counted(&profile_path)?;
    }
    Ok(removed_count)
}

fn remove_deprecated_config_keys(claude_home: &Path) -> Result<(), String> {
    let config_file = config_path(claude_home);
    if !config_file.is_file() {
        return Ok(());
    }
    let original_text = read_text_if_exists(&config_file).unwrap_or_default();
    let updated_text = remove_managed_block(&original_text);
    if updated_text != original_text {
        write_text(&config_file, &updated_text)?;
    }
    Ok(())
}

fn copy_file_if_changed(source: &Path, target: &Path) -> Result<bool, String> {
    if target.is_file() {
        let source_bytes =
            fs::read(source).map_err(|error| format!("read {}: {error}", display_path(source)))?;
        let target_bytes =
            fs::read(target).map_err(|error| format!("read {}: {error}", display_path(target)))?;
        if source_bytes == target_bytes {
            return Ok(false);
        }
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", display_path(parent)))?;
    }
    fs::copy(source, target).map_err(|error| {
        format!(
            "copy {} to {}: {error}",
            display_path(source),
            display_path(target)
        )
    })?;
    Ok(true)
}

fn write_text_if_changed(path: &Path, content: &str) -> Result<bool, String> {
    if path.is_file() {
        let existing = read_text_if_exists(path).unwrap_or_default();
        if existing == content {
            return Ok(false);
        }
    }
    write_text(path, content)?;
    Ok(true)
}

fn sync_directory_delta(
    source_directory: &Path,
    target_directory: &Path,
    stats: &mut SyncStats,
) -> Result<(), String> {
    if !source_directory.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(target_directory)
        .map_err(|error| format!("create {}: {error}", display_path(target_directory)))?;
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
            sync_directory_delta(&source_path, &target_path, stats)?;
        } else if file_type.is_file() {
            if target_path.is_file() {
                if copy_file_if_changed(&source_path, &target_path)? {
                    stats.updated += 1;
                } else {
                    stats.unchanged += 1;
                }
            } else {
                copy_file_if_changed(&source_path, &target_path)?;
                stats.created += 1;
            }
        }
    }
    Ok(())
}

fn remove_path_if_exists_counted(path: &Path) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }
    remove_path_if_exists(path)?;
    Ok(1)
}

fn sync_root_files(layout: &RepositoryLayout, claude_home: &Path) -> Result<usize, String> {
    let mut synced_count = 0;
    for root_file_name in &layout.root_files {
        let source_path = layout.root_path.join(root_file_name);
        let target_path = claude_home.join(root_file_name);
        if copy_file_if_changed(&source_path, &target_path)? {
            synced_count += 1;
        }
    }
    Ok(synced_count)
}

fn sync_skills(layout: &RepositoryLayout, claude_home: &Path) -> Result<usize, String> {
    let mut synced_count = 0;
    for skill in &layout.skills {
        let target_skill_directory = skills_directory(claude_home).join(&skill.name);
        let target_skill_file = target_skill_directory.join("SKILL.md");
        if copy_file_if_changed(&skill.skill_path.join("SKILL.md"), &target_skill_file)? {
            synced_count += 1;
        }
        for relative_directory in SKILL_SYNC_DIRECTORIES {
            let source_directory = skill.skill_path.join(relative_directory);
            let target_directory = target_skill_directory.join(relative_directory);
            let mut stats = SyncStats::default();
            sync_directory_delta(&source_directory, &target_directory, &mut stats)?;
        }
    }
    Ok(synced_count)
}

fn sync_agents(layout: &RepositoryLayout, claude_home: &Path) -> Result<usize, String> {
    let mut synced_count = 0;
    for skill in &layout.skills {
        for agent_config in &skill.agent_configs {
            let parsed = parse_agent_config(agent_config)?;
            let toml_content = render_agent_toml(&parsed, &agent_config.agent_name)?;
            let target_path = agent_profiles_directory(claude_home)
                .join(format!("{}.toml", agent_config.agent_name));
            if write_text_if_changed(&target_path, &toml_content)? {
                synced_count += 1;
            }
        }
    }
    Ok(synced_count)
}

fn write_managed_config(claude_home: &Path) -> Result<(), String> {
    let config_file = config_path(claude_home);
    let original_text = read_text_if_exists(&config_file).unwrap_or_default();
    let cleaned_text = remove_managed_block(&original_text);
    let managed_block = format!(
        "# BEGIN MANAGED BLOCK ({})\n# END MANAGED BLOCK\n",
        unix_timestamp()
    );
    let updated_text = if cleaned_text.trim().is_empty() {
        managed_block
    } else {
        format!("{}\n{}", cleaned_text.trim_end(), managed_block)
    };
    write_text(&config_file, &updated_text)?;
    Ok(())
}

fn remove_managed_block(text: &str) -> String {
    let mut lines = Vec::new();
    let mut inside_block = false;
    for line in text.lines() {
        if line.starts_with("# BEGIN MANAGED BLOCK") {
            inside_block = true;
            continue;
        }
        if line.starts_with("# END MANAGED BLOCK") {
            inside_block = false;
            continue;
        }
        if !inside_block {
            lines.push(line);
        }
    }
    lines.join("\n")
}

pub fn publish_native_executable(
    repository_root: &Path,
    claude_home: &Path,
) -> Result<bool, String> {
    let target = detect_current_target().map_err(|error| format!("detect target: {error}"))?;
    let source_path = repository_root
        .join("target")
        .join(target.directory_name())
        .join("release")
        .join(executable_file_name());
    if !source_path.is_file() {
        return Ok(false);
    }
    let target_path = installed_executable_path(claude_home);
    atomic_copy_executable(&source_path, &target_path)?;
    Ok(true)
}

fn atomic_copy_executable(source: &Path, target: &Path) -> Result<(), String> {
    if cfg!(windows) {
        publish_specific_executable(source, target)?;
    } else {
        let temp_path = target.with_extension("tmp");
        publish_specific_executable(source, &temp_path)?;
        fs::rename(&temp_path, target).map_err(|error| {
            format!(
                "rename {} to {}: {error}",
                display_path(&temp_path),
                display_path(target)
            )
        })?;
    }
    Ok(())
}

fn publish_specific_executable(source: &Path, target: &Path) -> Result<(), String> {
    if cfg!(windows) && target.is_file() {
        spawn_self_replace(source, target)?;
    } else {
        fs::copy(source, target).map_err(|error| {
            format!(
                "copy {} to {}: {error}",
                display_path(source),
                display_path(target)
            )
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(target)
                .map_err(|error| format!("read metadata for {}: {error}", display_path(target)))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(target, permissions).map_err(|error| {
                format!("set permissions for {}: {error}", display_path(target))
            })?;
        }
    }
    Ok(())
}

fn spawn_self_replace(source: &Path, target: &Path) -> Result<(), String> {
    let command_line = build_self_replace_command(source, target);
    Command::new("cmd")
        .args(["/C", &command_line])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("spawn self-replace: {error}"))?;
    thread::sleep(Duration::from_millis(100));
    Ok(())
}

fn build_self_replace_command(source: &Path, target: &Path) -> String {
    format!(
        "timeout /t 1 /nobreak >nul && copy /y \"{}\" \"{}\" >nul",
        source.display(),
        target.display()
    )
}

fn executable_file_name() -> String {
    if cfg!(windows) {
        "claude-skills.exe".to_string()
    } else {
        "claude-skills".to_string()
    }
}

fn write_install_metadata(
    build_version: &str,
    repository_root: &Path,
    claude_home: &Path,
) -> Result<(), String> {
    let repo_version = repo_version_for_source(build_version, repository_root);
    let manager_version = format!("{}-{}", build_version, git_short_head(repository_root));
    let metadata = format!("repo_version={repo_version}\nmanager_version={manager_version}\n");
    write_text(
        &super::verify::install_metadata_path(claude_home),
        &metadata,
    )?;
    Ok(())
}

pub fn repo_version_for_source(build_version: &str, repository_root: &Path) -> String {
    meaningful_repo_version(build_version).unwrap_or_else(|| git_short_head(repository_root))
}

pub fn repo_version_from_metadata_or_build(metadata: &str, build_version: &str) -> Option<String> {
    super::verify::metadata_value(metadata, "repo_version")
        .filter(|value| *value != "unknown")
        .map(str::to_string)
        .or_else(|| {
            super::verify::metadata_value(metadata, "manager_version")
                .and_then(repo_version_from_build_version)
        })
        .or_else(|| meaningful_repo_version(build_version))
}

fn meaningful_repo_version(build_version: &str) -> Option<String> {
    if build_version == "dev" || build_version.is_empty() {
        return None;
    }
    Some(build_version.to_string())
}

fn repo_version_from_build_version(manager_version: &str) -> Option<String> {
    let commit_hash = manager_version.split('-').next_back()?;
    if commit_hash.len() >= 7 {
        Some(commit_hash[..7].to_string())
    } else {
        None
    }
}

fn write_inventories(layout: &RepositoryLayout, claude_home: &Path) -> Result<(), String> {
    let skill_names: Vec<String> = layout.skills.iter().map(|s| s.name.clone()).collect();
    write_lines(
        &state_directory(claude_home).join("managed-skills.txt"),
        &skill_names,
    )?;
    write_lines(
        &state_directory(claude_home).join("managed-agents.txt"),
        &layout.agent_names,
    )?;
    Ok(())
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
    if let Err(parse_error) = flag_set.parse(arguments) {
        let _ = writeln!(standard_error, "{}", parse_error.message);
        return 1;
    }
    let repository_root = match resolve_update_repository_root(flag_set.string_value("repo-root")) {
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
    let current_branch =
        current_git_branch(&repository_root).unwrap_or_else(|_| "main".to_string());
    let _ = writeln!(
        standard_output,
        "Updating repository from origin/{current_branch}"
    );
    if let Err(error) = run_command(
        "git",
        &[
            "pull".to_string(),
            "origin".to_string(),
            current_branch.clone(),
        ],
        Some(&repository_root),
    ) {
        let _ = writeln!(standard_error, "git pull failed: {error}");
        return 1;
    }
    let _ = writeln!(standard_output, "Building native Rust executable");
    let build_result = run_command(
        "cargo",
        &[
            "build".to_string(),
            "--release".to_string(),
            "--bin".to_string(),
            "claude-skills".to_string(),
        ],
        Some(&repository_root),
    );
    if let Err(error) = build_result {
        let _ = writeln!(standard_error, "cargo build failed: {error}");
        return 1;
    }
    let _ = writeln!(standard_output, "Installing updated skill pack");
    match install_from_paths(build_version, &repository_root, &claude_home) {
        Ok(summary) => {
            write_install_summary(&summary, standard_output);
            0
        }
        Err(error) => {
            let _ = writeln!(standard_error, "install failed: {error}");
            1
        }
    }
}

fn resolve_update_repository_root(flag_value: &str) -> Result<PathBuf, String> {
    if !flag_value.trim().is_empty() {
        return resolve_repository_root(flag_value);
    }
    match std::env::current_dir() {
        Ok(path) if repository_layout_is_complete(&path) => Ok(path),
        _ => Err("Repository root not found. Use --repo-root to specify the path.".to_string()),
    }
}

fn current_git_branch(repository_root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repository_root)
        .output()
        .map_err(|error| format!("run git: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse failed".to_string());
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|error| format!("parse git output: {error}"))
}

pub fn run_uninstall_command(
    arguments: &[String],
    standard_output: &mut dyn Write,
    standard_error: &mut dyn Write,
) -> u8 {
    let mut flag_set = FlagSet::new("uninstall");
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
    let mut removed_count = 0;
    match remove_stale_managed_files(&claude_home) {
        Ok(count) => removed_count += count,
        Err(error) => {
            let _ = writeln!(standard_error, "remove managed files failed: {error}");
            return 1;
        }
    }
    for root_file_name in ["AGENTS.md", "README.md"] {
        let path = claude_home.join(root_file_name);
        match remove_path_if_exists_counted(&path) {
            Ok(count) => removed_count += count,
            Err(error) => {
                let _ = writeln!(standard_error, "remove {root_file_name} failed: {error}");
                return 1;
            }
        }
    }
    let executable_path = installed_executable_path(&claude_home);
    match remove_path_if_exists_counted(&executable_path) {
        Ok(count) => removed_count += count,
        Err(error) => {
            let _ = writeln!(standard_error, "remove executable failed: {error}");
            return 1;
        }
    }
    if let Err(error) = remove_deprecated_config_keys(&claude_home) {
        let _ = writeln!(
            standard_error,
            "remove deprecated config keys failed: {error}"
        );
        return 1;
    }
    let _ = writeln!(standard_output, "Uninstall complete");
    let _ = writeln!(standard_output, "  Removed files: {removed_count}");
    0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_install_repository_root_prefers_current_directory() {
        let root = create_minimal_layout("resolve-install-repo-root");
        let result = resolve_install_repository_root_from_candidates(&[Some(root.clone())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), root);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_install_repository_root_falls_back_to_executable_parent() {
        let root = create_minimal_layout("resolve-install-repo-root-fallback");
        let result = resolve_install_repository_root_from_candidates(&[None, Some(root.clone())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), root);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_install_repository_root_fails_when_no_candidate_is_complete() {
        let result = resolve_install_repository_root_from_candidates(&[None, None]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Repository root not found"));
    }

    #[test]
    fn remove_managed_block_removes_block_from_config() {
        let text =
            "key=value\n# BEGIN MANAGED BLOCK (123)\nold=data\n# END MANAGED BLOCK\nother=line\n";
        let result = remove_managed_block(text);
        assert_eq!(result, "key=value\nother=line");
    }

    #[test]
    fn remove_managed_block_preserves_text_without_block() {
        let text = "key=value\nother=line\n";
        let result = remove_managed_block(text);
        // lines().join("\n") drops the trailing newline; that is expected behavior
        assert_eq!(result, "key=value\nother=line");
    }

    #[test]
    fn repo_version_prefers_meaningful_build_version() {
        let root = create_minimal_layout("repo-version-build");
        let result = repo_version_for_source("1.2.3", &root);
        assert_eq!(result, "1.2.3");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repo_version_falls_back_to_git_short_head() {
        let root = create_minimal_layout("repo-version-git");
        let result = repo_version_for_source("dev", &root);
        assert!(!result.is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repo_version_recovers_from_installed_metadata() {
        let metadata = "repo_version=1.2.3\nmanager_version=dev-abc123\n";
        assert_eq!(
            repo_version_from_metadata_or_build(metadata, "dev").as_deref(),
            Some("1.2.3")
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
