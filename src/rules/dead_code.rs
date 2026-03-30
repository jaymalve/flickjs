use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ExportDefaultDeclaration, ExportNamedDeclaration, ImportDeclaration,
    ImportDeclarationSpecifier,
};
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::{LintDiagnostic, RuleOrigin, Severity};

// ── Import graph types ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub has_side_effects: bool,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub source: String,
    pub names: Vec<ImportedName>,
    pub is_type_only: bool,
    pub is_side_effect: bool,
}

#[derive(Debug, Clone)]
pub enum ImportedName {
    Default,
    Named(String),
    Namespace,
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub name: String,
    pub is_default: bool,
    pub is_type_only: bool,
    pub span_start: u32,
    pub span_end: u32,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub struct ImportGraph {
    pub files: HashMap<PathBuf, FileInfo>,
    /// Maps a normalized import source to the resolved file path
    pub resolved_imports: HashMap<(PathBuf, String), Option<PathBuf>>,
}

// ── Analysis ───────────────────────────────────────────────

/// Collect import/export information from a parsed file
pub fn analyze_file(path: &Path, source: &str) -> FileInfo {
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);

    let mut imports = Vec::new();
    let mut exports = Vec::new();

    for node in semantic.semantic.nodes().iter() {
        match node.kind() {
            AstKind::ImportDeclaration(decl) => {
                imports.push(collect_import(source, decl));
            }
            AstKind::ExportNamedDeclaration(decl) => {
                exports.extend(collect_named_exports(source, decl));
            }
            AstKind::ExportDefaultDeclaration(decl) => {
                exports.push(collect_default_export(source, decl));
            }
            _ => {}
        }
    }

    FileInfo {
        path: path.to_path_buf(),
        imports,
        exports,
        has_side_effects: false,
    }
}

fn collect_import(source: &str, decl: &ImportDeclaration<'_>) -> ImportInfo {
    let source_value = decl.source.value.to_string();
    let is_type_only = decl.import_kind.is_type();

    let names = match &decl.specifiers {
        Some(specifiers) => specifiers
            .iter()
            .map(|spec| match spec {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => ImportedName::Default,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => ImportedName::Namespace,
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    ImportedName::Named(s.imported.name().to_string())
                }
            })
            .collect(),
        None => Vec::new(),
    };

    let is_side_effect = decl.specifiers.as_ref().map_or(true, |s| s.is_empty());

    ImportInfo {
        source: source_value,
        names,
        is_type_only,
        is_side_effect,
    }
}

fn collect_named_exports(source: &str, decl: &ExportNamedDeclaration<'_>) -> Vec<ExportInfo> {
    let mut exports = Vec::new();

    // Re-exports: export { foo } from './bar'
    if decl.source.is_some() {
        for spec in &decl.specifiers {
            let (line, col) = offset_to_line_col(source, spec.span.start as usize);
            exports.push(ExportInfo {
                name: spec.exported.name().to_string(),
                is_default: false,
                is_type_only: decl.export_kind.is_type(),
                span_start: spec.span.start,
                span_end: spec.span.end,
                line,
                col,
            });
        }
        return exports;
    }

    // Named specifiers: export { foo, bar }
    for spec in &decl.specifiers {
        let (line, col) = offset_to_line_col(source, spec.span.start as usize);
        exports.push(ExportInfo {
            name: spec.exported.name().to_string(),
            is_default: false,
            is_type_only: decl.export_kind.is_type(),
            span_start: spec.span.start,
            span_end: spec.span.end,
            line,
            col,
        });
    }

    // Declaration exports: export const foo = ..., export function bar() {}
    if let Some(declaration) = &decl.declaration {
        match declaration {
            oxc_ast::ast::Declaration::VariableDeclaration(var_decl) => {
                for declarator in &var_decl.declarations {
                    for binding in declarator.id.get_binding_identifiers() {
                        let (line, col) = offset_to_line_col(source, binding.span.start as usize);
                        exports.push(ExportInfo {
                            name: binding.name.to_string(),
                            is_default: false,
                            is_type_only: false,
                            span_start: binding.span.start,
                            span_end: binding.span.end,
                            line,
                            col,
                        });
                    }
                }
            }
            oxc_ast::ast::Declaration::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    let (line, col) = offset_to_line_col(source, id.span.start as usize);
                    exports.push(ExportInfo {
                        name: id.name.to_string(),
                        is_default: false,
                        is_type_only: false,
                        span_start: id.span.start,
                        span_end: id.span.end,
                        line,
                        col,
                    });
                }
            }
            oxc_ast::ast::Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    let (line, col) = offset_to_line_col(source, id.span.start as usize);
                    exports.push(ExportInfo {
                        name: id.name.to_string(),
                        is_default: false,
                        is_type_only: false,
                        span_start: id.span.start,
                        span_end: id.span.end,
                        line,
                        col,
                    });
                }
            }
            oxc_ast::ast::Declaration::TSTypeAliasDeclaration(alias) => {
                let (line, col) = offset_to_line_col(source, alias.id.span.start as usize);
                exports.push(ExportInfo {
                    name: alias.id.name.to_string(),
                    is_default: false,
                    is_type_only: true,
                    span_start: alias.id.span.start,
                    span_end: alias.id.span.end,
                    line,
                    col,
                });
            }
            oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
                let (line, col) = offset_to_line_col(source, iface.id.span.start as usize);
                exports.push(ExportInfo {
                    name: iface.id.name.to_string(),
                    is_default: false,
                    is_type_only: true,
                    span_start: iface.id.span.start,
                    span_end: iface.id.span.end,
                    line,
                    col,
                });
            }
            oxc_ast::ast::Declaration::TSEnumDeclaration(en) => {
                let (line, col) = offset_to_line_col(source, en.id.span.start as usize);
                exports.push(ExportInfo {
                    name: en.id.name.to_string(),
                    is_default: false,
                    is_type_only: false,
                    span_start: en.id.span.start,
                    span_end: en.id.span.end,
                    line,
                    col,
                });
            }
            _ => {}
        }
    }

    exports
}

fn collect_default_export(source: &str, decl: &ExportDefaultDeclaration<'_>) -> ExportInfo {
    let (line, col) = offset_to_line_col(source, decl.span.start as usize);
    ExportInfo {
        name: "default".to_string(),
        is_default: true,
        is_type_only: false,
        span_start: decl.span.start,
        span_end: decl.span.end,
        line,
        col,
    }
}

// ── Import resolution ──────────────────────────────────────

/// Try to resolve a relative import to a file path.
/// Handles: ./foo → ./foo.ts, ./foo/index.ts, etc.
pub fn resolve_import(from_file: &Path, import_source: &str, all_files: &[PathBuf]) -> Option<PathBuf> {
    // Only resolve relative imports
    if !import_source.starts_with('.') {
        return None;
    }

    let from_dir = from_file.parent()?;
    let base = from_dir.join(import_source);

    let extensions = ["", ".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs"];
    let index_names = ["index.ts", "index.tsx", "index.js", "index.jsx"];

    // Try direct match with extensions
    for ext in &extensions {
        let candidate = if ext.is_empty() {
            base.clone()
        } else {
            PathBuf::from(format!("{}{}", base.display(), ext))
        };
        let canonical = candidate.canonicalize().ok()?;
        if all_files.iter().any(|f| f.canonicalize().ok().as_ref() == Some(&canonical)) {
            return Some(canonical);
        }
    }

    // Try as directory with index file
    for index in &index_names {
        let candidate = base.join(index);
        let canonical = candidate.canonicalize().ok()?;
        if all_files.iter().any(|f| f.canonicalize().ok().as_ref() == Some(&canonical)) {
            return Some(canonical);
        }
    }

    None
}

// ── Build import graph ─────────────────────────────────────

pub fn build_import_graph(files: &[(PathBuf, String)], all_file_paths: &[PathBuf]) -> ImportGraph {
    let mut file_infos = HashMap::new();
    let mut resolved_imports = HashMap::new();

    // Phase 1: Analyze each file
    for (path, source) in files {
        let info = analyze_file(path, source);
        file_infos.insert(path.clone(), info);
    }

    // Phase 2: Resolve imports
    for (path, info) in &file_infos {
        for import in &info.imports {
            let resolved = resolve_import(path, &import.source, all_file_paths);
            resolved_imports.insert((path.clone(), import.source.clone()), resolved);
        }
    }

    ImportGraph {
        files: file_infos,
        resolved_imports,
    }
}

// ── Dead code detection ────────────────────────────────────

/// Find exports that are never imported by any other file.
/// Returns (file_path, diagnostic) pairs for proper file association.
pub fn find_unused_exports(graph: &ImportGraph) -> Vec<(PathBuf, LintDiagnostic)> {
    let mut diagnostics = Vec::new();

    // Build a set of all imported names per file
    let mut imported_names: HashMap<PathBuf, HashSet<String>> = HashMap::new();

    for ((from_file, source), resolved_path) in &graph.resolved_imports {
        let Some(target_path) = resolved_path else {
            continue;
        };

        let Some(from_info) = graph.files.get(from_file) else {
            continue;
        };

        for import in &from_info.imports {
            if import.source != *source {
                continue;
            }

            let entry = imported_names.entry(target_path.clone()).or_default();
            for name in &import.names {
                match name {
                    ImportedName::Default => {
                        entry.insert("default".to_string());
                    }
                    ImportedName::Named(n) => {
                        entry.insert(n.clone());
                    }
                    ImportedName::Namespace => {
                        // Namespace import uses everything
                        entry.insert("*".to_string());
                    }
                }
            }
            if import.is_side_effect {
                entry.insert("*".to_string());
            }
        }
    }

    // Check each file's exports
    for (path, info) in &graph.files {
        // Skip entry-point-like files
        if is_likely_entry_point(path) {
            continue;
        }

        let used_names = imported_names.get(path);
        let has_namespace_import = used_names
            .map(|names| names.contains("*"))
            .unwrap_or(false);

        if has_namespace_import {
            continue; // Everything is used via namespace import
        }

        for export in &info.exports {
            let is_used = used_names
                .map(|names| names.contains(&export.name))
                .unwrap_or(false);

            if !is_used {
                diagnostics.push((
                    path.clone(),
                    LintDiagnostic {
                        rule_name: "unused-export".to_string(),
                        message: format!(
                            "Export `{}` is not imported by any other file",
                            export.name
                        ),
                        span: format!("{}:{}", export.line, export.col),
                        severity: Severity::Warning,
                        origin: RuleOrigin::Engine,
                        fix: None,
                        byte_start: export.span_start,
                        byte_end: export.span_end,
                        node_kind: Some("ExportDeclaration".to_string()),
                        symbol: Some(export.name.clone()),
                    },
                ));
            }
        }
    }

    diagnostics
}

/// Find files that are never imported by any other file.
/// Returns (file_path, diagnostic) pairs for proper file association.
pub fn find_unused_files(graph: &ImportGraph) -> Vec<(PathBuf, LintDiagnostic)> {
    let mut diagnostics = Vec::new();

    // Collect all files that are imported by at least one other file
    let mut imported_files: HashSet<PathBuf> = HashSet::new();
    for (_, resolved) in &graph.resolved_imports {
        if let Some(target) = resolved {
            imported_files.insert(target.clone());
        }
    }

    for (path, _) in &graph.files {
        if is_likely_entry_point(path) {
            continue;
        }

        let canonical = path.canonicalize().ok();
        let is_imported = canonical
            .as_ref()
            .map(|c| imported_files.contains(c))
            .unwrap_or(false);

        if !is_imported {
            diagnostics.push((
                path.clone(),
                LintDiagnostic {
                    rule_name: "unused-file".to_string(),
                    message: format!(
                        "File `{}` is not imported by any other file",
                        path.display()
                    ),
                    span: "1:1".to_string(),
                    severity: Severity::Warning,
                    origin: RuleOrigin::Engine,
                    fix: None,
                    byte_start: 0,
                    byte_end: 0,
                    node_kind: None,
                    symbol: None,
                },
            ));
        }
    }

    diagnostics
}

/// Find package.json dependencies that are never imported.
pub fn find_unused_dependencies(
    graph: &ImportGraph,
    package_json_path: &Path,
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    let content = match std::fs::read_to_string(package_json_path) {
        Ok(c) => c,
        Err(_) => return diagnostics,
    };

    let parsed: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return diagnostics,
    };

    let deps = parsed
        .get("dependencies")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    // Collect all bare (non-relative) import sources
    let mut used_packages: HashSet<String> = HashSet::new();
    for info in graph.files.values() {
        for import in &info.imports {
            if !import.source.starts_with('.') {
                // Extract package name: "@scope/pkg/foo" → "@scope/pkg", "pkg/foo" → "pkg"
                let package_name = extract_package_name(&import.source);
                used_packages.insert(package_name);
            }
        }
    }

    for dep in &deps {
        if !used_packages.contains(dep) {
            diagnostics.push(LintDiagnostic {
                rule_name: "unused-dependency".to_string(),
                message: format!("Dependency `{dep}` is listed in package.json but never imported"),
                span: "1:1".to_string(),
                severity: Severity::Warning,
                origin: RuleOrigin::Engine,
                fix: None,
                byte_start: 0,
                byte_end: 0,
                node_kind: None,
                symbol: Some(dep.clone()),
            });
        }
    }

    diagnostics
}

fn extract_package_name(source: &str) -> String {
    if source.starts_with('@') {
        // Scoped: @scope/pkg/subpath → @scope/pkg
        let parts: Vec<&str> = source.splitn(3, '/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            source.to_string()
        }
    } else {
        // Regular: pkg/subpath → pkg
        source.split('/').next().unwrap_or(source).to_string()
    }
}

fn is_likely_entry_point(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let file_stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Common entry point patterns
    matches!(
        file_stem,
        "index" | "main" | "app" | "server" | "cli" | "bin" | "entry" | "mod"
    ) || file_name.contains(".config.")
        || file_name.contains(".test.")
        || file_name.contains(".spec.")
        || file_name.contains(".stories.")
        || file_name.contains(".e2e.")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".d.mts")
        || file_name.ends_with(".d.cts")
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_package_name_from_scoped() {
        assert_eq!(extract_package_name("@scope/pkg/foo"), "@scope/pkg");
        assert_eq!(extract_package_name("@scope/pkg"), "@scope/pkg");
    }

    #[test]
    fn extracts_package_name_from_regular() {
        assert_eq!(extract_package_name("lodash/fp"), "lodash");
        assert_eq!(extract_package_name("react"), "react");
    }

    #[test]
    fn identifies_entry_points() {
        assert!(is_likely_entry_point(Path::new("src/index.ts")));
        assert!(is_likely_entry_point(Path::new("src/main.ts")));
        assert!(is_likely_entry_point(Path::new("foo.test.ts")));
        assert!(is_likely_entry_point(Path::new("foo.config.js")));
        assert!(!is_likely_entry_point(Path::new("src/utils.ts")));
        assert!(!is_likely_entry_point(Path::new("src/helpers/format.ts")));
    }

    #[test]
    fn analyzes_file_imports_and_exports() {
        let source = r#"
import { foo } from './foo';
import type { Bar } from './bar';
import * as utils from './utils';

export const value = 1;
export function helper() {}
export default class MyClass {}
"#;
        let info = analyze_file(Path::new("test.ts"), source);

        assert_eq!(info.imports.len(), 3);
        assert_eq!(info.imports[0].source, "./foo");
        assert_eq!(info.imports[1].is_type_only, true);

        assert_eq!(info.exports.len(), 3);
        assert_eq!(info.exports[0].name, "value");
        assert_eq!(info.exports[1].name, "helper");
        assert_eq!(info.exports[2].name, "default");
    }
}
