# MemoriesV2 Long-Term Architecture

This document defines the repo-native direction for long-term memory in `claude_skills`.

## Goal

- Make `memoriesv2` the canonical durable second-layer memory store.
- Keep the native `claude-skills memory ...` and `claude-skills orchestration ...` commands as the write path during migration.
- Add repo-native retrieval, consolidation, and indexing so the system behaves like a long-term memory runtime instead of a collection of files.
- Maximize practical prompt-cache reuse with deterministic packet assembly. Do not promise a literal 100 percent cache hit rate.

## Why Change

The repo already has strong storage primitives:

- scoped workspace and workstream memory
- working brief, completion gate, and execution trace artifacts
- a mirrored `memoriesv2` layout
- research cache
- local code search

The current runtime now has the core long-term memory loop in-repo: canonical scoped storage, deterministic retrieval packets, an append-only workstream ledger, typed graph entities, explicit hook-capture events, relation graph records, consolidation summaries, and an incremental local memory index. The remaining work is about widening adoption and hardening more commands against drift, not inventing those primitives from scratch.

## Open-Source Ideas Mapped Into This Repo

- `Project-AI-MemoryCore`: layered session-to-long-term memory and consolidation after active work
- `beads`: explicit relationship tracking across tasks, decisions, and dependencies instead of loose note piles
- `octave-mcp`: canonical packets, deterministic artifacts, and receipt-oriented handoffs without depending on MCP transport
- `memory-product comparator`: typed entity recall and save-loop capture ideas adapted into explicit native commands instead of hidden interception
- `indexing comparator` and `indexing-code comparator`: local incremental indexing, lineage, and code retrieval without a hosted service

## Canonical Storage Model

`memoriesv2` becomes the durable source of truth under `~/.claude/memoriesv2/`:

- `global/`
  - reusable patterns
  - mistake ledger
  - shared research cache
- `workspaces/<workspace-slug>/`
  - workspace profile
  - summary, learnings, mistakes
  - reference material
- `workspaces/<workspace-slug>/workstreams/<workstream-key>/`
  - session state
  - working brief
  - completion gate
  - workstream status
  - execution trace
  - working buffer
  - review notes
  - workstream mistakes
  - reference material
- `workspaces/<workspace-slug>/workstreams/<workstream-key>/lanes/<agent-instance>/`
  - lane memory
  - lane working buffer

Compatibility rule:

- the existing `memories/` tree and native commands remain operational during migration
- writes continue through the native command surfaces
- `memoriesv2` now owns the canonical durable writes for working brief, completion gate, session state, working buffer, and workstream status, while the legacy `memories/` tree is synchronized as a compatibility surface
- scope-aware `memoriesv2`-first loaders now own working brief, completion gate, workstream status, and execution trace reads so legacy files do not silently become the source of truth again
- resume-status and other orchestration inspection paths should prefer canonical `memoriesv2` artifact paths and treat legacy files as compatibility mirrors only

## Current Compatibility Surface

The remaining legacy surface is intentionally narrow and non-authoritative:

- `memory/working-brief*.json`
- `memory/completion-gate.json`
- `memory/workstream-status*.json`
- `memory/execution-trace*.json`
- `memory/SESSION-STATE.md`
- `memory/working-buffer.md`

These files still exist so older commands and human inspection flows keep working, but they are synchronized from canonical `memoriesv2` artifacts instead of serving as the durable source of truth. When a canonical artifact exists and the legacy copy is missing or diverges, the runtime should surface that drift instead of silently trusting the legacy file.

## Runtime Components

### 1. Deterministic Retrieval Packets

The native `memoriesv2 retrieve` command is implemented and now:

- reads scoped artifacts in canonical order
- appends incremental memory-index matches after the stable artifact prefix
- appends first-class entity recall results so reusable people, projects, decisions, tools, and workflows do not stay implicit in generic graph nodes
- supports graph-backed semantic recall modes: `direct`, `bridge`, and `blended`
- supports first-class graph relationship queries through `memoriesv2 graph query` so operators can filter the current workstream graph by node, relation, or supporting text without reading the raw artifact
- appends matching research-cache entries after the memory-search tail
- optionally appends local code-search hits last
- emits receipt-backed JSON or markdown packets designed for better prompt-cache reuse

The semantic recall modes stay honest about scope:

- `direct`: surface only graph edges whose nodes, summaries, or evidence directly match the active query
- `bridge`: surface bridge-only graph edges that do not directly match the query text but do connect to the current memory-index hits
- `blended`: emit the union of direct graph matches and bridge-only graph matches so relationship recall can widen without pretending this is vector retrieval

The graph and entity surfaces now also stay honest about scope:

- `memoriesv2 graph list`: inspect the current scoped graph as stored
- `memoriesv2 graph query --node <text>`: filter edges by matching either endpoint node
- `memoriesv2 graph query --relation <type>`: narrow edges to one normalized relation type
- `memoriesv2 graph query --contains <text>`: search the current graph summaries, evidence, and node labels without pretending edge filters replace typed entity recall
- `memoriesv2 entity upsert|list|query`: manage typed entities on the same canonical graph file and expose them as first-class retrieval material

### 2. Append-Only Memory Ledger

The append-only workstream ledger is implemented and records:

- event type
- timestamp
- scope
- source command
- affected artifact
- normalized summary
- optional relation ids

Purpose:

- preserve durable lineage
- support replay and consolidation
- make it clear which command wrote which durable fact
- keep entity-memory and hook-capture writes on the same durable lineage surface as the rest of the workstream

### 3. Consolidation Rules

Consolidation is implemented through a native `memoriesv2 consolidate` command that promotes active artifacts into durable workstream and workspace summaries instead of copying everything forever.

Promotion targets:

- repeated validated wins -> global or workspace patterns
- repeated mistakes -> mistake ledgers
- reusable external findings -> shared research cache with freshness metadata
- workstream-specific outcomes -> workstream summary and review notes

### 4. Typed Entity Layer

The workstream graph now carries a first-class typed entity layer:

- entities live beside graph edges in the canonical workstream graph artifact
- `memoriesv2 entity upsert` records typed entities such as decisions, tools, workflows, artifacts, and people without creating a separate store
- `memoriesv2 entity query` gives operators a direct filter surface instead of forcing every lookup through edge text alone
- `memoriesv2 retrieve` reuses that typed entity surface through entity recall results

This keeps the entity owner explicit while still reusing the same canonical graph file and ledger lineage.

### 5. Explicit Hook Capture

The save-loop parity surface is implemented through an explicit native `memoriesv2 hook capture` command:

- callers choose when to record checkpoint, stop, precompact, manual, or other hook-style events
- optional artifact receipts make the captured event traceable back to the durable file that was saved
- optional graph linkage keeps the event connected to the existing workstream graph owner
- the surface stays caller-driven and inspectable instead of relying on background interception or prompt-only automation

### 6. Relation Graph

The workstream relation graph is implemented and connects:

- requirement -> plan item -> trace evidence -> completion state
- research finding -> decision -> touched files
- review finding -> fix -> validation evidence
- lane packet -> task handoff -> terminal outcome

This borrows the useful part of `beads` without introducing a separate service.

### 7. Incremental Local Indexing

The workspace memory index is implemented with native `memoriesv2 index build|status|reset` commands and memory-aware retrieval:

- index durable memory artifacts by scope and kind
- preserve stable file lineage across refreshes
- refresh incrementally where possible
- keep retrieval local and repo-owned
- feed indexed memory matches back into `memoriesv2 retrieve` without rescanning every artifact on every turn

## Prompt-Cache Strategy

To improve cache reuse honestly:

- keep the packet schema stable
- keep canonical artifact order fixed
- emit stable scoped memory first
- append query-specific memory-index matches later
- append query-specific research matches after the memory-index tail
- append code-search evidence last
- avoid rewriting the front of the packet when only volatile evidence changed

This is how to aim for very high cache reuse in practice. It is not a guarantee of literal 100 percent cache hits.

## Migration Order

1. Keep `memoriesv2` as the canonical durable write authority and remove any drift back to legacy-first reads.
2. Expand scope-aware `memoriesv2`-first loaders across the remaining command surfaces.
3. Tighten retrieval ranking and packet assembly as real usage data exposes better heuristics.
4. Harden more orchestration and maintenance paths against authority drift with targeted tests.
5. Keep the index and consolidation layers incremental, local, and deterministic as the artifact set grows.

## Current Production Slice

The current production-oriented slice stays repo-native and intentionally avoids external dependencies:

- no MCP
- no hosted service
- canonical `memoriesv2` durable writes with legacy compatibility sync
- deterministic retrieval packets with receipts
- append-only ledger plus typed entities plus relation graph plus consolidation summaries
- explicit hook capture for caller-driven save-loop parity
- incremental local memory indexing
- optional local code-search augmentation
- targeted regression coverage across memory, memoriesv2, orchestration, app, and help surfaces
