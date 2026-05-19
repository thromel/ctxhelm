use crate::freshness::load_or_refresh_inventory;
use crate::inventory::{canonicalize, FileInventoryEntry, InventoryError, InventoryOptions};
use crate::policy::{read_safe_source, SourceReadStatus, SOURCE_READ_MAX_BYTES};
use crate::search::{query_term_weight, query_terms};
use ctxpack_core::{CacheStatus, Diagnostic, DiagnosticSeverity, FileRole};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolOptions {
    pub limit: usize,
}

impl Default for SymbolOptions {
    fn default() -> Self {
        Self { limit: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Type,
    Constant,
    Import,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub path: String,
    pub language: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    pub signature: String,
    pub exported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchResult {
    pub symbol: CodeSymbol,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolExtractionReport {
    pub symbols: Vec<CodeSymbol>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchReport {
    pub results: Vec<SymbolSearchResult>,
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    pub cache_status: CacheStatus,
}

pub fn extract_symbols(repo_root: impl AsRef<Path>) -> Result<Vec<CodeSymbol>, InventoryError> {
    Ok(extract_symbols_report(repo_root)?.symbols)
}

pub fn extract_symbols_report(
    repo_root: impl AsRef<Path>,
) -> Result<SymbolExtractionReport, InventoryError> {
    let repo_root = canonicalize(repo_root.as_ref())?;
    let inventory_report = load_or_refresh_inventory(&repo_root, &InventoryOptions::default())?;
    let mut diagnostics = inventory_report.diagnostics.clone();
    let cache_status = inventory_report.cache_status.clone();
    let mut symbols = Vec::new();

    for file in &inventory_report.inventory.files {
        if file.generated || file.role == FileRole::Sensitive || file.ignored {
            continue;
        }
        if file.language.is_none() {
            continue;
        }

        let source = read_safe_source(
            &repo_root,
            &inventory_report.inventory,
            &file.path,
            SOURCE_READ_MAX_BYTES,
        )?;
        if !source.diagnostics.is_empty() {
            diagnostics.extend(source.diagnostics);
            diagnostics.push(partial_diagnostic(
                "parse_gap",
                "Symbol extraction skipped at least one source file because it could not be safely read.",
                vec![file.path.clone()],
            ));
        }
        let SourceReadStatus::Read = source.status else {
            continue;
        };
        let content = source.text.unwrap_or_default();
        symbols.extend(symbols_for_file(&file, &content));
    }

    symbols.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.start_line.cmp(&right.start_line))
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(SymbolExtractionReport {
        symbols,
        diagnostics,
        cache_status,
    })
}

fn partial_diagnostic(code: &str, message: &str, paths: Vec<String>) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: message.to_string(),
        count: paths.len(),
        paths,
    }
}

pub fn symbol_search(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SymbolOptions,
) -> Result<Vec<SymbolSearchResult>, InventoryError> {
    Ok(symbol_search_report(repo_root, query, options)?.results)
}

pub fn symbol_search_report(
    repo_root: impl AsRef<Path>,
    query: &str,
    options: &SymbolOptions,
) -> Result<SymbolSearchReport, InventoryError> {
    let query_terms = query_terms(query);
    let extraction = extract_symbols_report(repo_root)?;
    if query_terms.is_empty() {
        return Ok(SymbolSearchReport {
            results: Vec::new(),
            diagnostics: extraction.diagnostics,
            cache_status: extraction.cache_status,
        });
    }

    let mut results = extraction
        .symbols
        .into_iter()
        .filter_map(|symbol| score_symbol(symbol, &query_terms))
        .collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.symbol.path.cmp(&right.symbol.path))
            .then_with(|| left.symbol.start_line.cmp(&right.symbol.start_line))
    });
    results.truncate(options.limit.max(1));
    Ok(SymbolSearchReport {
        results,
        diagnostics: extraction.diagnostics,
        cache_status: extraction.cache_status,
    })
}

fn symbols_for_file(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    if let Some(mut symbols) = crate::tree_sitter_backend::symbols_for_file(file, content) {
        symbols.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then_with(|| left.start_line.cmp(&right.start_line))
                .then_with(|| left.end_line.cmp(&right.end_line))
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
        });
        symbols.dedup_by(|left, right| {
            left.path == right.path
                && left.name == right.name
                && left.kind == right.kind
                && left.start_line == right.start_line
        });
        if !symbols.is_empty() {
            return symbols;
        }
    }

    match file.language.as_deref() {
        Some("typescript" | "javascript") => symbols_for_js_like(file, content),
        Some("python") => symbols_for_python(file, content),
        Some("rust") => symbols_for_rust(file, content),
        Some("go") => symbols_for_go(file, content),
        Some("java") => symbols_for_java(file, content),
        Some("kotlin") => symbols_for_kotlin(file, content),
        _ => Vec::new(),
    }
}

fn symbols_for_js_like(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("import ") {
            if let Some(name) = import_name(trimmed) {
                symbols.push(code_symbol(
                    file,
                    name,
                    SymbolKind::Import,
                    line_no,
                    trimmed,
                    false,
                ));
            }
            continue;
        }

        let exported = trimmed.starts_with("export ");
        let rest = strip_modifiers(
            trimmed,
            &[
                "export",
                "default",
                "declare",
                "async",
                "public",
                "private",
                "protected",
                "static",
                "readonly",
            ],
        );
        if let Some(name) = identifier_after(rest, "function ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Function,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "interface ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Interface,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "type ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Type,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = variable_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Constant,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = js_method_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Method,
                line_no,
                trimmed,
                exported,
            ));
        }
    }
    symbols
}

fn symbols_for_python(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len().saturating_sub(line.trim_start().len());
        let rest = strip_modifiers(trimmed, &["async"]);
        if let Some(name) = identifier_after(rest, "def ") {
            let kind = if indent > 0 {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            };
            symbols.push(code_symbol(file, name, kind, line_no, trimmed, false));
        } else if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                false,
            ));
        }
    }
    symbols
}

fn symbols_for_rust(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#[") {
            continue;
        }
        let exported = trimmed.starts_with("pub ");
        let rest = strip_modifiers(trimmed, &["pub", "async", "unsafe"]);
        let candidates = [
            ("fn ", SymbolKind::Function),
            ("struct ", SymbolKind::Class),
            ("enum ", SymbolKind::Type),
            ("trait ", SymbolKind::Interface),
            ("type ", SymbolKind::Type),
            ("const ", SymbolKind::Constant),
            ("static ", SymbolKind::Constant),
            ("mod ", SymbolKind::Module),
        ];
        for (prefix, kind) in candidates {
            if let Some(name) = identifier_after(rest, prefix) {
                symbols.push(code_symbol(file, name, kind, line_no, trimmed, exported));
                break;
            }
        }
    }
    symbols
}

fn symbols_for_go(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(name) = go_func_name(trimmed) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Function,
                line_no,
                trimmed,
                is_exported_go(name),
            ));
        } else if let Some(name) = identifier_after(trimmed, "type ") {
            let kind = if trimmed.contains(" interface") {
                SymbolKind::Interface
            } else {
                SymbolKind::Type
            };
            symbols.push(code_symbol(
                file,
                name,
                kind,
                line_no,
                trimmed,
                is_exported_go(name),
            ));
        }
    }
    symbols
}

fn symbols_for_java(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('@') {
            continue;
        }
        let exported = java_is_public(trimmed);
        let rest = strip_modifiers(
            trimmed,
            &[
                "public",
                "protected",
                "private",
                "abstract",
                "static",
                "final",
                "sealed",
                "non-sealed",
                "strictfp",
                "synchronized",
                "native",
                "default",
            ],
        );
        if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "interface ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Interface,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "enum ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Type,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "record ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Type,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = java_constant_name(trimmed) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Constant,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = java_method_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Method,
                line_no,
                trimmed,
                exported,
            ));
        }
    }
    symbols
}

fn symbols_for_kotlin(file: &FileInventoryEntry, content: &str) -> Vec<CodeSymbol> {
    let mut symbols = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        let line_no = line_number(line_index);
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with('@')
            || trimmed == "}"
        {
            continue;
        }
        let exported = !trimmed.starts_with("private ");
        let rest = strip_modifiers(
            trimmed,
            &[
                "public",
                "internal",
                "private",
                "protected",
                "open",
                "abstract",
                "sealed",
                "data",
                "value",
                "inner",
                "companion",
                "inline",
                "suspend",
                "operator",
                "override",
                "tailrec",
            ],
        );
        if let Some(name) = identifier_after(rest, "class ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Class,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "interface ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Interface,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "object ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Module,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "typealias ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Type,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = identifier_after(rest, "fun ") {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Function,
                line_no,
                trimmed,
                exported,
            ));
        } else if let Some(name) = kotlin_value_name(rest) {
            symbols.push(code_symbol(
                file,
                name,
                SymbolKind::Constant,
                line_no,
                trimmed,
                exported,
            ));
        }
    }
    symbols
}

fn code_symbol(
    file: &FileInventoryEntry,
    name: &str,
    kind: SymbolKind,
    line_no: u32,
    signature: &str,
    exported: bool,
) -> CodeSymbol {
    CodeSymbol {
        name: name.to_string(),
        kind,
        path: file.path.clone(),
        language: file.language.clone(),
        start_line: line_no,
        end_line: line_no,
        signature: signature.chars().take(200).collect(),
        exported,
    }
}

fn score_symbol(symbol: CodeSymbol, query_terms: &[String]) -> Option<SymbolSearchResult> {
    let name = symbol.name.to_ascii_lowercase();
    let path = symbol.path.to_ascii_lowercase();
    let signature = symbol.signature.to_ascii_lowercase();
    let mut score = 0.0;
    let mut reasons = Vec::new();

    for term in query_terms {
        let weight = query_term_weight(term);
        if weight == 0.0 {
            continue;
        }
        let mut matched = false;
        if name == *term {
            score += 24.0 * weight;
            matched = true;
            reasons.push(format!("symbol name exactly matched `{term}`"));
        } else if name.starts_with(term) {
            score += 16.0 * weight;
            matched = true;
            reasons.push(format!("symbol name starts with `{term}`"));
        } else if name.contains(term) {
            score += 12.0 * weight;
            matched = true;
            reasons.push(format!("symbol name contains `{term}`"));
        }
        if path.contains(term) {
            score += 4.0 * weight;
            matched = true;
            reasons.push(format!("path contains `{term}`"));
        }
        if signature.contains(term) {
            score += 2.0 * weight;
            matched = true;
            reasons.push(format!("signature contains `{term}`"));
        }
        if !matched {
            score -= 2.0 * weight;
        }
    }

    if symbol.exported {
        score += 1.0;
    }
    if score <= 0.0 {
        return None;
    }

    reasons.sort();
    reasons.dedup();
    Some(SymbolSearchResult {
        symbol,
        score,
        reason: reasons.join("; "),
    })
}

fn line_number(line_index: usize) -> u32 {
    u32::try_from(line_index + 1).unwrap_or(u32::MAX)
}

fn strip_modifiers<'a>(line: &'a str, modifiers: &[&str]) -> &'a str {
    let mut rest = line.trim();
    loop {
        let mut changed = false;
        for modifier in modifiers {
            if let Some(next) = rest.strip_prefix(modifier).and_then(|value| {
                value
                    .starts_with(char::is_whitespace)
                    .then_some(value.trim_start())
            }) {
                rest = next;
                changed = true;
            }
        }
        if !changed {
            return rest;
        }
    }
}

fn identifier_after<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(prefix)?;
    take_identifier(rest)
}

fn take_identifier(input: &str) -> Option<&str> {
    let input = input.trim_start();
    let end = input
        .char_indices()
        .find_map(|(index, character)| {
            (!is_identifier_char(character) && index > 0).then_some(index)
        })
        .unwrap_or(input.len());
    let identifier = &input[..end];
    (!identifier.is_empty()).then_some(identifier)
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '$'
}

fn variable_name(line: &str) -> Option<&str> {
    ["const ", "let ", "var "]
        .into_iter()
        .find_map(|prefix| identifier_after(line, prefix))
}

fn js_method_name(line: &str) -> Option<&str> {
    let disallowed = [
        "if ",
        "for ",
        "while ",
        "switch ",
        "catch ",
        "return ",
        "function ",
    ];
    if disallowed.iter().any(|prefix| line.starts_with(prefix)) {
        return None;
    }
    let open_paren = line.find('(')?;
    let before = line[..open_paren].trim();
    if before.contains('=') || before.contains(' ') || before.is_empty() {
        return None;
    }
    take_identifier(before)
}

fn import_name(line: &str) -> Option<&str> {
    if let Some((_, module)) = line.split_once(" from ") {
        return quoted_value(module);
    }
    line.strip_prefix("import ").and_then(quoted_value)
}

fn quoted_value(input: &str) -> Option<&str> {
    let start = input.find(['"', '\''])?;
    let quote = input.as_bytes()[start] as char;
    let rest = &input[start + 1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn go_func_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("func ")?;
    let rest = if rest.starts_with('(') {
        let close = rest.find(')')?;
        rest[close + 1..].trim_start()
    } else {
        rest
    };
    take_identifier(rest)
}

fn is_exported_go(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

fn java_is_public(line: &str) -> bool {
    line.starts_with("public ") || line.contains(" public ")
}

fn java_constant_name(line: &str) -> Option<&str> {
    if !line.contains(" static ") && !line.starts_with("static ") {
        return None;
    }
    let before_equals = line.split('=').next()?.trim();
    let name = before_equals.split_whitespace().last()?;
    take_identifier(name)
}

fn java_method_name(line: &str) -> Option<&str> {
    let disallowed = [
        "if ", "for ", "while ", "switch ", "catch ", "return ", "throw ", "new ", "else ", "do ",
    ];
    if disallowed.iter().any(|prefix| line.starts_with(prefix)) {
        return None;
    }
    let open_paren = line.find('(')?;
    let before = line[..open_paren].trim();
    if before.is_empty() || before.contains('=') || before.ends_with('.') {
        return None;
    }
    let name = before.split_whitespace().last()?;
    take_identifier(name)
}

fn kotlin_value_name(line: &str) -> Option<&str> {
    ["val ", "var "]
        .into_iter()
        .find_map(|prefix| identifier_after(line, prefix))
}
