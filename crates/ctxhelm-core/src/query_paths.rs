const KNOWN_QUERY_PATH_EXTENSIONS: &[&str] = &[
    ".bash", ".c", ".cc", ".cpp", ".cs", ".go", ".h", ".hpp", ".java", ".js", ".json", ".jsx",
    ".kt", ".md", ".mjs", ".py", ".rb", ".rs", ".sh", ".toml", ".ts", ".tsx", ".txt", ".yaml",
    ".yml", ".zsh",
];

pub fn explicit_query_paths(query: &str) -> Vec<String> {
    let mut paths = query
        .split_whitespace()
        .filter_map(|token| {
            let path = normalize_query_path_token(token);
            looks_like_explicit_query_path(&path).then_some(path)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

pub fn normalize_query_path_token(token: &str) -> String {
    let mut cleaned = token
        .trim_matches(|character: char| {
            character.is_ascii_punctuation()
                && !matches!(character, '/' | '.' | '_' | '-' | ':' | '\\')
        })
        .replace('\\', "/");
    while cleaned.ends_with(':') && !cleaned.ends_with("::") {
        cleaned.pop();
    }
    cleaned.trim_start_matches("./").to_ascii_lowercase()
}

pub fn looks_like_explicit_query_path(path: &str) -> bool {
    path.contains('/')
        && KNOWN_QUERY_PATH_EXTENSIONS
            .iter()
            .any(|extension| path.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_query_paths_extract_common_source_config_doc_and_text_paths() {
        let paths = explicit_query_paths(
            "fix `src/auth/session.ts`, docs/setup.md, config/app.yaml and src/orders/status.txt",
        );

        assert_eq!(
            paths,
            vec![
                "config/app.yaml",
                "docs/setup.md",
                "src/auth/session.ts",
                "src/orders/status.txt"
            ]
        );
    }

    #[test]
    fn explicit_query_paths_normalize_backslashes_and_trailing_colons() {
        let paths = explicit_query_paths("inspect .\\src\\auth\\session.ts:");

        assert_eq!(paths, vec!["src/auth/session.ts"]);
    }

    #[test]
    fn explicit_query_paths_leave_stack_frames_to_stack_frame_extraction() {
        assert!(explicit_query_paths("src/auth/session.ts:42").is_empty());
    }
}
