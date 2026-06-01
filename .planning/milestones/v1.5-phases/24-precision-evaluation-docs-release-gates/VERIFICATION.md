# Phase 24 Verification

- `CTXHELM_BIN=/tmp/ctxhelm-target/debug/ctxhelm bash scripts/smoke-precision.sh`
- `bash scripts/check-release-docs.sh`
- `CTXHELM_HOME=$(mktemp -d) /tmp/ctxhelm-target/debug/ctxhelm symbols --repo ../RefactoringMiner --query UMLClassBaseDiff --limit 5`
- `CTXHELM_HOME=$(mktemp -d) /tmp/ctxhelm-target/debug/ctxhelm dependencies 'src/main 2/java/gui/RunWithTwoDirectories.java' --repo ../RefactoringMiner --limit 20`
