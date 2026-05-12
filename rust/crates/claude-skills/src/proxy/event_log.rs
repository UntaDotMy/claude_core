//! Purpose: Append token-saving measurement events for the gain/discover surfaces.
//! Caller: proxy::run after raw output is saved and compact output is rendered.
//! Dependencies: RunMeta, CompactResult, Claude home resolution, and JSONL file storage.
//! Main Functions: record_compaction_event.
//! Side Effects: Appends one JSON object per proxied command to the command compaction event log.

use crate::proxy::adapter::CompactResult;
use crate::proxy::raw_store::RunMeta;
use crate::runtime::{display_path, resolve_claude_home, COMMAND_COMPACTION_EVENTS_FILE_NAME};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionEvent {
    pub timestamp: String,
    #[serde(flatten)]
    pub meta: RunMeta,
}

pub fn record_compaction_event(meta: &RunMeta, compact: &CompactResult) {
    let Ok(claude_home) = resolve_claude_home("") else {
        return;
    };
    if fs::create_dir_all(&claude_home).is_err() {
        return;
    }
    let event_path = claude_home.join(COMMAND_COMPACTION_EVENTS_FILE_NAME);
    let payload = serde_json::json!({
        "timestamp": meta.started_at.to_string(),
        "command": &meta.command,
        "exit_code": meta.exit_code,
        "exitCode": meta.exit_code,
        "compacted": meta.compacted,
        "adapter_name": &meta.adapter_name,
        "adapterName": &meta.adapter_name,
        "tokens_before": meta.estimated_tokens_before,
        "reducer": &compact.adapter_name,
        "tokens_after": meta.estimated_tokens_after,
        "commandFamily": &compact.adapter_name,
        "tokens_saved": meta.estimated_tokens_saved.max(0) as usize,
        "exact_tokens_before": meta.estimated_tokens_before,
        "exact_tokens_after": meta.estimated_tokens_after,
        "exact_tokens_saved": meta.estimated_tokens_saved.max(0) as usize,
        "tokenizer": "o200k_base",
        "token_counting": "exact",
        "summary": &compact.summary,
        "savings_pct": meta.savings_pct,
        "stdoutBytes": meta.stdout_bytes,
        "raw_path": display_path(&meta.raw_path),
        "stderrBytes": meta.stderr_bytes,
        "rawBytes": meta.stdout_bytes + meta.stderr_bytes,
        "compact_path": display_path(&meta.compact_path),
        "renderedBytes": meta.compact_stdout_bytes + meta.compact_stderr_bytes,
        "savedBytes": (meta.stdout_bytes + meta.stderr_bytes)
            .saturating_sub(meta.compact_stdout_bytes + meta.compact_stderr_bytes),
        "tokensBefore": meta.estimated_tokens_before,
        "tokensAfter": meta.estimated_tokens_after,
        "tokensSaved": meta.estimated_tokens_saved.max(0) as usize,
        "savingsPct": meta.savings_pct,
        "rawPath": display_path(&meta.raw_path),
        "rawOutputPath": display_path(&meta.raw_path),
        "compactPath": display_path(&meta.compact_path),
        "agent": &meta.agent,
        "workspace": display_path(&meta.workspace),
    });
    let Ok(rendered) = serde_json::to_string(&payload) else {
        return;
    };
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(event_path)
    {
        let _ = writeln!(file, "{rendered}");
    }
}
