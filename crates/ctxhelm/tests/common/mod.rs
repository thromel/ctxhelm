use assert_cmd::assert::Assert;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CTXHELM_HOME_ENV: &str = "CTXHELM_HOME";

pub struct FixtureRepo {
    #[allow(dead_code)]
    pub temp: tempfile::TempDir,
    pub repo: PathBuf,
    pub home: PathBuf,
}

pub fn fixture_repo() -> FixtureRepo {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let home = temp.path().join("ctxhelm-home");

    fs::create_dir_all(repo.join("src/auth")).unwrap();
    fs::create_dir_all(repo.join("tests/auth")).unwrap();
    fs::create_dir_all(repo.join("dist")).unwrap();
    fs::write(
        repo.join("src/auth/session.ts"),
        r#"import { issueToken } from "./token";

export function requireSession(user?: { id: string }) {
  if (!user) {
    throw new Error("auth required");
  }
  issueToken(user.id);
  return user.id;
}
"#,
    )
    .unwrap();
    fs::write(
        repo.join("src/auth/token.ts"),
        r#"export function issueToken(userId: string) {
  return `token:${userId}`;
}
"#,
    )
    .unwrap();
    fs::write(
        repo.join("tests/auth/session.test.ts"),
        r#"import { requireSession } from "../../src/auth/session";

test("requireSession returns user id", () => {
  expect(requireSession({ id: "user-1" })).toBe("user-1");
});
"#,
    )
    .unwrap();
    fs::write(
        repo.join("package.json"),
        r#"{"scripts":{"test":"vitest run"}}"#,
    )
    .unwrap();
    fs::write(repo.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'\n").unwrap();
    fs::write(repo.join(".env"), "AUTH_SECRET=do-not-index\n").unwrap();
    fs::write(
        repo.join("dist/generated.min.js"),
        "function g(){return 1}\n",
    )
    .unwrap();

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "ctxhelm@example.com"]);
    run_git(&repo, &["config", "user.name", "ctxhelm"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "fixture"]);

    FixtureRepo { temp, repo, home }
}

pub fn run_git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn json_stdout(assert: Assert) -> Value {
    serde_json::from_slice(&assert.success().get_output().stdout).unwrap()
}
