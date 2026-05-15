//! Purpose: System map rendering and workspace structure detection
//! Caller: memory.rs scope/system-map commands
//! Dependencies: std::fs, std::path, crate::runtime
//! Main Functions: render_system_map, sanitize_key
//! Side Effects: Reads workspace directory structure

use std::fs;
use std::path::{Path, PathBuf};

use crate::runtime::display_path;

pub fn render_system_map(workspace_root: &Path) -> String {
    let top_level_entries = workspace_entries(workspace_root);
    let applications = detect_applications(workspace_root);
    let entrypoints = detect_entrypoints(workspace_root);
    let ownership_hints = detect_ownership_hints(workspace_root);
    let mut lines = vec![
        "# SYSTEM_MAP".to_string(),
        String::new(),
        format!("- workspace_root: {}", display_path(workspace_root)),
        "- storage: Claude Code-global per-workspace reference lane".to_string(),
        "- runtime: rust".to_string(),
        "- go_fallback: false".to_string(),
        String::new(),
        "## Top-Level Entries".to_string(),
    ];
    if top_level_entries.is_empty() {
        lines.push("- Not found".to_string());
    } else {
        for entry in &top_level_entries {
            lines.push(format!("- {} ({})", entry.name, entry.kind));
        }
    }
    lines.push(String::new());
    lines.push("## Direct Child Structure".to_string());
    append_direct_child_structure(workspace_root, &top_level_entries, &mut lines);
    lines.push(String::new());
    lines.push("## Applications".to_string());
    append_list_or_not_found(&mut lines, applications);
    lines.push(String::new());
    lines.push("## Entrypoints".to_string());
    append_list_or_not_found(&mut lines, entrypoints);
    lines.push(String::new());
    lines.push("## Ownership Hints".to_string());
    append_list_or_not_found(&mut lines, ownership_hints);
    lines.push(String::new());
    lines.push("## Maintenance".to_string());
    lines.push(
        "- Refresh this map after creating, deleting, moving, or renaming files or folders."
            .to_string(),
    );
    lines.push("- Command: `claude-skills memory system-map refresh`".to_string());
    lines.push(String::new());
    lines.join("\n")
}

pub fn sanitize_key(value: &str) -> String {
    let raw_key = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    collapse_separator_runs(&raw_key)
}

fn collapse_separator_runs(value: &str) -> String {
    let mut collapsed = String::new();
    let mut previous_was_separator = false;
    for character in value.chars() {
        if character == '-' {
            if !previous_was_separator {
                collapsed.push(character);
            }
            previous_was_separator = true;
        } else {
            collapsed.push(character);
            previous_was_separator = false;
        }
    }
    collapsed
}

struct WorkspaceEntry {
    name: String,
    kind: &'static str,
    path: PathBuf,
}

fn workspace_entries(workspace_root: &Path) -> Vec<WorkspaceEntry> {
    let mut entries = Vec::new();
    if let Ok(read_directory) = fs::read_dir(workspace_root) {
        for entry_result in read_directory.flatten() {
            let name = entry_result.file_name().to_string_lossy().to_string();
            let path = entry_result.path();
            if should_skip_workspace_entry(&name, &path) {
                continue;
            }
            let kind = if path.is_dir() { "dir" } else { "file" };
            entries.push(WorkspaceEntry { name, kind, path });
        }
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries.into_iter().take(200).collect()
}

fn append_direct_child_structure(
    workspace_root: &Path,
    entries: &[WorkspaceEntry],
    lines: &mut Vec<String>,
) {
    let mut rendered_any = false;
    for entry in entries.iter().filter(|entry| entry.path.is_dir()).take(60) {
        let mut child_dirs = Vec::new();
        let mut child_files = Vec::new();
        if let Ok(children) = fs::read_dir(&entry.path) {
            for child_result in children.flatten() {
                let child_name = child_result.file_name().to_string_lossy().to_string();
                let child_path = child_result.path();
                if should_skip_workspace_entry(&child_name, &child_path) {
                    continue;
                }
                if child_path.is_dir() {
                    child_dirs.push(format!("`{child_name}/`"));
                } else if child_path.is_file() && !is_probably_binary(&child_path) {
                    child_files.push(format!("`{child_name}`"));
                }
            }
        }
        child_dirs.sort();
        child_files.sort();
        let relative_path = entry
            .path
            .strip_prefix(workspace_root)
            .unwrap_or(&entry.path);
        lines.push(format!(
            "- `{}/` -> dirs: {}; files: {}.",
            markdown_path(relative_path),
            summarize_names(&child_dirs),
            summarize_names(&child_files)
        ));
        rendered_any = true;
    }
    if !rendered_any {
        lines.push("- Not found".to_string());
    }
}

fn detect_applications(workspace_root: &Path) -> Vec<String> {
    let mut applications = Vec::new();
    for marker in [
        ("Cargo.toml", "Rust workspace/package"),
        ("package.json", "JavaScript package"),
        ("go.mod", "Go module"),
        ("pyproject.toml", "Python project"),
        ("pom.xml", "Maven project"),
        ("build.gradle", "Gradle project"),
        ("terraform.tf", "Terraform root"),
    ] {
        if workspace_root.join(marker.0).is_file() {
            applications.push(format!("- `{}` - {}", marker.0, marker.1));
        }
    }
    for cargo_manifest in collect_matching_relative_paths(workspace_root, "Cargo.toml", 4, 40) {
        if cargo_manifest != "Cargo.toml" {
            applications.push(format!("- `{cargo_manifest}` - Rust crate"));
        }
    }
    applications.sort();
    applications.dedup();
    applications
}

fn detect_entrypoints(workspace_root: &Path) -> Vec<String> {
    let mut entrypoints = Vec::new();
    for relative_path in [
        "src/main.rs",
        "rust/crates/claude-skills/src/main.rs",
        "cmd/claude-skills/main.go",
        "main.go",
        "index.js",
        "src/index.ts",
    ] {
        if workspace_root.join(relative_path).is_file() {
            entrypoints.push(format!("- `{relative_path}`"));
        }
    }
    for relative_path in collect_matching_relative_paths(workspace_root, "main.rs", 6, 40) {
        entrypoints.push(format!("- `{relative_path}`"));
    }
    entrypoints.sort();
    entrypoints.dedup();
    entrypoints
}

fn detect_ownership_hints(workspace_root: &Path) -> Vec<String> {
    let mut hints = Vec::new();
    if workspace_root.join("AGENTS.md").is_file() {
        hints.push("- `AGENTS.md` - managed agent routing and repository policy".to_string());
    }
    if workspace_root.join("README.md").is_file() {
        hints.push("- `README.md` - user-facing product and install surface".to_string());
    }
    for entry in workspace_entries(workspace_root) {
        if entry.path.is_dir() && entry.path.join("SKILL.md").is_file() {
            hints.push(format!(
                "- `{}/SKILL.md` - specialist skill surface",
                entry.name
            ));
        }
    }
    hints
}

fn collect_matching_relative_paths(
    workspace_root: &Path,
    file_name: &str,
    max_depth: usize,
    max_results: usize,
) -> Vec<String> {
    let mut matches = Vec::new();
    collect_matching_relative_paths_inner(
        workspace_root,
        workspace_root,
        file_name,
        0,
        max_depth,
        max_results,
        &mut matches,
    );
    matches.sort();
    matches
}

fn collect_matching_relative_paths_inner(
    workspace_root: &Path,
    directory: &Path,
    file_name: &str,
    depth: usize,
    max_depth: usize,
    max_results: usize,
    matches: &mut Vec<String>,
) {
    if depth > max_depth || matches.len() >= max_results {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry_result in entries.flatten() {
        let child_name = entry_result.file_name().to_string_lossy().to_string();
        let child_path = entry_result.path();
        if should_skip_workspace_entry(&child_name, &child_path) {
            continue;
        }
        if child_path.is_dir() {
            collect_matching_relative_paths_inner(
                workspace_root,
                &child_path,
                file_name,
                depth + 1,
                max_depth,
                max_results,
                matches,
            );
        } else if child_path.is_file() && child_name == file_name {
            let relative_path = child_path
                .strip_prefix(workspace_root)
                .unwrap_or(&child_path);
            matches.push(markdown_path(relative_path));
            if matches.len() >= max_results {
                return;
            }
        }
    }
}

fn append_list_or_not_found(lines: &mut Vec<String>, values: Vec<String>) {
    if values.is_empty() {
        lines.push("- Not found".to_string());
    } else {
        lines.extend(values);
    }
}

fn summarize_names(names: &[String]) -> String {
    if names.is_empty() {
        return "Not found".to_string();
    }
    let mut rendered = names
        .iter()
        .take(12)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > 12 {
        rendered.push_str(&format!(", ... {} more", names.len() - 12));
    }
    rendered
}

fn markdown_path(path: &Path) -> String {
    display_path(path).replace('\\', "/")
}

fn should_skip_workspace_entry(name: &str, path: &Path) -> bool {
    matches!(
        name,
        ".git" | ".claude" | "target" | "node_modules" | "vendor"
    ) || (name.starts_with('.') && path.is_dir() && !matches!(name, ".github" | ".gitlab"))
        || (path.is_file() && is_probably_binary(path))
}

fn is_probably_binary(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or(""),
        "exe" | "dll" | "png" | "jpg" | "jpeg" | "gif" | "zip" | "gz" | "tar" | "lock"
    )
}
