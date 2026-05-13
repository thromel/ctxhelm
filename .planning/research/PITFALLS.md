# Pitfalls Research

**Domain:** Local-first repository context compiler for coding agents
**Researched:** 2026-05-13
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Treating Cached Inventory as Ground Truth

**What goes wrong:**
ctxpack returns files that were deleted, renamed, newly ignored, moved into generated output, or changed from safe source into sensitive config. Retrieval appears to work, but the agent is reasoning over an old repository snapshot.

**Why it happens:**
Local-first tools often optimize for fast repeat reads and forget that active coding sessions mutate files continuously. In ctxpack, the current concern is concrete: `load_or_build_inventory` reuses `inventory.json` whenever it exists instead of checking filesystem state, ignore-file hashes, tool version, or inventory options.

**How to avoid:**
Make inventory freshness a first-class contract before graph or scoring work. Store repo root, ctxpack version, inventory options, scan timestamp, ignore-file hashes, file count, file mtimes/sizes, and a cheap dirty summary. Every read path that powers `search`, `symbols`, `prepare_task`, `get_pack`, `related_tests`, dependency graph, and MCP resources should either prove freshness, rebuild, or return a stale-cache diagnostic. Add mutation fixtures for create, delete, rename, ignore, generated-path move, sensitive rename, and option changes.

**Warning signs:**
- Historical eval improves only when `ctxpack index` was run manually first.
- `search` returns paths that no longer exist.
- `current_diff` and inventory disagree about changed safe paths.
- Privacy exclusions pass in fresh tests but fail after cache reuse.
- Users learn to run `ctxpack index` as a superstition before every query.

**Phase to address:**
Phase 1 - Cache Freshness and Diagnostics. This must come before graph lift because stale inputs make every scoring experiment untrustworthy.

---

### Pitfall 2: Privacy Policy as a Filename Denylist

**What goes wrong:**
Secrets enter the safe inventory and later leak through MCP safe file slices, context packs, generated cards, or traces. Local-first does not help if the local MCP client or cloud-backed agent receives snippets from `.npmrc`, `.netrc`, SSH keys, package auth files, service account JSON, Docker compose secrets, or repo-specific credential paths.

**Why it happens:**
Developers start with obvious names like `.env` and `private.key`, then assume privacy is solved. Repository secrets are diverse and often live in ordinary config formats. ctxpack already depends on safe inventory for every downstream exposure, so a denylist miss has broad blast radius.

**How to avoid:**
Move classification into a dedicated privacy policy module with table-driven corpus tests. Default to conservative handling for auth-bearing dotfiles and credential-shaped JSON names. Revalidate every snippet path against the current safe inventory immediately before reading, not just when the plan was built. Add repo-local allow/deny policy configuration with source-free reporting of why files were excluded. Keep traces and cards source-free by design.

**Warning signs:**
- New source surfaces reuse inventory paths without rechecking safety.
- Tests only assert `.env` and `*.key`.
- Generated cards contain config filenames that look like auth material.
- `--include-sensitive` is the only escape hatch for legitimate false positives.
- Privacy status says `local_only` but does not list excluded counts or policy version.

**Phase to address:**
Phase 2 - Privacy Policy Hardening. This should land before any broader pack/card expansion or richer MCP file resources.

---

### Pitfall 3: Context Bloat Masquerading as Better Recall

**What goes wrong:**
Recall@10 rises by stuffing more files into the context plan, but agents get slower and less accurate because packs contain noisy tests, docs, generated-like files, duplicate snippets, or weakly related graph neighbors. The product loses its core value: the smallest safe evidence set.

**Why it happens:**
RAG systems often optimize retrieval recall in isolation. MCP client guidance explicitly warns that loading too many tools or intermediate results wastes tokens, increases latency, and degrades model performance. The same applies to repo context packs: a larger pack can be a worse agent input even if a hidden label appears somewhere inside it.

**How to avoid:**
Treat token budget, precision, and context usefulness as first-class metrics. Keep brief, standard, and deep budgets, but require each budget to report selected file count, snippet bytes/tokens, evidence type mix, duplicate paths removed, and risk flags for low-confidence expansion. Optimize for lift at fixed budgets. Add eval columns for Recall@5/10, precision@k, source/test role recall, pack size, and "unhelpful retrieved file" samples.

**Warning signs:**
- Deep packs become the default recommendation for ordinary bug fixes.
- Most selected files have generic reasons like "lexical match" or "dependency edge".
- Recall improves only when file limits increase.
- Agents ignore ctxpack suggestions and use their own search repeatedly.
- RefactoringMiner eval ties lexical baseline while returning more mixed evidence.

**Phase to address:**
Phase 3 - Measured Retrieval Lift. Do not accept graph expansion unless it improves fixed-budget retrieval quality, not just total context volume.

---

### Pitfall 4: Misleading Dependency and Symbol Edges

**What goes wrong:**
ctxpack sends the agent from a real target file to an irrelevant or wrong neighbor because import resolution, symbol extraction, or dependency direction is approximate. False graph edges are more harmful than absent edges because agents tend to trust structural evidence.

**Why it happens:**
Line-based parsers catch common demos but miss aliases, workspaces, package exports, namespace packages, generated modules, Rust module edge cases, Java classpaths, and dynamic imports. Current research on repository-level code retrieval increasingly points toward graph-aware retrieval, but graph quality matters: deterministic AST-derived graphs are reported as more reliable than LLM-generated or incomplete graphs for codebase reasoning.

**How to avoid:**
Keep graph expansion deterministic and provenance-rich. Add language-specific parser traits behind current contracts; start with JS/TS alias/workspace resolution and Rust modules because those are likely to affect ctxpack itself. Store edge type, direction, confidence, resolver, and reason. Penalize low-confidence edges in planning and surface them as risk flags rather than primary targets. Add fixtures for false positive edges and missing alias edges before using graph score as a ranking boost.

**Warning signs:**
- Dependency graph looks plausible in cards but fails targeted fixture assertions.
- `related` returns many one-hop neighbors with identical confidence.
- Graph expansion hurts lexical baseline on real repos.
- Co-change and dependency evidence conflict without explanation.
- Users see "dependency" edges to barrels, generated types, or config files more often than implementation files.

**Phase to address:**
Phase 3 - Measured Retrieval Lift, with parser/resolver subphase before graph-based ranking changes.

---

### Pitfall 5: Weak Historical Eval Methodology

**What goes wrong:**
The roadmap optimizes to a metric that does not represent real agent help. Commit subjects are noisy prompts, changed files are incomplete labels, deletes and renames disappear, tests may be labels or context depending on task type, and a tied lexical baseline hides whether graph/history signals add value.

**Why it happens:**
Historical eval is attractive because it is source-free and easy to automate. But retrieval evaluation guidance separates context precision, context recall, entity/claim coverage, and generation usefulness. Code retrieval benchmarks also emphasize task diversity and realistic technical domains, not one recall number on one slice.

**How to avoid:**
Keep historical eval, but make it diagnostic rather than decorative. Freeze `--base`/`--head` ranges for repeatable tuning. Report source-file recall separately from test/config/docs recall. Track precision@k, MRR, role-aware gaps, rename/delete handling, low-information commits, graph-only wins, graph-caused false positives, history-only wins, and budget size. Add at least two external smoke repositories after RefactoringMiner before declaring lift. Use real agent dogfood traces as a second signal, but keep them source-free.

**Warning signs:**
- A roadmap item claims success because Recall@10 improved while Recall@5 or precision fell.
- The same commits are not used across tuning runs.
- Deleted, renamed, generated, or sensitive paths are silently excluded from denominators.
- Eval output cannot explain why ctxpack tied lexical baseline.
- No distinction between "source target found" and "test recommendation found".

**Phase to address:**
Phase 4 - Evaluation Rigor. It should run in parallel with retrieval lift but gate claims that ctxpack beats lexical baseline.

---

### Pitfall 6: Building More Retrieval Modalities Without Fusion Discipline

**What goes wrong:**
Lexical, symbol, dependency, co-change, current-diff, and active-path signals all compete in a pile of heuristics. Adding one more signal changes ranking in surprising ways, and no one can explain why a file was selected.

**Why it happens:**
Hybrid retrieval works when each signal has a calibrated job. It fails when all signals become undifferentiated boosts. Research on repository-level code retrieval shows value in multi-path retrieval and reranking, but the benefit comes from matching query type and task type to the right retrieval path.

**How to avoid:**
Define a fusion contract: each candidate carries source signal, score, confidence, reason, role, freshness status, privacy status, and budget cost. Add task-type-specific weights for bug fix, feature, refactor, review, test writing, and explanation. Require ablation reporting in eval: lexical-only, lexical+symbols, lexical+graph, lexical+history, current-diff anchored, and full fusion. Keep score math in one module with golden ranking tests.

**Warning signs:**
- A file's reason lists many signals but no dominant evidence.
- Tuning one weight fixes RefactoringMiner and breaks small fixtures.
- Current-diff anchors always dominate even when the task is unrelated.
- Co-change hints are treated as primary evidence instead of soft hints.
- The same query returns unstable rankings after unrelated module changes.

**Phase to address:**
Phase 3 - Measured Retrieval Lift. Fusion refactor should happen before broad graph expansion.

---

### Pitfall 7: Read-Only Product With Surprising Write Side Effects

**What goes wrong:**
Users trust ctxpack as read-only, but normal read flows write inventory and traces under home. This can fail in read-only home directories, leak source-adjacent metadata longer than expected, fill disk, or violate expectations in CI and client smoke tests.

**Why it happens:**
Local cache writes feel harmless compared with source edits. But "read-only" is a product contract about user trust, not only repository mutation. ctxpack currently writes local inventory and source-free traces during common read-oriented operations.

**How to avoid:**
Document the exact local writes in README, generated AGENTS.md, and CLI help. Add `--no-trace`, config-level trace controls, trace retention by count/age/bytes, and non-fatal trace write failures for MCP. Separate "repo-local writes" from "home cache writes" in init and diagnostics. Add a `ctxpack doctor` or diagnostics output that reports cache paths and sizes.

**Warning signs:**
- MCP `prepare_task` fails because trace append fails.
- Users are surprised by `~/.ctxpack` growth.
- Tests need environment locks because global `CTXPACK_HOME` writes are implicit.
- "Read-only" appears in docs without a precise cache/traces caveat.
- Generated adapters imply no mutation anywhere.

**Phase to address:**
Phase 1 - Cache Freshness and Diagnostics, with trace retention in Phase 5 - Operational Hardening.

---

### Pitfall 8: MCP Session Semantics That Work Only in Unit Tests

**What goes wrong:**
`prepare_task` returns pack resource URIs that fail when the client reconnects, starts another server process, or resolves resources from a different cwd. The demo works in one stdio session but real Codex, Claude Code, Cursor, or OpenCode usage becomes brittle.

**Why it happens:**
MCP resources and local server lifecycle are client-dependent. ctxpack currently has session-scoped pack resources backed by process-local memory. The README already tells clients to pass explicit `repo` because server cwd can differ from the active project.

**How to avoid:**
Prefer `get_pack` as the durable path for clients. For pack resources, either persist enough source-free plan metadata to regenerate within freshness/privacy constraints or mark resource URIs as session-scoped in structured output and prompts. Add client-path tests for reconnect, explicit repo, omitted repo, different cwd, concurrent requests, and resource reads from a new process.

**Warning signs:**
- Resource-read tests only call the same process that created the pack.
- Users report `ctxpack://pack/...` not found after client restart.
- Generated prompts omit explicit `repo`.
- Multiple MCP clients attached to the same repo return different packs.
- Session cache grows without eviction.

**Phase to address:**
Phase 5 - MCP and Client Operational Hardening. This follows core retrieval work but should gate any claim of broad client support.

---

### Pitfall 9: Opaque Partial Failures

**What goes wrong:**
Weak retrieval looks like "no relevant files" when the real problem is stale cache, unreadable files, non-UTF-8 content, skipped large files, git timeout, missing `tar`, parser failure, or dependency graph truncation. Users cannot tell whether to improve the task prompt, reindex, fix permissions, or ignore ctxpack.

**Why it happens:**
Search APIs often collapse operational failures into empty results for convenience. ctxpack currently has paths that read files with `unwrap_or_default`, so unreadable content can become an empty match surface.

**How to avoid:**
Return structured diagnostics on every user-facing plan/search/related response: stale/fresh inventory status, unreadable file count, skipped large/binary count, git status, dependency resolver coverage, parser warnings, trace write status, and external-tool timeout counts. Keep errors non-fatal where possible, but never invisible. Add CLI and MCP JSON tests for diagnostic fields.

**Warning signs:**
- Risk flags only describe task ambiguity, not subsystem health.
- `co_change_unavailable` appears but other failures do not.
- Large repos produce weak plans with no skipped-file counts.
- Historical eval gaps cannot be traced to retrieval, label, freshness, or parser causes.
- Users need `RUST_LOG` or manual inspection to understand routine failures.

**Phase to address:**
Phase 1 - Cache Freshness and Diagnostics. Diagnostics should ship before optimizing ranking.

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Reusing inventory without freshness metadata | Fast repeated reads | Unsafe stale retrieval and invalid evals | Never for user-facing retrieval after Phase 1 |
| Extending `is_sensitive_path` with one-off strings | Quick privacy fixes | Permanent whack-a-mole policy with snippet leak risk | Only as emergency patch before policy module |
| Adding graph score boosts inline | Faster graph lift experiment | Unexplainable ranking and hard-to-reproduce regressions | Only behind eval flag and ablation report |
| Keeping all index/compiler/MCP code in giant files | Avoids refactor overhead | Privacy, scoring, and protocol changes become coupled | Acceptable until Phase 3 starts changing fusion |
| Treating `unwrap_or_default` reads as harmless | Fewer errors to plumb | Silent false negatives and hidden non-UTF-8 issues | Never for retrieval inputs after diagnostics |
| Using only unit-level MCP protocol tests | Fast validation | Real clients fail on cwd, reconnect, process, and config differences | Acceptable for protocol shape, not client support claims |

## Integration Gotchas

Common mistakes when connecting to external services or host agents.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Codex CLI | Claiming support after server starts but before the client invokes `prepare_task` | Run real client smoke with explicit `repo`, isolated config, and asserted target file output |
| Claude Code | Depending on server cwd or hidden global config | Generate repo-local setup guidance and require explicit `repo` in prompts/tools |
| Cursor/OpenCode adapters | Treating thin rules as product-complete integration | Keep rules small, source-free, and tied to MCP/CLI commands that are actually tested |
| MCP resources | Returning process-local URIs as if durable | Label session scope or regenerate from persisted source-free plan metadata |
| Local git/tar | Assuming tools exist and are fast | Centralize process execution, timeouts, version/error diagnostics, and partial-result warnings |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Re-reading full files per operation | Slow search/symbol/pack runs, high memory, repeated disk churn | Store size metadata, cap reads, cache decoded text per operation, stream hashes | Large repos, huge source files, binary files with source-like extensions |
| Sequential git history scans | Eval and co-change hints feel hung or time out | Batch with `git log --name-only`, apply limits at git query layer, cache commit file sets | Long histories like RefactoringMiner |
| Unbounded traces and pack cache | Growing `~/.ctxpack`, long-lived MCP process memory | Retention limits, LRU pack cache, cache-size diagnostics | Frequent MCP use and repeated deep packs |
| Expanding all graph neighbors | Better-looking graph cards but noisy plans | Edge confidence, role filters, budget-aware traversal, ablation tests | Monorepos and dependency-heavy JS/Rust projects |
| Large MCP/tool payloads | More tokens, higher latency, lower model attention | Progressive discovery style, concise structured results, budgeted resources | Many tools/resources or deep packs |

## Security Mistakes

Domain-specific security issues beyond general application security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Assuming local-first means private | Safe snippets can still reach a cloud-backed agent through MCP | Enforce safe inventory, source-free traces/cards, explicit pack snippet policy |
| Stale privacy state | Files renamed into sensitive paths remain retrievable | Freshness checks must include path classification and ignore policy hashes |
| Broad MCP surface | Larger attack and prompt-injection surface | Keep small read-only tool set, no autonomous editing, no shell execution tools |
| Insecure local MCP transport choices | Other local processes or browser attacks reach server | Keep stdio default; if HTTP is ever added, require auth and restricted IPC |
| Hidden trace retention | Source-free metadata may still be sensitive over time | Retention controls and visible cache/traces diagnostics |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Weak plans without explanation | User distrusts ctxpack or retries blindly | Return diagnostics and missing-info questions with concrete causes |
| Too many setup surfaces | Users cannot tell whether AGENTS.md, adapter files, MCP, or CLI is authoritative | Keep AGENTS.md/MCP primary, adapters thin, CLI for debugging |
| Deep packs as default | Agents receive too much noise | Recommend smallest budget that satisfies confidence and role coverage |
| Vague privacy labels | Users do not know what is safe to share | Report local-only status plus excluded counts and snippet policy |
| Manual reindex ritual | Users blame themselves for stale results | Automatic freshness checks with clear rebuild diagnostics |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Cache invalidation:** Search works on fresh inventory - verify create/delete/rename/ignore/sensitive mutations after cache exists.
- [ ] **Privacy policy:** `.env` is excluded - verify package auth files, SSH key names, service account JSON, `.netrc`, `.pypirc`, `.npmrc`, and nested secret directories.
- [ ] **Graph retrieval:** Dependency graph renders - verify graph expansion beats lexical baseline at fixed budget and does not increase false positives.
- [ ] **Historical eval:** Recall@10 reports - verify frozen commit range, role-aware recall, precision@k, rename/delete cases, and graph/history ablations.
- [ ] **MCP resources:** Pack URI reads in unit test - verify reconnect, different process, explicit repo, omitted repo, and client cwd mismatch.
- [ ] **Read-only claim:** No source edits happen - verify and document home cache/traces writes, retention, and `--no-trace`.
- [ ] **Diagnostics:** Risk flags exist - verify stale cache, unreadable files, git timeout, parser failure, skipped large files, and trace write failures appear in structured output.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Stale inventory shipped | MEDIUM | Add freshness metadata, invalidate all old caches by schema version, add mutation regression tests, document reindex workaround |
| Secret-like file exposed in pack | HIGH | Patch privacy policy, invalidate affected caches/cards, add corpus test, audit pack/file-resource revalidation, document affected versions |
| Graph expansion hurts retrieval | MEDIUM | Disable graph boost behind config, keep graph as risk evidence, run ablation, add false-positive fixtures before re-enabling |
| Eval metric misled roadmap | MEDIUM | Freeze benchmark ranges, add role-aware and precision metrics, rerun old and new retrieval stacks, update roadmap gates |
| MCP resource lifecycle fails | LOW | Prefer `get_pack`, label session-scoped URIs, add reconnect tests, optionally persist source-free plan metadata |
| Context bloat becomes default | MEDIUM | Lower budget defaults, add pack-size gates, require fixed-budget lift, prune duplicate/low-confidence evidence |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Treating cached inventory as ground truth | Phase 1 - Cache Freshness and Diagnostics | Mutation tests plus CLI/MCP stale-cache diagnostics |
| Opaque partial failures | Phase 1 - Cache Freshness and Diagnostics | Structured diagnostics appear in search, prepare_task, related, get_pack, and eval |
| Read-only product with surprising writes | Phase 1 and Phase 5 - Operational Hardening | README/help disclose writes; `--no-trace`; retention tests; non-fatal MCP trace errors |
| Privacy policy as filename denylist | Phase 2 - Privacy Policy Hardening | Privacy corpus tests and pack-level revalidation before snippet reads |
| Context bloat masquerading as recall | Phase 3 - Measured Retrieval Lift | Fixed-budget Recall@5/10, precision@k, pack-size, and role-recall gates |
| Misleading dependency and symbol edges | Phase 3 - Measured Retrieval Lift | Parser/resolver fixtures, edge confidence, graph ablation report |
| Retrieval modalities without fusion discipline | Phase 3 - Measured Retrieval Lift | Central scorer, candidate provenance, task-type weights, ablation tests |
| Weak historical eval methodology | Phase 4 - Evaluation Rigor | Frozen ranges, rename/delete cases, role-aware metrics, multiple repo smokes |
| MCP session semantics only in unit tests | Phase 5 - MCP and Client Operational Hardening | Real client smoke plus reconnect/different-process resource tests |

## Sources

- Local project context: `.planning/PROJECT.md`, `.planning/codebase/CONCERNS.md`, `.planning/codebase/TESTING.md`, `README.md` - HIGH confidence for current ctxpack state and known risks.
- Model Context Protocol Security Best Practices, current MCP docs - HIGH confidence for local MCP server compromise, scope minimization, and session risks: https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices
- Model Context Protocol Client Best Practices, current MCP docs - HIGH confidence for context/tool bloat and progressive discovery guidance: https://modelcontextprotocol.io/docs/develop/clients/client-best-practices
- Ragas Context Precision docs - HIGH confidence for retrieval precision framing: https://docs.ragas.io/en/v0.2.0/concepts/metrics/available_metrics/context_precision/
- LlamaIndex RAGChecker example - MEDIUM confidence for fine-grained RAG diagnostic metric categories: https://docs.llamaindex.ai/en/stable/examples/evaluation/RAGChecker/
- Retrieval-Augmented Code Generation survey, arXiv v2 Jan 25 2026 - MEDIUM confidence for repository-level code generation requiring long-range dependencies and global consistency: https://arxiv.org/abs/2510.04905
- RANGER repository-level graph-enhanced retrieval, arXiv Sep 27 2025 - MEDIUM confidence for graph-enhanced repository retrieval and hybrid BM25 pairing: https://arxiv.org/abs/2509.25257
- CodexGraph, NAACL 2025 - MEDIUM confidence for code graph databases addressing low recall in similarity-only retrieval: https://aclanthology.org/2025.naacl-long.7/
- CodeRAG, EMNLP 2025 - MEDIUM confidence for multi-path retrieval and reranking issues in repository-level code completion: https://aclanthology.org/2025.emnlp-main.1187/
- CoIR benchmark repository, ACL 2025 - MEDIUM confidence for broad code retrieval benchmark framing: https://github.com/coir-team/coir
- FreshStack, NeurIPS 2025 Datasets and Benchmarks - MEDIUM confidence for realistic technical-domain retrieval evaluation and temporal drift concerns: https://fresh-stack.github.io/

---
*Pitfalls research for: Repo Context Packer*
*Researched: 2026-05-13*
