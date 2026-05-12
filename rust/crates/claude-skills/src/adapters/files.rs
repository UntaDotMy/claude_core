//! Purpose: Compact file read and list commands into maps, counts, and bounded samples.
//! Caller: AdapterRegistry for cat, sed, head, tail, jq, find, tree, and recursive ls.
//! Dependencies: CommandAst classification, RunMeta, and shared adapter helpers.
//! Main Functions: FilesAdapter::compact.
//! Side Effects: None; raw recovery is persisted by proxy::run.

use crate::adapters::common::{compact_edges, make_result};
use crate::proxy::adapter::{CommandAdapter, CompactResult};
use crate::proxy::command_ast::{CommandAst, CommandKind};
use crate::proxy::raw_store::RunMeta;

pub struct FilesAdapter;

impl CommandAdapter for FilesAdapter {
    fn name(&self) -> &'static str {
        "files"
    }

    fn matches(&self, ast: &CommandAst) -> bool {
        matches!(
            ast.detected_kind,
            CommandKind::FileRead | CommandKind::FileList
        )
    }

    fn compact(
        &self,
        stdout: &[u8],
        stderr: &[u8],
        exit_code: i32,
        meta: &RunMeta,
    ) -> CompactResult {
        let stdout_text = String::from_utf8_lossy(stdout);
        let stderr_text = String::from_utf8_lossy(stderr);
        let program = meta.command.split_whitespace().next().unwrap_or("file");
        let compact_stdout = if ["cat", "sed", "head", "tail", "jq"]
            .iter()
            .any(|name| program.ends_with(name))
            && stdout_text.lines().count() > 120
        {
            compact_file_read(&meta.command, &stdout_text)
        } else {
            compact_edges(&stdout_text, "file output", 80)
        };
        let compacted = compact_stdout != stdout_text || stdout_text.lines().count() > 80;
        make_result(
            self.name(),
            meta.command.clone(),
            compact_stdout,
            compact_edges(&stderr_text, "file diagnostics", 40),
            exit_code,
            meta,
            compacted,
        )
    }
}

fn compact_file_read(command: &str, stdout: &str) -> String {
    let line_count = stdout.lines().count();
    let target = command
        .split_whitespace()
        .last()
        .unwrap_or("file")
        .trim_matches('"')
        .trim_matches('\'');
    let symbols = collect_symbols(stdout);
    let mut rendered = format!("file: {target}, {line_count} lines");
    if !symbols.is_empty() {
        rendered.push_str("\n\nsymbols:");
        for symbol in symbols.iter().take(24) {
            rendered.push_str(&format!("\n- {symbol}"));
        }
    }
    rendered.push_str(&format!(
        "\n\nUse:\nclaude-skills read {target} --range 1:120"
    ));
    rendered
}

fn collect_symbols(stdout: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        let candidate = if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("function ")
        {
            Some(trimmed)
        } else {
            None
        };
        if let Some(symbol) = candidate {
            symbols.push(symbol.to_string());
        }
    }
    symbols
}

#[cfg(test)]
mod tests {
    use super::FilesAdapter;
    use crate::proxy::adapter::CommandAdapter;
    use crate::proxy::raw_store::RunMeta;
    use std::path::PathBuf;

    #[test]
    fn large_cat_is_summarized_with_symbols() {
        let mut stdout = String::new();
        stdout.push_str("pub struct CompactResult {}\n");
        for index in 0..160 {
            stdout.push_str(&format!("fn helper_{index}() {{}}\n"));
        }
        let result = FilesAdapter.compact(
            stdout.as_bytes(),
            b"",
            0,
            &meta("cat src/lib.rs", stdout.len()),
        );
        assert!(result.compacted);
        assert!(result.stdout.contains("file: src/lib.rs"));
        assert!(result.stdout.contains("pub struct CompactResult"));
        assert!(result.stdout.contains("Use:"));
    }

    fn meta(command: &str, stdout_bytes: usize) -> RunMeta {
        RunMeta {
            raw_id: "raw".to_string(),
            command: command.to_string(),
            cwd: PathBuf::from("."),
            started_at: 1,
            duration_ms: 1,
            exit_code: 0,
            adapter_name: "files".to_string(),
            raw_path: PathBuf::from("/tmp/raw"),
            compact_path: PathBuf::new(),
            agent: "test".to_string(),
            workspace: PathBuf::from("."),
            stdout_bytes,
            stderr_bytes: 0,
            compact_stdout_bytes: 0,
            compact_stderr_bytes: 0,
            estimated_tokens_before: stdout_bytes / 4,
            estimated_tokens_after: 0,
            estimated_tokens_saved: 0,
            savings_pct: 0.0,
            compacted: false,
        }
    }
}
