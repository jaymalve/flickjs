use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{Expression, JSXAttributeValue, JSXExpression, LogicalExpression};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::operator::LogicalOperator;

use super::helpers::file_uses_react;

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoArrayIndexKey),
        Box::new(NoPreventDefault),
        Box::new(NoConditionalRenderZero),
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
    NoArrayIndexKey,
    "react/no-array-index-key",
    run_no_array_index_key
);
react_rule!(
    NoPreventDefault,
    "react/no-prevent-default",
    run_no_prevent_default
);
react_rule!(
    NoConditionalRenderZero,
    "react/no-conditional-render-zero",
    run_no_conditional_render_zero
);

fn run_no_array_index_key(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXAttribute(attribute) = node.kind() else {
                return None;
            };
            if !attribute.is_key() {
                return None;
            }
            let JSXAttributeValue::ExpressionContainer(container) = attribute.value.as_ref()?
            else {
                return None;
            };
            let identifier = match &container.expression {
                JSXExpression::Identifier(identifier) => identifier,
                _ => return None,
            };
            let name = identifier.name.as_str();
            if !matches!(name, "index" | "idx" | "i") && !name.ends_with("Index") {
                return None;
            }
            Some(ctx.diagnostic(
                rule_name,
                "Avoid using array indexes as React keys",
                attribute.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_prevent_default(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            let tag_name = opening.name.to_string();
            let expected_attr = match tag_name.as_str() {
                "form" => "onSubmit",
                "a" => "onClick",
                _ => return None,
            };
            let handler_attr = opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| attribute.is_identifier(expected_attr))?;
            let JSXAttributeValue::ExpressionContainer(container) = handler_attr.value.as_ref()? else {
                return None;
            };
            let handler_span = container.expression.span();
            let has_prevent_default = ctx.semantic.nodes().iter().any(|candidate| {
                match candidate.kind() {
                    AstKind::CallExpression(call) => {
                        handler_span.start <= call.span.start
                            && call.span.end <= handler_span.end
                            && call
                                .callee
                                .get_member_expr()
                                .is_some_and(|member| member.static_property_name() == Some("preventDefault"))
                    }
                    _ => false,
                }
            });
            has_prevent_default.then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid relying on `preventDefault()` in JSX event handlers when native behavior should remain intact",
                    handler_attr.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_conditional_render_zero(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::LogicalExpression(logical) = node.kind() else {
                return None;
            };
            is_length_and_jsx(logical).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "`.length && <JSX>` can render `0`; compare the length explicitly instead",
                    logical.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn is_length_and_jsx(logical: &LogicalExpression<'_>) -> bool {
    logical.operator == LogicalOperator::And
        && logical
            .left
            .without_parentheses()
            .get_member_expr()
            .is_some_and(|member| member.static_property_name() == Some("length"))
        && matches!(
            logical.right.without_parentheses(),
            Expression::JSXElement(_) | Expression::JSXFragment(_)
        )
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
    fn flags_array_index_key() {
        let messages = rule_messages(
            "react/no-array-index-key",
            "export function List({ items }) {\n  return items.map((item, index) => <div key={index}>{item}</div>);\n}\n",
        );
        assert_eq!(messages, vec!["Avoid using array indexes as React keys"]);
    }

    #[test]
    fn flags_prevent_default_in_form() {
        let messages = rule_messages(
            "react/no-prevent-default",
            "export function Form() {\n  return <form onSubmit={(event) => { event.preventDefault(); }} />;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid relying on `preventDefault()` in JSX event handlers when native behavior should remain intact"]
        );
    }

    #[test]
    fn flags_conditional_render_zero() {
        let messages = rule_messages(
            "react/no-conditional-render-zero",
            "export function List({ items }) {\n  return <section>{items.length && <ul />}</section>;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["`.length && <JSX>` can render `0`; compare the length explicitly instead"]
        );
    }
}
