//! Purpose: Load optional project-specific declarative filters for the proxy.
//! Caller: proxy::adapters registry builder after built-in Rust adapters.
//! Dependencies: CommandAdapter contract and local TOML/YAML-like filter files.
//! Main Functions: load_project_filter_adapters.
//! Side Effects: Reads optional filter files from the current workspace.

use crate::adapters::common::{make_result, normalized_command};
use crate::proxy::adapter::{CommandAdapter, CompactResult};
use crate::proxy::command_ast::CommandAst;
use crate::proxy::raw_store::RunMeta;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct DeclarativeFilter {
    name: String,
    command: String,
    exit_code: Option<i32>,
    keep: Vec<String>,
    max_lines: usize,
}

pub struct ProjectFilterAdapter {
    filter: DeclarativeFilter,
}

impl CommandAdapter for ProjectFilterAdapter {
    fn name(&self) -> &'static str {
        "project-filter"
    }

    fn matches(&self, ast: &CommandAst) -> bool {
        let normalized = normalized_command(&ast.program, &ast.args);
        !self.filter.command.is_empty()
            && (ast.original_command.starts_with(&self.filter.command)
                || normalized.starts_with(&self.filter.command)
                || ast.program.eq_ignore_ascii_case(&self.filter.command)
                || ast.program.ends_with(&self.filter.command))
    }

    fn compact(
        &self,
        stdout: &[u8],
        stderr: &[u8],
        exit_code: i32,
        meta: &RunMeta,
    ) -> CompactResult {
        if matches!(self.filter.exit_code, Some(expected) if expected != exit_code) {
            return make_result(
                self.name(),
                normalized_command(&meta.program, &meta.args),
                String::from_utf8_lossy(stdout).to_string(),
                String::from_utf8_lossy(stderr).to_string(),
                exit_code,
                meta,
                false,
            );
        }
        let merged = format!(
            "{}\n{}",
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(stderr)
        );
        let mut kept = Vec::new();
        for line in merged.lines() {
            let normalized = line.to_ascii_lowercase();
            if self
                .filter
                .keep
                .iter()
                .any(|needle| normalized.contains(&needle.to_ascii_lowercase()))
            {
                kept.push(line.trim().to_string());
            }
            if kept.len() >= self.filter.max_lines.max(1) {
                break;
            }
        }
        let rendered = if kept.is_empty() {
            format!("filter {} matched; no keep lines found", self.filter.name)
        } else {
            kept.join("\n")
        };
        make_result(
            self.name(),
            format!("filter {}", self.filter.name),
            rendered,
            String::new(),
            exit_code,
            meta,
            true,
        )
    }
}

pub fn load_project_filter_adapters() -> Vec<Box<dyn CommandAdapter>> {
    let mut adapters: Vec<Box<dyn CommandAdapter>> = Vec::new();
    for path in filter_paths() {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for filter in parse_filters(&text) {
            adapters.push(Box::new(ProjectFilterAdapter { filter }));
        }
    }
    adapters
}

fn filter_paths() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap_or_default();
    vec![
        cwd.join(".claude-skills").join("filters.toml"),
        cwd.join(".claude-skills").join("filters.yaml"),
        cwd.join(".claude-skills").join("filters.yml"),
        cwd.join("claude-skills.filters.toml"),
    ]
}

fn parse_filters(text: &str) -> Vec<DeclarativeFilter> {
    let mut filters = Vec::new();
    let mut current = DeclarativeFilter {
        max_lines: 40,
        ..DeclarativeFilter::default()
    };
    let mut in_filter = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[filters]]" || line == "- filter:" || line == "filters:" {
            if in_filter && !current.name.is_empty() && !current.command.is_empty() {
                filters.push(current.clone());
            }
            current = DeclarativeFilter {
                max_lines: 40,
                ..DeclarativeFilter::default()
            };
            in_filter = true;
            continue;
        }
        if let Some((key, value)) = split_key_value(line) {
            match key {
                "name" => current.name = strip_quotes(value).to_string(),
                "command" => current.command = strip_quotes(value).to_string(),
                "exit_code" | "exitCode" => current.exit_code = strip_quotes(value).parse().ok(),
                "max_lines" | "maxLines" => {
                    current.max_lines = strip_quotes(value).parse().unwrap_or(40)
                }
                "keep" => current.keep = parse_keep(value),
                _ => {}
            }
        } else if line.starts_with('"') || line.starts_with('-') {
            let value = strip_quotes(line.trim_start_matches('-').trim());
            if !value.is_empty() {
                current.keep.push(value.to_string());
            }
        }
    }
    if in_filter && !current.name.is_empty() && !current.command.is_empty() {
        filters.push(current);
    }
    filters
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    if let Some(index) = line.find('=') {
        return Some((line[..index].trim(), line[index + 1..].trim()));
    }
    if let Some(index) = line.find(':') {
        return Some((line[..index].trim(), line[index + 1..].trim()));
    }
    None
}

fn parse_keep(value: &str) -> Vec<String> {
    let value = value.trim().trim_start_matches('[').trim_end_matches(']');
    value
        .split(',')
        .map(strip_quotes)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn strip_quotes(value: &str) -> &str {
    value
        .trim()
        .trim_matches(',')
        .trim_matches('"')
        .trim_matches('\'')
}
