# Phase 24 Verification

- `CTXPACK_BIN=/tmp/ctxpack-target/debug/ctxpack bash scripts/smoke-precision.sh`
- `bash scripts/check-release-docs.sh`
- `CTXPACK_HOME=$(mktemp -d) /tmp/ctxpack-target/debug/ctxpack symbols --repo ../RefactoringMiner --query UMLClassBaseDiff --limit 5`
- `CTXPACK_HOME=$(mktemp -d) /tmp/ctxpack-target/debug/ctxpack dependencies 'src/main 2/java/gui/RunWithTwoDirectories.java' --repo ../RefactoringMiner --limit 20`
