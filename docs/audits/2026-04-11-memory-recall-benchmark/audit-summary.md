# Memory Recall Benchmark Audit Summary

- Audit date: 2026-04-11
- Audited repo: `UntaDotMy/claude_skills` at `bc6dad0`
- Comparator: the memory-product comparator source
- Evidence sources: `docs/memoriesv2-long-term-architecture.md`, `docs/open-source-memory-patterns.md`, `docs/memory-recall-benchmark-bundle.md`, and anonymized peer-audit snapshots for the memory-product comparator class.

## Current score

- `claude_skills` `7/10`
- `memory-product comparator` `9/10`

| Surface | `claude_skills` | Comparator | Evidence-backed conclusion |
| --- | --- | --- | --- |
| Durable memory recall | `7/10` | `memory-product comparator` `9/10` | `claude_skills` already ships scoped durable artifacts, research cache reuse, a relation graph, consolidation, and incremental memory indexing. `memory-product comparator` still leads because it adds semantic recall, a four-layer runtime, hook-driven save and precompact flows, tool-facing integrations, and a public benchmark posture that is more memory-specific than this repo's current broader audit set. |

## Main findings

- `claude_skills` is no longer missing the storage model. The gap is recall product depth and benchmark specificity.
- `memory-product comparator` still sets the bar on semantic memory recall and benchmark packaging.
- The next repo-local memory wins should be measured against this separated bundle rather than mixed back into the broader workflow scorecard.
