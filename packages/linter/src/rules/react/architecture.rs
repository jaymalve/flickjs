use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast::AstKind;
use oxc_span::Span;
use oxc_syntax::node::NodeId;

use super::helpers::{file_uses_react, is_component_name};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoGiantComponent),
        Box::new(NoRenderInRender),
        Box::new(NoNestedComponent),
    ]
}

macro_rules! react_rule {
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
                project.has_react
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                if !ctx.project.has_react || !file_uses_react(ctx) {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

react_rule!(
    NoGiantComponent,
    "react/no-giant-component",
    run_no_giant_component
);
react_rule!(
    NoRenderInRender,
    "react/no-render-in-render",
    run_no_render_in_render
);
react_rule!(
    NoNestedComponent,
    "react/no-nested-component",
    run_no_nested_component
);

fn run_no_giant_component(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| component_definition(ctx, node.id(), node.kind()))
        .filter_map(|definition| {
            let start_line = ctx.offset_to_line_col(definition.span.start as usize).0;
            let end_line = ctx.offset_to_line_col(definition.span.end as usize).0;
            let lines = end_line.saturating_sub(start_line) + 1;
            (lines > 300).then(|| {
                ctx.diagnostic(
                    rule_name,
                    format!(
                        "Component `{}` spans {lines} lines; consider splitting it up",
                        definition.name
                    ),
                    definition.diagnostic_span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_render_in_render(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let Expression::Identifier(identifier) = call.callee.without_parentheses() else {
                return None;
            };
            let name = identifier.name.as_str();
            if !name.starts_with("render")
                || !name
                    .chars()
                    .nth(6)
                    .is_some_and(|ch| ch.is_ascii_uppercase())
            {
                return None;
            }
            let inside_jsx = ctx
                .semantic
                .nodes()
                .ancestor_kinds(node.id())
                .any(|kind| matches!(kind, AstKind::JSXExpressionContainer(_)));
            inside_jsx.then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid calling `render*()` helpers directly from JSX",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_nested_component(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| component_definition(ctx, node.id(), node.kind()))
        .filter_map(|definition| {
            enclosing_component_name(ctx, definition.node_id).map(|outer| {
                ctx.diagnostic(
                    rule_name,
                    format!(
                        "Component `{}` is defined inside component `{}`",
                        definition.name, outer
                    ),
                    definition.diagnostic_span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

struct ComponentDefinition {
    node_id: NodeId,
    name: String,
    span: Span,
    diagnostic_span: Span,
}

fn component_definition(
    ctx: &LintContext,
    node_id: NodeId,
    kind: AstKind<'_>,
) -> Option<ComponentDefinition> {
    match kind {
        AstKind::Function(function) if function.body.is_some() => {
            let name = function.id.as_ref()?.name.to_string();
            is_component_name(&name).then_some(ComponentDefinition {
                node_id,
                name,
                span: function.span,
                diagnostic_span: function
                    .id
                    .as_ref()
                    .map(|id| id.span)
                    .unwrap_or(function.span),
            })
        }
        AstKind::ArrowFunctionExpression(function) => {
            let AstKind::VariableDeclarator(decl) = ctx.semantic.nodes().parent_kind(node_id)
            else {
                return None;
            };
            let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                return None;
            };
            let name = identifier.name.to_string();
            is_component_name(&name).then_some(ComponentDefinition {
                node_id,
                name,
                span: function.span,
                diagnostic_span: identifier.span,
            })
        }
        _ => None,
    }
}

fn enclosing_component_name(ctx: &LintContext, node_id: NodeId) -> Option<String> {
    for ancestor_id in ctx.semantic.nodes().ancestor_ids(node_id) {
        if ancestor_id == node_id {
            continue;
        }
        if let Some(component) =
            component_definition(ctx, ancestor_id, ctx.semantic.nodes().kind(ancestor_id))
        {
            return Some(component.name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("test.tsx", source, &ProjectInfo::test_react())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_giant_component() {
        let mut source = String::from("export function Giant() {\n");
        for _ in 0..305 {
            source.push_str("  const line = 1;\n");
        }
        source.push_str("  return <div />;\n}\n");

        let messages = rule_messages("react/no-giant-component", &source);
        assert_eq!(
            messages,
            vec!["Component `Giant` spans 308 lines; consider splitting it up"]
        );
    }

    #[test]
    fn flags_render_in_render() {
        let messages = rule_messages(
            "react/no-render-in-render",
            "export function Page() {\n  return <section>{renderCard()}</section>;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid calling `render*()` helpers directly from JSX"]
        );
    }

    #[test]
    fn flags_nested_component() {
        let messages = rule_messages(
            "react/no-nested-component",
            "export function Outer() {\n  function Inner() {\n    return <div />;\n  }\n  return <Inner />;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Component `Inner` is defined inside component `Outer`"]
        );
    }
}
