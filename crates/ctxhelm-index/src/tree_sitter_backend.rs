use crate::inventory::FileInventoryEntry;
use crate::symbols::{CodeSymbol, SymbolKind};

#[cfg(feature = "tree-sitter-parsers")]
mod imp {
    use super::*;
    use tree_sitter::{Language, Node, Parser};

    pub(crate) fn symbols_for_file(
        file: &FileInventoryEntry,
        content: &str,
    ) -> Option<Vec<CodeSymbol>> {
        let language = language_for_file(file)?;
        let mut parser = Parser::new();
        parser.set_language(&language).ok()?;
        let tree = parser.parse(content, None)?;
        let mut symbols = Vec::new();
        collect_symbols(file, content, tree.root_node(), &mut symbols);
        Some(symbols)
    }

    fn language_for_file(file: &FileInventoryEntry) -> Option<Language> {
        match file.language.as_deref()? {
            "typescript" => {
                if file.path.ends_with(".tsx") {
                    Some(tree_sitter_typescript::LANGUAGE_TSX.into())
                } else {
                    Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                }
            }
            "javascript" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            "java" => Some(tree_sitter_java::LANGUAGE.into()),
            "kotlin" => Some(tree_sitter_kotlin_ng::LANGUAGE.into()),
            _ => None,
        }
    }

    fn collect_symbols(
        file: &FileInventoryEntry,
        content: &str,
        node: Node,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        if let Some((name, kind)) = symbol_node_name_and_kind(file, node, content) {
            symbols.push(code_symbol(file, content, node, &name, kind));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_symbols(file, content, child, symbols);
        }
    }

    fn symbol_node_name_and_kind(
        file: &FileInventoryEntry,
        node: Node,
        content: &str,
    ) -> Option<(String, SymbolKind)> {
        let kind = match node.kind() {
            "function_definition" if file.language.as_deref() == Some("python") => {
                if has_parent_kind(node, "class_definition") {
                    SymbolKind::Method
                } else {
                    SymbolKind::Function
                }
            }
            "function_declaration" | "function_definition" | "function_item" => {
                SymbolKind::Function
            }
            "method_definition" | "method_declaration" => SymbolKind::Method,
            "class_declaration" | "class_definition" => SymbolKind::Class,
            "interface_declaration" | "trait_item" => SymbolKind::Interface,
            "type_alias_declaration"
            | "type_alias"
            | "type_item"
            | "enum_declaration"
            | "enum_item"
            | "record_declaration"
            | "type_declaration"
            | "type_spec" => SymbolKind::Type,
            "struct_item" => SymbolKind::Class,
            "const_item"
            | "static_item"
            | "lexical_declaration"
            | "variable_declarator"
            | "field_declaration"
            | "property_declaration" => SymbolKind::Constant,
            "mod_item" | "module" | "object_declaration" => SymbolKind::Module,
            _ => return None,
        };
        let name = node
            .child_by_field_name("name")
            .and_then(|child| child_text(child, content))
            .or_else(|| child_identifier_text(node, content))?;
        Some((name, kind))
    }

    fn has_parent_kind(mut node: Node, kind: &str) -> bool {
        while let Some(parent) = node.parent() {
            if parent.kind() == kind {
                return true;
            }
            node = parent;
        }
        false
    }

    fn child_identifier_text(node: Node, content: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(
                child.kind(),
                "identifier" | "type_identifier" | "property_identifier"
            ) {
                return child_text(child, content);
            }
        }
        None
    }

    fn child_text(node: Node, content: &str) -> Option<String> {
        node.utf8_text(content.as_bytes())
            .ok()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn code_symbol(
        file: &FileInventoryEntry,
        content: &str,
        node: Node,
        name: &str,
        kind: SymbolKind,
    ) -> CodeSymbol {
        let start_line = node.start_position().row as u32 + 1;
        let end_line = node.end_position().row as u32 + 1;
        let signature = signature_for_node(content, node, start_line);
        CodeSymbol {
            name: name.to_string(),
            kind,
            path: file.path.clone(),
            language: file.language.clone(),
            start_line,
            end_line: end_line.max(start_line),
            exported: exported_symbol(file, name, &signature),
            signature,
        }
    }

    fn signature_for_node(content: &str, node: Node, start_line: u32) -> String {
        let first_line = content
            .lines()
            .nth(start_line.saturating_sub(1) as usize)
            .unwrap_or("")
            .trim();
        if !first_line.is_empty() {
            return first_line.chars().take(200).collect();
        }
        node.utf8_text(content.as_bytes())
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .chars()
            .take(200)
            .collect()
    }

    fn exported_symbol(file: &FileInventoryEntry, name: &str, signature: &str) -> bool {
        let signature = signature.trim();
        match file.language.as_deref() {
            Some("typescript" | "javascript") => signature.starts_with("export "),
            Some("rust") => signature.starts_with("pub "),
            Some("java") => signature.starts_with("public "),
            Some("kotlin") => !signature.starts_with("private "),
            Some("go") => name
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_uppercase()),
            _ => false,
        }
    }
}

#[cfg(not(feature = "tree-sitter-parsers"))]
mod imp {
    use super::*;

    pub(crate) fn symbols_for_file(
        _file: &FileInventoryEntry,
        _content: &str,
    ) -> Option<Vec<CodeSymbol>> {
        None
    }
}

pub(crate) use imp::symbols_for_file;
