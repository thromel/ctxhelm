use crate::inventory::{canonicalize, ctxhelm_home, repo_id_for_path, InventoryError};
use ctxhelm_core::{Diagnostic, DiagnosticSeverity, EvalTrace, TraceStatus, TraceStatusKind};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn append_eval_trace(
    repo_root: impl AsRef<Path>,
    trace: &EvalTrace,
) -> Result<PathBuf, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = trace_path(&repo_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InventoryError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| InventoryError::Write {
            path: path.clone(),
            source,
        })?;
    let json = serde_json::to_string(trace).map_err(InventoryError::Serialize)?;
    writeln!(file, "{json}").map_err(|source| InventoryError::Write {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub fn try_append_eval_trace(repo_root: impl AsRef<Path>, trace: &EvalTrace) -> TraceStatus {
    let path = trace_path_for_repo_root(repo_root.as_ref());
    match append_eval_trace(repo_root, trace) {
        Ok(path) => TraceStatus {
            status: TraceStatusKind::Written,
            path: Some(path.display().to_string()),
            diagnostics: Vec::new(),
        },
        Err(error) => TraceStatus {
            status: TraceStatusKind::WriteFailed,
            path: path.map(|path| path.display().to_string()),
            diagnostics: vec![Diagnostic {
                code: "trace_write_failed".to_string(),
                severity: DiagnosticSeverity::Warning,
                message: format!("Eval trace was not recorded: {error}"),
                paths: Vec::new(),
                count: 1,
            }],
        },
    }
}

pub fn list_eval_traces(
    repo_root: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<EvalTrace>, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let repo_id = repo_id_for_path(&repo_root);
    let path = trace_path(&repo_id);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(InventoryError::Read {
                path: path.clone(),
                source,
            })
        }
    };

    let mut traces = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<EvalTrace>(line).map_err(|source| InventoryError::Deserialize {
                path: path.clone(),
                source,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    traces.reverse();
    traces.truncate(limit.max(1));
    Ok(traces)
}

pub fn trace_path(repo_id: &str) -> PathBuf {
    ctxhelm_home()
        .join("repos")
        .join(repo_id)
        .join("traces.jsonl")
}

fn trace_path_for_repo_root(repo_root: &Path) -> Option<PathBuf> {
    let repo_root = canonicalize(repo_root).ok()?;
    let repo_id = repo_id_for_path(&repo_root);
    Some(trace_path(&repo_id))
}
