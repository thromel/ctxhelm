# Phase 279 - Task-Mentioned Commit Path Anchors

## Goal

Improve public commit reproduction tasks where the user names an upstream
commit SHA but does not name the changed files. The planner should use the
commit metadata as a local, source-free anchor signal instead of relying only on
task terms.

## Code Kept

Prepare-task planning now extracts task-mentioned commit revisions only when the
prompt explicitly contains `commit` and a 7-40 character hex token. For each
revision, ctxhelm runs local `git diff-tree --name-status -z`, labels changed
paths through the safe inventory, and promotes safe changed files into the
normal anchor set.

The promoted paths are added as:

- explicit path query facets with origin `mentioned_commit_changed_path`
- lexical terms
- graph seeds
- related-test terms
- prepare-task target anchors

The path signal is bounded by the normal prepare-task target limit and emits
diagnostics:

- `mentioned_commit_changed_paths`
- `mentioned_commit_changed_paths_excluded`
- `mentioned_commit_changed_paths_unavailable`
- `mentioned_commit_partial`

## Privacy Boundary

The planner reads commit changed-path metadata only from the local Git checkout.
It does not return commit subjects, source text, patches, or file contents, and
it filters all changed paths through the safe inventory before adding them to
the context plan.

## Validation

Focused validation:

```bash
cargo test -p ctxhelm-compiler prepare_plan_promotes_task_mentioned_commit_changed_paths --locked
cargo run -q -p ctxhelm -- prepare-task "Recreate the behavior from public commit a62c08c: Promote task-mentioned commit changed paths." --repo /Users/romel/Documents/GitHub/ctxhelm --mode bug-fix --no-trace
```

The unit proof builds a temporary Git repository, creates a commit that changes
two safe Java files, asks prepare-task to recreate the commit behavior from the
short SHA, and asserts both changed files are selected while serialized plan
JSON does not contain source content from the changed files.

## Decision

Promote this planner behavior. It is source-free, local-only, inventory-filtered,
and addresses a real class of issue-reproduction tasks where a commit SHA is
more precise than the surrounding task prose.
