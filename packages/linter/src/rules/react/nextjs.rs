use crate::project::ProjectInfo;
use crate::rules::server::helpers::{
    expression_static_name as server_expression_static_name, is_orm_mutate_call,
};
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{BindingPattern, Expression, JSXAttribute, JSXAttributeValue};
use oxc_ast::AstKind;
use oxc_span::Span;
use std::ffi::OsStr;

use super::helpers::{
    callee_static_name, exports_name, file_has_directive, has_jsx_element, program, span_contains,
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
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    let has_suspense = has_jsx_element(ctx, "Suspense");
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            (callee_static_name(&call.callee).as_deref() == Some("useSearchParams")
                && !has_suspense)
                .then(|| {
                    ctx.diagnostic(
                        rule_name,
                        "Wrap `useSearchParams()` consumers in `<Suspense>`",
                        call.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
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
                        is_component_like_name(identifier.name.as_str())
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
                is_component_like_name(identifier.name.as_str()).then(|| {
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

fn is_component_like_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
        || matches!(name, "Page" | "Layout" | "Template" | "Default")
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(path: &str, rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project(path, source, &ProjectInfo::test_all())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
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
