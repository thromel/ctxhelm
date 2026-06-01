use crate::inventory::{normalize_input_path, InventoryError, RepoInventory};
use ctxhelm_core::{Diagnostic, DiagnosticSeverity, FileRole};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const POLICY_VERSION: &str = "ctxhelm-policy-v1";
pub const SOURCE_READ_MAX_BYTES: u64 = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceRead {
    pub path: String,
    pub status: SourceReadStatus,
    pub text: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceReadStatus {
    Read,
    Skipped(SourceReadReason),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceReadReason {
    PolicyExcluded,
    Binary,
    NonUtf8,
    Oversized,
    Unreadable,
}

pub fn read_safe_source(
    repo_root: impl AsRef<Path>,
    inventory: &RepoInventory,
    path: &str,
    max_bytes: u64,
) -> Result<SourceRead, InventoryError> {
    let repo_root = repo_root.as_ref();
    let normalized = normalize_input_path(repo_root, path);
    let role = classify_path(&normalized);
    let entry = inventory.files.iter().find(|file| file.path == normalized);

    if matches!(role, FileRole::Sensitive | FileRole::Generated)
        || entry.is_none()
        || entry.is_some_and(|file| {
            file.generated || matches!(file.role, FileRole::Sensitive | FileRole::Generated)
        })
    {
        return Ok(skipped(
            normalized,
            SourceReadReason::PolicyExcluded,
            "source_policy_excluded",
            "Source file was excluded by ctxhelm source policy.",
            DiagnosticSeverity::Warning,
        ));
    }

    if entry.is_some_and(|file| file.size_bytes > max_bytes) {
        return Ok(skipped(
            normalized,
            SourceReadReason::Oversized,
            "source_oversized",
            "Source file exceeded the safe read size limit.",
            DiagnosticSeverity::Warning,
        ));
    }

    let absolute = repo_root.join(&normalized);
    let bytes = match fs::read(&absolute) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(skipped(
                normalized,
                SourceReadReason::Unreadable,
                "source_unreadable",
                "Source file could not be read.",
                DiagnosticSeverity::Warning,
            ));
        }
    };

    if bytes.contains(&0) {
        return Ok(skipped(
            normalized,
            SourceReadReason::Binary,
            "source_binary",
            "Source file appeared to be binary data.",
            DiagnosticSeverity::Warning,
        ));
    }

    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return Ok(skipped(
                normalized,
                SourceReadReason::NonUtf8,
                "source_non_utf8",
                "Source file was not valid UTF-8.",
                DiagnosticSeverity::Warning,
            ));
        }
    };

    Ok(SourceRead {
        path: normalized,
        status: SourceReadStatus::Read,
        text: Some(text),
        diagnostics: Vec::new(),
    })
}

fn skipped(
    path: String,
    reason: SourceReadReason,
    code: &str,
    message: &str,
    severity: DiagnosticSeverity,
) -> SourceRead {
    SourceRead {
        path: path.clone(),
        status: SourceReadStatus::Skipped(reason),
        text: None,
        diagnostics: vec![Diagnostic {
            code: code.to_string(),
            severity,
            message: message.to_string(),
            paths: vec![path],
            count: 1,
        }],
    }
}

pub fn classify_path(path: &str) -> FileRole {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());

    if is_sensitive_path(&lower, name) {
        return FileRole::Sensitive;
    }
    if is_generated_path(&lower, name) {
        return FileRole::Generated;
    }
    if is_test_path(&lower, name) {
        return FileRole::Test;
    }
    if is_schema_path(&lower, name) {
        return FileRole::Schema;
    }
    if is_config_path(&lower, name) {
        return FileRole::Config;
    }
    if is_docs_path(&lower, name) {
        return FileRole::Docs;
    }
    if language_for_path(path).is_some() {
        return FileRole::Source;
    }

    FileRole::Unknown
}

fn is_sensitive_path(lower: &str, name: &str) -> bool {
    name == ".env"
        || name.starts_with(".env.")
        || name == ".npmrc"
        || name == ".yarnrc.yml"
        || name == ".pypirc"
        || name == ".netrc"
        || matches!(name, "id_rsa" | "id_ed25519" | "credentials")
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name.ends_with(".p12")
        || name.ends_with(".pfx")
        || name.ends_with(".crt")
        || name.ends_with(".cer")
        || name.ends_with(".dump")
        || name.ends_with(".sql.gz")
        || name == "terraform.tfstate"
        || name.ends_with(".tfstate")
        || name == "serviceaccountkey.json"
        || name.contains("firebase-adminsdk")
        || lower.contains("/.env.")
        || lower.starts_with(".ssh/")
        || lower.contains("/.ssh/")
        || lower == ".aws/credentials"
        || lower.ends_with("/.aws/credentials")
        || lower.contains("secret")
        || lower.contains("credentials")
}

fn is_generated_path(lower: &str, name: &str) -> bool {
    lower.contains("/node_modules/")
        || lower.contains("/.gradle/")
        || lower.contains("/.ctxhelm/cache/")
        || lower.ends_with("/.ctxhelm/eval-history.json")
        || lower.contains("/.fastembed_cache/")
        || lower.contains("/src/main 2/")
        || lower.contains("/src/test 2/")
        || lower.contains("/target/")
        || lower.contains("/dist/")
        || lower.contains("/build/")
        || lower.contains("/build ")
        || lower.contains("/coverage/")
        || lower.contains("/vendor/")
        || lower.contains("/resources/astdiff/")
        || lower.contains("/resources/mappings/")
        || lower.contains("/resources/oracle/")
        || lower.contains("/resources/web/monaco/")
        || lower.starts_with("node_modules/")
        || lower.starts_with(".gradle/")
        || lower.starts_with(".ctxhelm/cache/")
        || lower == ".ctxhelm/eval-history.json"
        || lower.starts_with(".fastembed_cache/")
        || lower.starts_with("src/main 2/")
        || lower.starts_with("src/test 2/")
        || lower.starts_with("target/")
        || lower.starts_with("dist/")
        || lower.starts_with("build/")
        || lower.starts_with("build ")
        || lower.starts_with("coverage/")
        || lower.starts_with("vendor/")
        || name.ends_with(".min.js")
        || name.ends_with(".min.css")
        || name == "package-lock.json"
        || name == "pnpm-lock.yaml"
        || name == "yarn.lock"
        || name == "cargo.lock"
}

fn is_test_path(lower: &str, name: &str) -> bool {
    lower.contains("/tests/")
        || lower.contains("/test/")
        || lower.contains("/spec/")
        || lower.starts_with("tests/")
        || lower.starts_with("test/")
        || lower.starts_with("spec/")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.ends_with("_test.go")
        || name.ends_with("_test.py")
        || name.starts_with("test_")
}

fn is_config_path(lower: &str, name: &str) -> bool {
    matches!(
        name,
        "package.json"
            | "pyproject.toml"
            | "cargo.toml"
            | "go.mod"
            | "go.sum"
            | "tsconfig.json"
            | "vite.config.ts"
            | "next.config.js"
            | "dockerfile"
            | "compose.yaml"
            | "docker-compose.yml"
            | "ctxhelm.toml"
    ) || lower.ends_with(".config.js")
        || lower.ends_with(".config.ts")
        || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
}

fn is_schema_path(lower: &str, name: &str) -> bool {
    lower.contains("/migrations/")
        || lower.contains("/schema/")
        || name.ends_with(".graphql")
        || name.ends_with(".graphqls")
        || name.ends_with(".proto")
        || name.ends_with(".prisma")
        || name.ends_with(".sql")
}

fn is_docs_path(lower: &str, name: &str) -> bool {
    lower.starts_with("docs/")
        || name == "readme.md"
        || name == "agents.md"
        || name.ends_with(".md")
        || name.ends_with(".mdx")
        || name.ends_with(".rst")
        || name.ends_with(".txt")
}

pub(crate) fn language_for_path(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(lower.as_str());
    let extension = name.rsplit_once('.').map(|(_, extension)| extension);

    match extension {
        Some("rs") => Some("rust"),
        Some("ts") | Some("tsx") => Some("typescript"),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => Some("javascript"),
        Some("py") => Some("python"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("kt") | Some("kts") => Some("kotlin"),
        Some("scala") => Some("scala"),
        Some("cs") => Some("csharp"),
        Some("rb") => Some("ruby"),
        Some("php") => Some("php"),
        Some("dart") => Some("dart"),
        Some("c") | Some("h") => Some("c"),
        Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") => Some("cpp"),
        Some("swift") => Some("swift"),
        Some("sql") => Some("sql"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_inventory, FileInventoryEntry, InventoryMetadata, InventoryOptions, RepoInventory,
    };
    use ctxhelm_core::{DiagnosticSeverity, FileRole};
    use std::fs;

    #[test]
    fn policy_classifies_common_credentials_and_generated_families() {
        for path in [
            ".npmrc",
            ".yarnrc.yml",
            ".pypirc",
            ".netrc",
            ".ssh/id_rsa",
            ".ssh/id_ed25519",
            "config/serviceAccountKey.json",
            "config/firebase-adminsdk-prod.json",
            ".aws/credentials",
            "infra/terraform.tfstate",
        ] {
            assert_eq!(classify_path(path), FileRole::Sensitive, "{path}");
        }

        for path in [
            "node_modules/react/index.js",
            ".ctxhelm/cache/fastembed/models--jinaai--jina-embeddings-v2-base-code/blobs/model.onnx",
            ".ctxhelm/eval-history.json",
            ".fastembed_cache/models--jinaai--jina-embeddings-v2-base-code/blobs/model.onnx",
            "src/main 2/java/org/example/Copied.java",
            "src/test 2/java/org/example/CopiedTest.java",
            "target/debug/app",
            "dist/app.js",
            "build/output.js",
            "coverage/lcov.info",
            "vendor/rack/lib.rb",
            "assets/app.min.css",
            "package-lock.json",
            "Cargo.lock",
        ] {
            assert_eq!(classify_path(path), FileRole::Generated, "{path}");
        }

        assert_eq!(classify_path("src/lib.rs"), FileRole::Source);
        assert_eq!(classify_path("package.json"), FileRole::Config);
    }

    #[test]
    fn read_safe_source_reads_utf8_inventory_entries() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn answer() -> u8 { 42 }\n").unwrap();
        let inventory = build_inventory(repo, &InventoryOptions::default()).unwrap();

        let source = read_safe_source(repo, &inventory, "src/lib.rs", 1024).unwrap();

        assert_eq!(source.status, SourceReadStatus::Read);
        assert_eq!(source.path, "src/lib.rs");
        assert_eq!(
            source.text.as_deref(),
            Some("pub fn answer() -> u8 { 42 }\n")
        );
        assert!(source.diagnostics.is_empty());
    }

    #[test]
    fn read_safe_source_skips_policy_excluded_paths_without_source_text() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::write(
            repo.join(".npmrc"),
            "//registry.npmjs.org/:_authToken=secret\n",
        )
        .unwrap();
        let inventory = build_inventory(repo, &InventoryOptions::default()).unwrap();

        let source = read_safe_source(repo, &inventory, ".npmrc", 1024).unwrap();

        assert_eq!(
            source.status,
            SourceReadStatus::Skipped(SourceReadReason::PolicyExcluded)
        );
        assert!(source.text.is_none());
        assert_eq!(source.diagnostics[0].code, "source_policy_excluded");
        assert_eq!(source.diagnostics[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(source.diagnostics[0].paths, vec![".npmrc"]);
    }

    #[test]
    fn read_safe_source_reports_binary_non_utf8_oversized_and_unreadable() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/binary.rs"), b"pub fn bad() {}\0\n").unwrap();
        fs::write(repo.join("src/non_utf8.rs"), [0xff, 0xfe, b'\n']).unwrap();
        fs::write(repo.join("src/large.rs"), "0123456789\n").unwrap();
        let inventory = RepoInventory {
            repo_id: "repo-1".to_string(),
            repo_root: repo.to_path_buf(),
            metadata: InventoryMetadata::default(),
            files: vec![
                entry("src/binary.rs", 16),
                entry("src/non_utf8.rs", 3),
                entry("src/large.rs", 11),
                entry("src/missing.rs", 1),
            ],
            ignored_count: 0,
            generated_count: 0,
            sensitive_count: 0,
        };

        let binary = read_safe_source(repo, &inventory, "src/binary.rs", 1024).unwrap();
        assert_eq!(
            binary.status,
            SourceReadStatus::Skipped(SourceReadReason::Binary)
        );
        assert!(binary.text.is_none());
        assert_eq!(binary.diagnostics[0].code, "source_binary");

        let non_utf8 = read_safe_source(repo, &inventory, "src/non_utf8.rs", 1024).unwrap();
        assert_eq!(
            non_utf8.status,
            SourceReadStatus::Skipped(SourceReadReason::NonUtf8)
        );
        assert!(non_utf8.text.is_none());
        assert_eq!(non_utf8.diagnostics[0].code, "source_non_utf8");

        let oversized = read_safe_source(repo, &inventory, "src/large.rs", 4).unwrap();
        assert_eq!(
            oversized.status,
            SourceReadStatus::Skipped(SourceReadReason::Oversized)
        );
        assert!(oversized.text.is_none());
        assert_eq!(oversized.diagnostics[0].code, "source_oversized");

        let unreadable = read_safe_source(repo, &inventory, "src/missing.rs", 1024).unwrap();
        assert_eq!(
            unreadable.status,
            SourceReadStatus::Skipped(SourceReadReason::Unreadable)
        );
        assert!(unreadable.text.is_none());
        assert_eq!(unreadable.diagnostics[0].code, "source_unreadable");
    }

    fn entry(path: &str, size_bytes: u64) -> FileInventoryEntry {
        FileInventoryEntry {
            path: path.to_string(),
            language: Some("rust".to_string()),
            role: FileRole::Source,
            hash: "hash".to_string(),
            size_bytes,
            generated: false,
            ignored: false,
        }
    }
}
