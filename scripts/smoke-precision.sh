#!/usr/bin/env bash
set -euo pipefail

ctxpack_bin="${CTXPACK_BIN:-ctxpack}"
work_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$work_dir"
}
trap cleanup EXIT

repo="$work_dir/repo"
home="$work_dir/home"
mkdir -p "$repo/src/main/java/org/example/auth" \
  "$repo/src/main/java/org/example/user" \
  "$repo/src/main/kotlin/org/example/web" \
  "$home"
git -C "$repo" init >/dev/null
git -C "$repo" config user.email "ctxpack@example.com"
git -C "$repo" config user.name "ctxpack"

cat >"$repo/src/main/java/org/example/auth/AuthService.java" <<'JAVA'
package org.example.auth;

import org.example.user.UserRepository;

public class AuthService {
  public static final String COOKIE = "sid";

  public String requireSession() {
    return UserRepository.class.getName();
  }
}
JAVA

cat >"$repo/src/main/java/org/example/user/UserRepository.java" <<'JAVA'
package org.example.user;

public class UserRepository {}
JAVA

cat >"$repo/src/main/kotlin/org/example/web/AuthController.kt" <<'KOTLIN'
package org.example.web

import org.example.auth.AuthService

class AuthController(private val auth: AuthService) {
  fun redirectToLogin(): String = "/login"
}
KOTLIN

cat >"$repo/.env" <<'ENV'
SECRET_SHOULD_NOT_PERSIST=secret
ENV

git -C "$repo" add .
git -C "$repo" commit -m fixture >/dev/null

symbols_json="$work_dir/symbols.json"
CTXPACK_HOME="$home" "$ctxpack_bin" symbols --repo "$repo" --query requireSession --limit 5 >"$symbols_json"
python3 - "$symbols_json" <<'PY'
import json
import pathlib
import sys

symbols = json.loads(pathlib.Path(sys.argv[1]).read_text())
if not any(item["symbol"]["language"] == "java" and item["symbol"]["name"] == "requireSession" for item in symbols):
    raise SystemExit("java requireSession symbol was not returned")
PY

deps_json="$work_dir/dependencies.json"
CTXPACK_HOME="$home" "$ctxpack_bin" dependencies src/main/java/org/example/auth/AuthService.java --repo "$repo" --limit 10 >"$deps_json"
python3 - "$deps_json" <<'PY'
import json
import pathlib
import sys

edges = json.loads(pathlib.Path(sys.argv[1]).read_text())
expected = (
    "src/main/java/org/example/auth/AuthService.java",
    "src/main/java/org/example/user/UserRepository.java",
)
if not any((edge["sourcePath"], edge["targetPath"]) == expected for edge in edges):
    raise SystemExit("java package import edge was not returned")
PY

discover_json="$work_dir/precision-discover.json"
CTXPACK_HOME="$home" "$ctxpack_bin" precision discover --repo "$repo" --limit 20 --format json >"$discover_json"
python3 - "$discover_json" "$repo/.ctxpack/precision-edges.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
overlay = pathlib.Path(sys.argv[2]).read_text()
if report.get("provider") != "local_tree_sitter_reference_scan":
    raise SystemExit("tree-sitter precision discovery provider was not reported")
if report.get("discoveredEdges", 0) < 1:
    raise SystemExit("tree-sitter precision discovery found no edges")
if "SECRET_SHOULD_NOT_PERSIST" in overlay:
    raise SystemExit("precision discovery overlay leaked secret marker")
PY

precision_input="$work_dir/precision.json"
cat >"$precision_input" <<'JSON'
{
  "schemaVersion": 1,
  "provider": "scip-json-fixture",
  "edges": [
    {
      "sourcePath": "src/main/kotlin/org/example/web/AuthController.kt",
      "targetPath": "src/main/java/org/example/auth/AuthService.java",
      "edgeType": "calls",
      "symbol": "AuthService",
      "confidence": 0.99,
      "reason": "local precision smoke"
    },
    {
      "sourcePath": ".env",
      "targetPath": "src/main/java/org/example/auth/AuthService.java",
      "edgeType": "calls",
      "reason": "SECRET_SHOULD_NOT_PERSIST"
    }
  ]
}
JSON

import_json="$work_dir/precision-import.json"
CTXPACK_HOME="$home" "$ctxpack_bin" precision import --repo "$repo" --input "$precision_input" --format json >"$import_json"
python3 - "$import_json" "$repo/.ctxpack/precision-edges.json" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
overlay = pathlib.Path(sys.argv[2]).read_text()
if report.get("acceptedEdges") != 1:
    raise SystemExit("expected one accepted precision edge")
if report.get("rejectedEdges") != 1:
    raise SystemExit("expected one rejected precision edge")
if "SECRET_SHOULD_NOT_PERSIST" in overlay:
    raise SystemExit("precision overlay leaked rejected source marker")
PY

precision_deps_json="$work_dir/precision-dependencies.json"
CTXPACK_HOME="$home" "$ctxpack_bin" dependencies src/main/kotlin/org/example/web/AuthController.kt --repo "$repo" --limit 10 >"$precision_deps_json"
python3 - "$precision_deps_json" <<'PY'
import json
import pathlib
import sys

edges = json.loads(pathlib.Path(sys.argv[1]).read_text())
if not any(edge["kind"] == "precision:calls" and edge["reason"] == "local precision smoke" for edge in edges):
    raise SystemExit("precision dependency edge was not returned")
PY

echo "precision smoke passed"
