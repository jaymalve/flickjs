use crate::project::ProjectInfo;
use crate::rules::dead_code::ImportGraph;
use crate::rules::server::helpers::{
    expression_static_name as server_expression_static_name, is_orm_mutate_call,
};
use crate::rules::{LintContext, LintDiagnostic, LintRule, RuleOrigin, Severity};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Expression, ExportDefaultDeclarationKind, JSXAttribute, JSXAttributeValue,
};
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{SourceType, Span};
use oxc_syntax::node::NodeId;
#[cfg(test)]
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::helpers::{
    callee_static_name, exports_name, file_has_directive, is_component_name, is_custom_hook_name,
    program, span_contains,
};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoImgElement),
        Box::new(PreferNextLink),
        Box::new(NoHeadElement),
        Box::new(NoHeadImport),
        Box::new(NoDocumentImport),
        Box::new(NoScriptInHead),
        Box::new(NoSearchParamsWithoutSuspense),
        Box::new(MissingMetadata),
        Box::new(NoSideEffectInGetHandler),
        Box::new(NoAsyncClientComponent),
    ]
}

macro_rules! next_rule {
    ($name:ident, $rule_name:literal, $run_fn:ident) => {
        pub struct $name;

        impl LintRule for $name {
            fn name(&self) -> &'static str {
                $rule_name
            }

            fn default_severity(&self) -> Severity {
                Severity::Warning
            }

            fn applies_to_project(&self, project: &ProjectInfo) -> bool {
                project.has_next
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                if !ctx.project.has_next {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

next_rule!(NoImgElement, "nextjs/no-img-element", run_no_img_element);
next_rule!(
    PreferNextLink,
    "nextjs/prefer-next-link",
    run_prefer_next_link
);
next_rule!(NoHeadElement, "nextjs/no-head-element", run_no_head_element);
next_rule!(NoHeadImport, "nextjs/no-head-import", run_no_head_import);
next_rule!(
    NoDocumentImport,
    "nextjs/no-document-import",
    run_no_document_import
);
next_rule!(
    NoScriptInHead,
    "nextjs/no-script-in-head",
    run_no_script_in_head
);
next_rule!(
    NoSearchParamsWithoutSuspense,
    "nextjs/no-search-params-without-suspense",
    run_no_search_params_without_suspense
);
next_rule!(
    MissingMetadata,
    "nextjs/missing-metadata",
    run_missing_metadata
);
next_rule!(
    NoSideEffectInGetHandler,
    "nextjs/no-side-effect-in-get-handler",
    run_no_side_effect_in_get_handler
);
next_rule!(
    NoAsyncClientComponent,
    "nextjs/no-async-client-component",
    run_no_async_client_component
);

fn run_no_img_element(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_name_rule(
        ctx,
        rule_name,
        "img",
        "Prefer `next/image` over raw `<img>` elements in Next.js apps",
    )
}

fn run_prefer_next_link(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if opening.name.to_string() != "a" {
                return None;
            }
            let href = opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| attribute.is_identifier("href"))
                .and_then(jsx_attribute_static_string)?;
            if !is_internal_href(&href)
                || opening
                    .attributes
                    .iter()
                    .filter_map(|item| item.as_attribute())
                    .any(|attribute| {
                        attribute.is_identifier("target") || attribute.is_identifier("download")
                    })
            {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Use `next/link` for internal navigation instead of raw anchors",
                opening.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_head_element(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_name_rule(
        ctx,
        rule_name,
        "head",
        "Avoid raw `<head>` elements in Next.js components",
    )
}

fn run_no_head_import(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    import_source_rule(
        ctx,
        rule_name,
        "next/head",
        "Avoid `next/head` in modern Next.js app-router code",
    )
}

fn run_no_document_import(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ImportDeclaration(import_decl) = node.kind() else {
                return None;
            };
            if import_decl.source.value.as_str() != "next/document" || is_document_file(ctx) {
                return None;
            }
            Some(ctx.diagnostic(
                rule_name,
                "Only custom `_document` files should import `next/document`",
                import_decl.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_script_in_head(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if opening.name.to_string() != "script" {
                return None;
            }
            ctx.semantic
                .nodes()
                .ancestor_kinds(node.id())
                .any(|kind| {
                    matches!(
                        kind,
                        AstKind::JSXElement(element) if element.opening_element.name.to_string() == "head"
                    )
                })
                .then(|| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid placing `<script>` tags directly inside `<head>` in Next.js",
                        opening.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_search_params_without_suspense(
    ctx: &LintContext,
    _rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    collect_search_params_diagnostics_for_file(ctx.file_path, ctx.source, Severity::Warning)
}

fn run_missing_metadata(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    if !is_app_router_metadata_file(ctx)
        || exports_name(ctx, "metadata")
        || exports_name(ctx, "generateMetadata")
    {
        return Vec::new();
    }
    let Some(program) = program(ctx) else {
        return Vec::new();
    };
    vec![ctx.diagnostic(
        rule_name,
        "App-router pages and layouts should export `metadata` or `generateMetadata`",
        program.span,
        Severity::Warning,
    )]
}

fn run_no_side_effect_in_get_handler(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    if !is_route_file(ctx) {
        return Vec::new();
    }

    exported_function_spans(ctx, "GET")
        .into_iter()
        .filter_map(|(name_span, body_span)| {
            function_contains_get_side_effects(ctx, body_span).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "GET route handlers should avoid mutations and other server side effects",
                    name_span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_async_client_component(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    if !file_has_directive(ctx, "use client") {
        return Vec::new();
    }

    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::Function(function)
                if function.r#async
                    && function.id.as_ref().is_some_and(|identifier| {
                        is_component_name(identifier.name.as_str())
                    }) =>
            {
                Some(ctx.diagnostic(
                    rule_name,
                    "Client components should not be async functions",
                    function.span,
                    Severity::Warning,
                ))
            }
            AstKind::ArrowFunctionExpression(function) if function.r#async => {
                let AstKind::VariableDeclarator(decl) = ctx.semantic.nodes().parent_kind(node.id())
                else {
                    return None;
                };
                let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                    return None;
                };
                is_component_name(identifier.name.as_str()).then(|| {
                    ctx.diagnostic(
                        rule_name,
                        "Client components should not be async functions",
                        function.span,
                        Severity::Warning,
                    )
                })
            }
            _ => None,
        })
        .collect()
}

fn jsx_name_rule(
    ctx: &LintContext,
    rule_name: &'static str,
    name: &str,
    message: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            (opening.name.to_string() == name)
                .then(|| ctx.diagnostic(rule_name, message, opening.span, Severity::Warning))
        })
        .collect()
}

fn import_source_rule(
    ctx: &LintContext,
    rule_name: &'static str,
    source: &str,
    message: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ImportDeclaration(import_decl) = node.kind() else {
                return None;
            };
            (import_decl.source.value.as_str() == source)
                .then(|| ctx.diagnostic(rule_name, message, import_decl.span, Severity::Warning))
        })
        .collect()
}

fn jsx_attribute_static_string(attribute: &JSXAttribute<'_>) -> Option<String> {
    match attribute.value.as_ref()? {
        JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        JSXAttributeValue::ExpressionContainer(container) => match &container.expression {
            oxc_ast::ast::JSXExpression::StringLiteral(literal) => Some(literal.value.to_string()),
            oxc_ast::ast::JSXExpression::TemplateLiteral(template) => {
                template.single_quasi().map(|value| value.to_string())
            }
            _ => None,
        },
        _ => None,
    }
}

fn is_internal_href(href: &str) -> bool {
    href.starts_with('/') && !href.starts_with("//") && !matches!(href, "/" | "/_next" | "/_next/")
}

fn is_document_file(ctx: &LintContext) -> bool {
    ctx.file_path
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("_document."))
}

fn is_route_file(ctx: &LintContext) -> bool {
    ctx.file_path
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("route."))
}

fn is_app_router_metadata_file(ctx: &LintContext) -> bool {
    ctx.file_path
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            matches!(
                name,
                "page.js"
                    | "page.jsx"
                    | "page.ts"
                    | "page.tsx"
                    | "layout.js"
                    | "layout.jsx"
                    | "layout.ts"
                    | "layout.tsx"
            )
        })
}

const SEARCH_PARAMS_RULE_NAME: &str = "nextjs/no-search-params-without-suspense";
const SEARCH_PARAMS_MESSAGE: &str = "Wrap `useSearchParams()` consumers in `<Suspense>`";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchFileAnalysis {
    symbols: HashMap<String, SearchSymbolData>,
    exports: HashMap<String, String>,
    imports: HashMap<String, ImportedBinding>,
    top_level_search_calls: Vec<SpanInfo>,
}

impl SearchFileAnalysis {
    /// Returns true if this file has any search-params-relevant usage that
    /// would produce diagnostics or participate in cross-file resolution.
    pub(crate) fn has_search_params_usage(&self) -> bool {
        !self.top_level_search_calls.is_empty()
            || self
                .symbols
                .values()
                .any(|s| !s.direct_search_calls.is_empty())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchSymbolData {
    kind: SearchSymbolKind,
    direct_search_calls: Vec<SpanInfo>,
    call_edges: Vec<PendingCallEdge>,
    render_edges: Vec<PendingRenderEdge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum SearchSymbolKind {
    Component,
    Hook,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SearchSymbolKey {
    file: PathBuf,
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ImportedBinding {
    source: String,
    imported_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PendingCallEdge {
    target: SymbolRef,
    span: SpanInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PendingRenderEdge {
    target: SymbolRef,
    span: SpanInfo,
    wrapped: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum SymbolRef {
    Local(String),
    Import(ImportedBinding),
}

#[derive(Clone, Debug)]
struct ResolvedSearchSymbol {
    kind: SearchSymbolKind,
    direct_search_calls: Vec<SpanInfo>,
    call_edges: Vec<ResolvedCallEdge>,
    render_edges: Vec<ResolvedRenderEdge>,
}

#[derive(Clone, Debug)]
struct ResolvedCallEdge {
    target: SearchSymbolKey,
    span: SpanInfo,
}

#[derive(Clone, Debug)]
struct ResolvedRenderEdge {
    target: SearchSymbolKey,
    span: SpanInfo,
    wrapped: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SpanInfo {
    byte_start: u32,
    byte_end: u32,
    line: usize,
    col: usize,
}

#[cfg(test)]
pub(crate) fn collect_search_params_project_diagnostics(
    file_sources: &[(PathBuf, String)],
    graph: &ImportGraph,
    severity: Severity,
) -> Vec<(PathBuf, LintDiagnostic)> {
    let analyses = analyze_search_params_files(file_sources);
    collect_search_params_diagnostics(&analyses, Some(graph), severity)
}

pub(crate) fn collect_search_params_diagnostics_from_analyses(
    analyses: &HashMap<PathBuf, SearchFileAnalysis>,
    graph: &ImportGraph,
    severity: Severity,
) -> Vec<(PathBuf, LintDiagnostic)> {
    collect_search_params_diagnostics(analyses, Some(graph), severity)
}

fn collect_search_params_diagnostics_for_file(
    path: &Path,
    source: &str,
    severity: Severity,
) -> Vec<LintDiagnostic> {
    let analyses = HashMap::from([(path.to_path_buf(), analyze_search_params_file(path, source))]);
    collect_search_params_diagnostics(&analyses, None, severity)
        .into_iter()
        .filter_map(|(file_path, diagnostic)| (file_path == path).then_some(diagnostic))
        .collect()
}

#[cfg(test)]
fn analyze_search_params_files(
    file_sources: &[(PathBuf, String)],
) -> HashMap<PathBuf, SearchFileAnalysis> {
    file_sources
        .par_iter()
        .map(|(path, source)| (path.clone(), analyze_search_params_file(path, source)))
        .collect()
}

fn analyze_search_params_file(path: &Path, source: &str) -> SearchFileAnalysis {
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);

    collect_search_analysis_from_semantic(source, &semantic.semantic)
}

/// Collect search-params analysis reusing an existing Semantic from the lint pass.
pub(crate) fn collect_search_analysis_from_semantic(
    source: &str,
    semantic: &Semantic<'_>,
) -> SearchFileAnalysis {
    let mut analysis = SearchFileAnalysis {
        symbols: HashMap::new(),
        exports: HashMap::new(),
        imports: HashMap::new(),
        top_level_search_calls: Vec::new(),
    };

    collect_local_search_symbols(semantic, &mut analysis);
    collect_imported_search_symbols(semantic, &mut analysis);
    collect_exported_search_symbols(semantic, &mut analysis);
    collect_search_symbol_usage(source, semantic, &mut analysis);

    analysis
}

fn collect_local_search_symbols(semantic: &Semantic<'_>, analysis: &mut SearchFileAnalysis) {
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::Function(function) if function.body.is_some() => {
                let Some(identifier) = &function.id else {
                    continue;
                };
                let Some(kind) = search_symbol_kind(identifier.name.as_str()) else {
                    continue;
                };
                analysis
                    .symbols
                    .entry(identifier.name.to_string())
                    .or_insert_with(|| SearchSymbolData {
                        kind,
                        direct_search_calls: Vec::new(),
                        call_edges: Vec::new(),
                        render_edges: Vec::new(),
                    });
            }
            AstKind::VariableDeclarator(decl) => {
                let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                    continue;
                };
                let Some(init) = &decl.init else {
                    continue;
                };
                if !matches!(
                    init.without_parentheses(),
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                    continue;
                }
                let Some(kind) = search_symbol_kind(identifier.name.as_str()) else {
                    continue;
                };
                analysis
                    .symbols
                    .entry(identifier.name.to_string())
                    .or_insert_with(|| SearchSymbolData {
                        kind,
                        direct_search_calls: Vec::new(),
                        call_edges: Vec::new(),
                        render_edges: Vec::new(),
                    });
            }
            _ => {}
        }
    }
}

fn collect_imported_search_symbols(semantic: &Semantic<'_>, analysis: &mut SearchFileAnalysis) {
    for node in semantic.nodes().iter() {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            continue;
        };
        let Some(specifiers) = &import_decl.specifiers else {
            continue;
        };
        for specifier in specifiers {
            match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let local_name = specifier.local.name.to_string();
                    if search_symbol_kind(&local_name).is_none() {
                        continue;
                    }
                    analysis.imports.insert(
                        local_name,
                        ImportedBinding {
                            source: import_decl.source.value.to_string(),
                            imported_name: specifier.imported.name().to_string(),
                        },
                    );
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let local_name = specifier.local.name.to_string();
                    if search_symbol_kind(&local_name).is_none() {
                        continue;
                    }
                    analysis.imports.insert(
                        local_name,
                        ImportedBinding {
                            source: import_decl.source.value.to_string(),
                            imported_name: "default".to_string(),
                        },
                    );
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }
}

fn collect_exported_search_symbols(semantic: &Semantic<'_>, analysis: &mut SearchFileAnalysis) {
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::ExportNamedDeclaration(decl) => {
                if decl.source.is_some() {
                    continue;
                }
                let Some(declaration) = &decl.declaration else {
                    continue;
                };
                match declaration {
                    oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                        let Some(identifier) = &function.id else {
                            continue;
                        };
                        if analysis.symbols.contains_key(identifier.name.as_str()) {
                            analysis.exports.insert(
                                identifier.name.to_string(),
                                identifier.name.to_string(),
                            );
                        }
                    }
                    oxc_ast::ast::Declaration::VariableDeclaration(decl) => {
                        for declarator in &decl.declarations {
                            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                                continue;
                            };
                            if analysis.symbols.contains_key(identifier.name.as_str()) {
                                analysis.exports.insert(
                                    identifier.name.to_string(),
                                    identifier.name.to_string(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            AstKind::ExportDefaultDeclaration(decl) => match &decl.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    let Some(identifier) = &function.id else {
                        continue;
                    };
                    if analysis.symbols.contains_key(identifier.name.as_str()) {
                        analysis
                            .exports
                            .insert("default".to_string(), identifier.name.to_string());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn collect_search_symbol_usage(
    source: &str,
    semantic: &Semantic<'_>,
    analysis: &mut SearchFileAnalysis,
) {
    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::CallExpression(call) => {
                let span = span_info(source, call.span);
                let Some(symbol_name) = enclosing_search_symbol_name(semantic, node.id(), analysis)
                else {
                    if callee_static_name(&call.callee).as_deref() == Some("useSearchParams") {
                        analysis.top_level_search_calls.push(span);
                    }
                    continue;
                };
                let is_search_params_call =
                    callee_static_name(&call.callee).as_deref() == Some("useSearchParams");
                let target = (!is_search_params_call)
                    .then(|| symbol_ref_from_call(&call.callee, analysis))
                    .flatten();
                let Some(symbol) = analysis.symbols.get_mut(&symbol_name) else {
                    continue;
                };
                if is_search_params_call {
                    symbol.direct_search_calls.push(span);
                    continue;
                }
                let Some(target) = target else {
                    continue;
                };
                symbol.call_edges.push(PendingCallEdge { target, span });
            }
            AstKind::JSXOpeningElement(opening) => {
                let Some(component_name) =
                    enclosing_component_name(semantic, node.id(), analysis)
                else {
                    continue;
                };
                let Some(target) = symbol_ref_from_jsx_name(&opening.name.to_string(), analysis)
                else {
                    continue;
                };
                let wrapped = semantic.nodes().ancestor_kinds(node.id()).any(|kind| {
                    matches!(
                        kind,
                        AstKind::JSXElement(element)
                            if element.opening_element.name.to_string() == "Suspense"
                    )
                });
                if let Some(symbol) = analysis.symbols.get_mut(&component_name) {
                    symbol.render_edges.push(PendingRenderEdge {
                        target,
                        span: span_info(source, opening.span),
                        wrapped,
                    });
                }
            }
            _ => {}
        }
    }
}

fn collect_search_params_diagnostics(
    analyses: &HashMap<PathBuf, SearchFileAnalysis>,
    graph: Option<&ImportGraph>,
    severity: Severity,
) -> Vec<(PathBuf, LintDiagnostic)> {
    let resolved_symbols = resolve_search_symbols(analyses, graph);
    let mut incoming_renders: HashMap<SearchSymbolKey, usize> = HashMap::new();
    for symbol in resolved_symbols.values() {
        for edge in &symbol.render_edges {
            *incoming_renders.entry(edge.target.clone()).or_default() += 1;
        }
    }

    let mut memo = HashMap::new();
    let mut diagnostics = Vec::new();

    for (file_path, analysis) in analyses {
        for span in &analysis.top_level_search_calls {
            diagnostics.push((
                file_path.clone(),
                search_params_diagnostic(span, severity.clone()),
            ));
        }
    }

    for (key, symbol) in &resolved_symbols {
        for edge in &symbol.render_edges {
            if !edge.wrapped
                && symbol_requires_suspense(&edge.target, &resolved_symbols, &mut memo, &mut HashSet::new())
            {
                diagnostics.push((
                    key.file.clone(),
                    search_params_diagnostic(&edge.span, severity.clone()),
                ));
            }
        }
    }

    for (key, symbol) in &resolved_symbols {
        if symbol.kind != SearchSymbolKind::Component {
            continue;
        }
        if incoming_renders.get(key).copied().unwrap_or(0) > 0 {
            continue;
        }
        if !symbol_requires_suspense(key, &resolved_symbols, &mut memo, &mut HashSet::new()) {
            continue;
        }
        for span in &symbol.direct_search_calls {
            diagnostics.push((
                key.file.clone(),
                search_params_diagnostic(span, severity.clone()),
            ));
        }
        for edge in &symbol.call_edges {
            if symbol_requires_suspense(&edge.target, &resolved_symbols, &mut memo, &mut HashSet::new()) {
                diagnostics.push((
                    key.file.clone(),
                    search_params_diagnostic(&edge.span, severity.clone()),
                ));
            }
        }
    }

    diagnostics
}

fn resolve_search_symbols(
    analyses: &HashMap<PathBuf, SearchFileAnalysis>,
    graph: Option<&ImportGraph>,
) -> HashMap<SearchSymbolKey, ResolvedSearchSymbol> {
    let canonical_to_original = graph
        .map(|graph| {
            graph
                .canonical_paths
                .iter()
                .map(|(original, canonical)| (canonical.clone(), original.clone()))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut resolved = HashMap::new();

    for (file_path, analysis) in analyses {
        for (name, symbol) in &analysis.symbols {
            let key = SearchSymbolKey {
                file: file_path.clone(),
                name: name.clone(),
            };
            let call_edges = symbol
                .call_edges
                .iter()
                .filter_map(|edge| {
                    resolve_symbol_ref(
                        file_path,
                        &edge.target,
                        analyses,
                        graph,
                        &canonical_to_original,
                    )
                    .map(|target| ResolvedCallEdge {
                        target,
                        span: edge.span.clone(),
                    })
                })
                .collect();
            let render_edges = symbol
                .render_edges
                .iter()
                .filter_map(|edge| {
                    resolve_symbol_ref(
                        file_path,
                        &edge.target,
                        analyses,
                        graph,
                        &canonical_to_original,
                    )
                    .map(|target| ResolvedRenderEdge {
                        target,
                        span: edge.span.clone(),
                        wrapped: edge.wrapped,
                    })
                })
                .collect();
            resolved.insert(
                key,
                ResolvedSearchSymbol {
                    kind: symbol.kind,
                    direct_search_calls: symbol.direct_search_calls.clone(),
                    call_edges,
                    render_edges,
                },
            );
        }
    }

    resolved
}

fn resolve_symbol_ref(
    from_file: &Path,
    symbol_ref: &SymbolRef,
    analyses: &HashMap<PathBuf, SearchFileAnalysis>,
    graph: Option<&ImportGraph>,
    canonical_to_original: &HashMap<PathBuf, PathBuf>,
) -> Option<SearchSymbolKey> {
    match symbol_ref {
        SymbolRef::Local(name) => analyses.get(from_file)?.symbols.get(name).map(|_| SearchSymbolKey {
            file: from_file.to_path_buf(),
            name: name.clone(),
        }),
        SymbolRef::Import(binding) => {
            let graph = graph?;
            let resolved_path = graph
                .resolved_imports
                .get(&(from_file.to_path_buf(), binding.source.clone()))?
                .clone()?;
            let target_file = canonical_to_original
                .get(&resolved_path)
                .cloned()
                .unwrap_or(resolved_path);
            let target_analysis = analyses.get(&target_file)?;
            let local_name = target_analysis.exports.get(&binding.imported_name)?;
            target_analysis.symbols.get(local_name).map(|_| SearchSymbolKey {
                file: target_file,
                name: local_name.clone(),
            })
        }
    }
}

fn symbol_requires_suspense(
    key: &SearchSymbolKey,
    symbols: &HashMap<SearchSymbolKey, ResolvedSearchSymbol>,
    memo: &mut HashMap<SearchSymbolKey, bool>,
    visiting: &mut HashSet<SearchSymbolKey>,
) -> bool {
    if let Some(requires) = memo.get(key) {
        return *requires;
    }
    let Some(symbol) = symbols.get(key) else {
        return false;
    };
    if !visiting.insert(key.clone()) {
        return false;
    }

    let requires = !symbol.direct_search_calls.is_empty()
        || symbol
            .call_edges
            .iter()
            .any(|edge| symbol_requires_suspense(&edge.target, symbols, memo, visiting));

    visiting.remove(key);
    memo.insert(key.clone(), requires);
    requires
}

fn enclosing_search_symbol_name(
    semantic: &Semantic<'_>,
    node_id: NodeId,
    analysis: &SearchFileAnalysis,
) -> Option<String> {
    for ancestor_id in semantic.nodes().ancestor_ids(node_id) {
        if let Some(name) = symbol_name_for_ancestor(semantic, ancestor_id, analysis) {
            return Some(name);
        }
    }
    None
}

fn enclosing_component_name(
    semantic: &Semantic<'_>,
    node_id: NodeId,
    analysis: &SearchFileAnalysis,
) -> Option<String> {
    let name = enclosing_search_symbol_name(semantic, node_id, analysis)?;
    matches!(
        analysis.symbols.get(&name).map(|symbol| symbol.kind),
        Some(SearchSymbolKind::Component)
    )
    .then_some(name)
}

fn symbol_name_for_ancestor(
    semantic: &Semantic<'_>,
    ancestor_id: NodeId,
    analysis: &SearchFileAnalysis,
) -> Option<String> {
    match semantic.nodes().kind(ancestor_id) {
        AstKind::Function(function) => {
            if let Some(identifier) = &function.id {
                let name = identifier.name.to_string();
                if analysis.symbols.contains_key(&name) {
                    return Some(name);
                }
            }
            let AstKind::VariableDeclarator(decl) = semantic.nodes().parent_kind(ancestor_id) else {
                return None;
            };
            let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                return None;
            };
            let name = identifier.name.to_string();
            analysis.symbols.contains_key(&name).then_some(name)
        }
        AstKind::ArrowFunctionExpression(_) => {
            let AstKind::VariableDeclarator(decl) = semantic.nodes().parent_kind(ancestor_id) else {
                return None;
            };
            let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                return None;
            };
            let name = identifier.name.to_string();
            analysis.symbols.contains_key(&name).then_some(name)
        }
        _ => None,
    }
}

fn symbol_ref_from_call(
    callee: &Expression<'_>,
    analysis: &SearchFileAnalysis,
) -> Option<SymbolRef> {
    let Expression::Identifier(identifier) = callee.without_parentheses() else {
        return None;
    };
    symbol_ref_from_name(identifier.name.as_str(), analysis)
}

fn symbol_ref_from_jsx_name(name: &str, analysis: &SearchFileAnalysis) -> Option<SymbolRef> {
    symbol_ref_from_name(name, analysis)
}

fn symbol_ref_from_name(name: &str, analysis: &SearchFileAnalysis) -> Option<SymbolRef> {
    if analysis.symbols.contains_key(name) {
        return Some(SymbolRef::Local(name.to_string()));
    }
    analysis
        .imports
        .get(name)
        .cloned()
        .map(SymbolRef::Import)
}

fn search_symbol_kind(name: &str) -> Option<SearchSymbolKind> {
    if is_custom_hook_name(name) {
        Some(SearchSymbolKind::Hook)
    } else if is_component_name(name) {
        Some(SearchSymbolKind::Component)
    } else {
        None
    }
}

fn span_info(source: &str, span: Span) -> SpanInfo {
    let (line, col) = offset_to_line_col(source, span.start as usize);
    SpanInfo {
        byte_start: span.start,
        byte_end: span.end,
        line,
        col,
    }
}

fn search_params_diagnostic(span: &SpanInfo, severity: Severity) -> LintDiagnostic {
    LintDiagnostic {
        rule_name: SEARCH_PARAMS_RULE_NAME.to_string(),
        message: SEARCH_PARAMS_MESSAGE.to_string(),
        span: format!("{}:{}", span.line, span.col),
        severity,
        origin: RuleOrigin::BuiltIn,
        fix: None,
        byte_start: span.byte_start,
        byte_end: span.byte_end,
        node_kind: None,
        symbol: None,
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (index, ch) in source.char_indices() {
        if index >= offset {
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

fn exported_function_spans(ctx: &LintContext, export_name: &str) -> Vec<(Span, Span)> {
    let mut spans = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::ExportNamedDeclaration(decl) = node.kind() else {
            continue;
        };
        let Some(declaration) = &decl.declaration else {
            continue;
        };
        match declaration {
            oxc_ast::ast::Declaration::FunctionDeclaration(function) => {
                let Some(identifier) = &function.id else {
                    continue;
                };
                let Some(body) = &function.body else {
                    continue;
                };
                if identifier.name == export_name {
                    spans.push((identifier.span, body.span));
                }
            }
            oxc_ast::ast::Declaration::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                        continue;
                    };
                    if identifier.name != export_name {
                        continue;
                    }
                    let Some(init) = &declarator.init else {
                        continue;
                    };
                    match init.without_parentheses() {
                        Expression::ArrowFunctionExpression(function) => {
                            spans.push((identifier.span, function.body.span));
                        }
                        Expression::FunctionExpression(function) => {
                            let Some(body) = &function.body else {
                                continue;
                            };
                            spans.push((identifier.span, body.span));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    spans
}

fn function_contains_get_side_effects(ctx: &LintContext, body_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        if !span_contains(body_span, call.span) {
            return false;
        }
        is_orm_mutate_call(call)
            || server_expression_static_name(&call.callee).is_some_and(|name| {
                matches!(
                    name.as_str(),
                    "revalidatePath" | "revalidateTag" | "redirect" | "notFound"
                )
            })
    })
}

#[cfg(test)]
mod tests {
    use super::collect_search_params_project_diagnostics;
    use crate::project::ProjectInfo;
    use crate::rules::dead_code::build_import_graph;
    use crate::rules::lint_source_for_test_with_project;
    use crate::rules::Severity;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn rule_messages(path: &str, rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project(path, source, &ProjectInfo::test_all())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    fn project_rule_messages(files: &[(&str, &str)]) -> Vec<(PathBuf, String)> {
        let dir = tempdir().unwrap();
        let mut paths = Vec::new();
        for (relative_path, source) in files {
            let path = dir.path().join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, source).unwrap();
            paths.push(path);
        }

        let file_sources = paths
            .iter()
            .map(|path| (path.clone(), fs::read_to_string(path).unwrap()))
            .collect::<Vec<_>>();
        let graph = build_import_graph(&file_sources, &paths, None);

        collect_search_params_project_diagnostics(&file_sources, &graph, Severity::Warning)
            .into_iter()
            .map(|(path, diagnostic)| (path, diagnostic.message))
            .collect()
    }

    #[test]
    fn flags_img_element() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-img-element",
            "export default function Page() { return <img src=\"/logo.png\" />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Prefer `next/image` over raw `<img>` elements in Next.js apps"]
        );
    }

    #[test]
    fn flags_internal_anchor() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/prefer-next-link",
            "export default function Page() { return <a href=\"/dashboard\">Dashboard</a>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Use `next/link` for internal navigation instead of raw anchors"]
        );
    }

    #[test]
    fn flags_head_element() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-head-element",
            "export default function Page() { return <head><title>Demo</title></head>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid raw `<head>` elements in Next.js components"]
        );
    }

    #[test]
    fn flags_head_import() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-head-import",
            "import Head from 'next/head';\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid `next/head` in modern Next.js app-router code"]
        );
    }

    #[test]
    fn flags_document_import() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-document-import",
            "import { Html } from 'next/document';\n",
        );
        assert_eq!(
            messages,
            vec!["Only custom `_document` files should import `next/document`"]
        );
    }

    #[test]
    fn flags_script_in_head() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-script-in-head",
            "export default function Page() { return <head><script src=\"/a.js\" /></head>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid placing `<script>` tags directly inside `<head>` in Next.js"]
        );
    }

    #[test]
    fn flags_search_params_without_suspense() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-search-params-without-suspense",
            "import { useSearchParams } from 'next/navigation';\nexport default function Page() { const params = useSearchParams(); return <div>{params.get('q')}</div>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap `useSearchParams()` consumers in `<Suspense>`"]
        );
    }

    #[test]
    fn unrelated_suspense_does_not_suppress_warning() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-search-params-without-suspense",
            "import { Suspense } from 'react';\nimport { useSearchParams } from 'next/navigation';\nfunction SearchResults() { const params = useSearchParams(); return <div>{params.get('q')}</div>; }\nfunction Other() { return <div />; }\nexport default function Page() { return <><Suspense><Other /></Suspense><SearchResults /></>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap `useSearchParams()` consumers in `<Suspense>`"]
        );
    }

    #[test]
    fn flags_exported_custom_hook_without_suspense_across_files() {
        let messages = project_rule_messages(&[
            (
                "app/page.tsx",
                "import { useSearchQuery as useSearchQueryAlias } from '../hooks/use-search-query';\nexport default function Page() { const query = useSearchQueryAlias(); return <div>{query}</div>; }\n",
            ),
            (
                "hooks/use-search-query.ts",
                "import { useSearchParams } from 'next/navigation';\nexport function useSearchQuery() { return useSearchParams().get('q'); }\n",
            ),
        ]);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].0.ends_with("app/page.tsx"));
        assert_eq!(
            messages[0].1,
            "Wrap `useSearchParams()` consumers in `<Suspense>`"
        );
    }

    #[test]
    fn allows_exported_custom_hook_when_consumer_is_wrapped_across_files() {
        let messages = project_rule_messages(&[
            (
                "app/page.tsx",
                "import { Suspense } from 'react';\nimport { ClientSearch } from '../components/client-search';\nexport default function Page() { return <Suspense><ClientSearch /></Suspense>; }\n",
            ),
            (
                "components/client-search.tsx",
                "import { useSearchQuery } from '../hooks/use-search-query';\nexport function ClientSearch() { const query = useSearchQuery(); return <div>{query}</div>; }\n",
            ),
            (
                "hooks/use-search-query.ts",
                "import { useSearchParams } from 'next/navigation';\nexport function useSearchQuery() { return useSearchParams().get('q'); }\n",
            ),
        ]);
        assert!(messages.is_empty());
    }

    #[test]
    fn flags_missing_metadata() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/missing-metadata",
            "export default function Page() { return <div />; }\n",
        );
        assert_eq!(
            messages,
            vec!["App-router pages and layouts should export `metadata` or `generateMetadata`"]
        );
    }

    #[test]
    fn flags_get_handler_side_effects() {
        let messages = rule_messages(
            "app/api/users/route.ts",
            "nextjs/no-side-effect-in-get-handler",
            "export async function GET() { await prisma.user.create({ data: {} }); return Response.json({ ok: true }); }\n",
        );
        assert_eq!(
            messages,
            vec!["GET route handlers should avoid mutations and other server side effects"]
        );
    }

    #[test]
    fn flags_async_client_component() {
        let messages = rule_messages(
            "app/page.tsx",
            "nextjs/no-async-client-component",
            "\"use client\";\nexport default async function Page() { return <div />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Client components should not be async functions"]
        );
    }
}
