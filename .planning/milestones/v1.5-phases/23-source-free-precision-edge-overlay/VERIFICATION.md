# Phase 23 Verification

- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm-index precision_edge_import_is_source_free_and_additive -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/ctxhelm-target cargo test -p ctxhelm --test cli_compat search_related_tests_dependencies_and_eval_history_emit_json_shapes -- --nocapture`

