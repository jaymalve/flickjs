use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, Expression, ImportDeclarationSpecifier, JSXAttribute,
    JSXAttributeValue, JSXOpeningElement, ObjectPropertyKind,
};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use std::collections::HashSet;
use std::sync::LazyLock;

use super::helpers::{callee_static_name, file_uses_react, is_hook_call};

static USE_MEMO_HOOKS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["useMemo"]));

static HEAVY_STATIC_IMPORTS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "framer-motion",
        "lodash",
        "moment",
        "chart.js",
        "monaco-editor",
        "three",
        "d3",
    ])
});

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoUseMemoSimpleExpr),
        Box::new(NoUnstableMotionProps),
        Box::new(NoLayoutAnimation),
        Box::new(NoAnimatePresenceInList),
        Box::new(NoMotionInList),
        Box::new(NoPropOnMemo),
        Box::new(NoHydrationFlicker),
        Box::new(NoTransitionAll),
        Box::new(NoWillChange),
        Box::new(NoBlurFilter),
        Box::new(NoHeavyShadow),
        Box::new(NoBarrelImport),
        Box::new(NoFullLodash),
        Box::new(NoMoment),
        Box::new(PreferDynamicImport),
        Box::new(UseLazyMotion),
        Box::new(NoUndeferredScript),
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
                if !ctx.project.has_react {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

react_rule!(
    NoUseMemoSimpleExpr,
    "react/no-usememo-simple-expr",
    run_no_usememo_simple_expr
);
react_rule!(
    NoUnstableMotionProps,
    "react/no-unstable-motion-props",
    run_no_unstable_motion_props
);
react_rule!(
    NoLayoutAnimation,
    "react/no-layout-animation",
    run_no_layout_animation
);
react_rule!(
    NoAnimatePresenceInList,
    "react/no-animate-presence-in-list",
    run_no_animate_presence_in_list
);
react_rule!(
    NoMotionInList,
    "react/no-motion-in-list",
    run_no_motion_in_list
);
react_rule!(NoPropOnMemo, "react/no-prop-on-memo", run_no_prop_on_memo);
react_rule!(
    NoHydrationFlicker,
    "react/no-hydration-flicker",
    run_no_hydration_flicker
);
react_rule!(
    NoTransitionAll,
    "react/no-transition-all",
    run_no_transition_all
);
react_rule!(NoWillChange, "react/no-will-change", run_no_will_change);
react_rule!(NoBlurFilter, "react/no-blur-filter", run_no_blur_filter);
react_rule!(NoHeavyShadow, "react/no-heavy-shadow", run_no_heavy_shadow);
react_rule!(
    NoBarrelImport,
    "react/no-barrel-import",
    run_no_barrel_import
);
react_rule!(NoFullLodash, "react/no-full-lodash", run_no_full_lodash);
react_rule!(NoMoment, "react/no-moment", run_no_moment);
react_rule!(
    PreferDynamicImport,
    "react/prefer-dynamic-import",
    run_prefer_dynamic_import
);
react_rule!(UseLazyMotion, "react/use-lazy-motion", run_use_lazy_motion);
react_rule!(
    NoUndeferredScript,
    "react/no-undeferred-script",
    run_no_undeferred_script
);

fn run_no_usememo_simple_expr(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    if !file_uses_react(ctx) {
        return Vec::new();
    }

    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_hook_call(call, &USE_MEMO_HOOKS) {
                return None;
            }
            let expression = callback_return_expression(call)?;
            is_simple_expression(expression).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid wrapping trivial expressions in `useMemo`",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_unstable_motion_props(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if !is_motion_component(opening) {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| {
                    matches!(
                        attribute.name.get_identifier().name.as_str(),
                        "animate"
                            | "initial"
                            | "exit"
                            | "transition"
                            | "variants"
                            | "whileHover"
                            | "whileTap"
                            | "whileInView"
                    ) && attribute_expression(attribute).is_some_and(is_unstable_prop_expression)
                })
                .map(|attribute| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid recreating motion prop objects and functions on every render",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_layout_animation(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            if !is_motion_component(opening) {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| attribute.is_identifier("layout"))
                .map(|attribute| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid layout animations on frequently rendered React surfaces",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_animate_presence_in_list(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXElement(element) = node.kind() else {
                return None;
            };
            if element.opening_element.name.to_string() != "AnimatePresence" {
                return None;
            }
            contains_map_call(ctx, element.span).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid wrapping list rendering directly in `AnimatePresence`",
                    element.opening_element.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_motion_in_list(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let member = call.callee.get_member_expr()?;
            if member.static_property_name() != Some("map") || !contains_motion_jsx(ctx, call.span)
            {
                return None;
            }
            Some(ctx.diagnostic(
                rule_name,
                "Avoid rendering motion components directly inside large `.map()` lists",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_prop_on_memo(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let memoized_components = collect_memoized_components(ctx);
    if memoized_components.is_empty() {
        return Vec::new();
    }

    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXOpeningElement(opening) = node.kind() else {
                return None;
            };
            let component_name = opening.name.get_identifier_name()?.to_string();
            if !memoized_components.contains(&component_name) {
                return None;
            }
            opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .find(|attribute| {
                    attribute_expression(attribute).is_some_and(is_unstable_prop_expression)
                })
                .map(|attribute| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid passing object, array, or function literals to memoized components",
                        attribute.span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_hydration_flicker(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    if !file_uses_react(ctx) {
        return Vec::new();
    }

    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            (callee_static_name(&call.callee).as_deref() == Some("useState"))
                .then(|| call.arguments.first()?.as_expression())
                .flatten()
                .filter(|expression| expression_uses_storage(ctx, expression.span()))
                .map(|expression| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid reading client-only storage during `useState` initialization",
                        expression.span(),
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_no_transition_all(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_style_string_rule(
        ctx,
        rule_name,
        |value| value.contains("transition-all") || value.contains("transition: all"),
        "Avoid `transition-all`; transition only the properties you need",
    )
}

fn run_no_will_change(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_style_string_rule(
        ctx,
        rule_name,
        |value| value.contains("will-change") || value.contains("willChange"),
        "Avoid broad `will-change` hints that keep the browser in a promoted state",
    )
}

fn run_no_blur_filter(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_style_string_rule(
        ctx,
        rule_name,
        |value| {
            value.contains("blur(") || value.contains("blur-") || value.contains("backdrop-blur")
        },
        "Avoid expensive blur and backdrop-blur effects on interactive UI paths",
    )
}

fn run_no_heavy_shadow(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    jsx_style_string_rule(
        ctx,
        rule_name,
        |value| {
            value.contains("shadow-2xl")
                || value.contains("shadow-[")
                || value.contains("box-shadow")
                || value.contains("boxShadow")
        },
        "Avoid large shadow effects on frequently updated UI",
    )
}

fn run_no_barrel_import(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    import_source_rule(
        ctx,
        rule_name,
        |source| {
            source.ends_with("/index")
                || source.ends_with("/index.ts")
                || source.ends_with("/index.tsx")
                || source.ends_with("/index.js")
                || source.ends_with("/index.jsx")
        },
        "Import from the concrete module instead of a barrel `index` file",
    )
}

fn run_no_full_lodash(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    import_source_rule(
        ctx,
        rule_name,
        |source| source == "lodash",
        "Avoid importing the full `lodash` package",
    )
}

fn run_no_moment(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    import_source_rule(
        ctx,
        rule_name,
        |source| source == "moment",
        "Avoid `moment`; prefer smaller modern date utilities",
    )
}

fn run_prefer_dynamic_import(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    import_source_rule(
        ctx,
        rule_name,
        |source| HEAVY_STATIC_IMPORTS.contains(source),
        "Consider dynamically importing heavy client-side dependencies",
    )
}

fn run_use_lazy_motion(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ImportDeclaration(import_decl) = node.kind() else {
                return None;
            };
            if import_decl.source.value.as_str() != "framer-motion" {
                return None;
            }
            let specifiers = import_decl.specifiers.as_ref()?;
            specifiers.iter().find_map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if specifier.imported.name() == "motion" =>
                {
                    Some(ctx.diagnostic(
                        rule_name,
                        "Prefer `LazyMotion` and `m` over importing `motion` directly",
                        specifier.span,
                        Severity::Warning,
                    ))
                }
                _ => None,
            })
        })
        .collect()
}

fn run_no_undeferred_script(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
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
            let has_src = opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .any(|attribute| attribute.is_identifier("src"));
            let has_async_or_defer = opening
                .attributes
                .iter()
                .filter_map(|item| item.as_attribute())
                .any(|attribute| {
                    attribute.is_identifier("async") || attribute.is_identifier("defer")
                });
            (has_src && !has_async_or_defer).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Add `async` or `defer` to external `<script>` tags",
                    opening.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn callback_return_expression<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a Expression<'a>> {
    let argument = call.arguments.first()?;
    match argument {
        Argument::ArrowFunctionExpression(function) => {
            function_body_return_expression(&function.body)
        }
        Argument::FunctionExpression(function) => function
            .body
            .as_ref()
            .and_then(|body| function_body_return_expression(body)),
        _ => None,
    }
}

fn function_body_return_expression<'a>(
    body: &'a oxc_ast::ast::FunctionBody<'a>,
) -> Option<&'a Expression<'a>> {
    match body.statements.as_slice() {
        [oxc_ast::ast::Statement::ReturnStatement(statement)] => statement.argument.as_ref(),
        [oxc_ast::ast::Statement::ExpressionStatement(statement)] => Some(&statement.expression),
        _ => None,
    }
}

fn is_simple_expression(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::Identifier(_)
        | Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_) => true,
        Expression::UnaryExpression(unary) => is_simple_expression(&unary.argument),
        Expression::BinaryExpression(binary) => {
            is_simple_expression(&binary.left) && is_simple_expression(&binary.right)
        }
        Expression::LogicalExpression(logical) => {
            is_simple_expression(&logical.left) && is_simple_expression(&logical.right)
        }
        Expression::TemplateLiteral(template) => {
            template.expressions.iter().all(is_simple_expression)
        }
        _ => false,
    }
}

fn is_motion_component(opening: &JSXOpeningElement<'_>) -> bool {
    opening.name.to_string().starts_with("motion.")
}

fn contains_map_call(ctx: &LintContext, span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        matches!(
            node.kind(),
            AstKind::CallExpression(call)
                if span_contains(span, call.span)
                    && call
                        .callee
                        .get_member_expr()
                        .and_then(|member| member.static_property_name())
                        == Some("map")
        )
    })
}

fn contains_motion_jsx(ctx: &LintContext, span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        matches!(
            node.kind(),
            AstKind::JSXOpeningElement(opening)
                if span_contains(span, opening.span) && is_motion_component(opening)
        )
    })
}

fn collect_memoized_components(ctx: &LintContext) -> HashSet<String> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::VariableDeclarator(decl) = node.kind() else {
                return None;
            };
            let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                return None;
            };
            let Expression::CallExpression(call) = decl.init.as_ref()?.without_parentheses() else {
                return None;
            };
            matches!(
                callee_static_name(&call.callee).as_deref(),
                Some("memo" | "React.memo")
            )
            .then(|| identifier.name.to_string())
        })
        .collect()
}

fn attribute_expression<'a>(attribute: &'a JSXAttribute<'a>) -> Option<&'a Expression<'a>> {
    let JSXAttributeValue::ExpressionContainer(container) = attribute.value.as_ref()? else {
        return None;
    };
    container.expression.as_expression()
}

fn is_unstable_prop_expression(expression: &Expression<'_>) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
    )
}

fn expression_uses_storage(ctx: &LintContext, span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| match node.kind() {
        AstKind::IdentifierReference(identifier) if span_contains(span, identifier.span) => {
            matches!(identifier.name.as_str(), "localStorage" | "sessionStorage")
        }
        AstKind::CallExpression(call) if span_contains(span, call.span) => {
            expression_static_storage_name(&call.callee)
        }
        AstKind::StaticMemberExpression(member) if span_contains(span, member.span) => {
            member.object.is_specific_id("window")
                && matches!(
                    member.property.name.as_str(),
                    "localStorage" | "sessionStorage"
                )
        }
        AstKind::ComputedMemberExpression(member) if span_contains(span, member.span) => {
            member.object.is_specific_id("window")
                && member
                    .static_property_name()
                    .is_some_and(|name| name == "localStorage" || name == "sessionStorage")
        }
        _ => false,
    })
}

fn expression_static_storage_name(expression: &Expression<'_>) -> bool {
    callee_static_name(expression).is_some_and(|name| {
        name.starts_with("localStorage")
            || name.starts_with("sessionStorage")
            || name.starts_with("window.localStorage")
            || name.starts_with("window.sessionStorage")
    })
}

fn jsx_style_string_rule(
    ctx: &LintContext,
    rule_name: &'static str,
    predicate: impl Fn(&str) -> bool,
    message: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::JSXAttribute(attribute) = node.kind() else {
                return None;
            };
            jsx_attribute_style_value(attribute)
                .filter(|value| predicate(value))
                .map(|_| ctx.diagnostic(rule_name, message, attribute.span, Severity::Warning))
        })
        .collect()
}

fn jsx_attribute_style_value(attribute: &JSXAttribute<'_>) -> Option<String> {
    if attribute.is_identifier("className") || attribute.is_identifier("class") {
        return jsx_attribute_static_string(attribute);
    }
    if !attribute.is_identifier("style") {
        return None;
    }
    let expression = attribute_expression(attribute)?;
    let Expression::ObjectExpression(object) = expression.without_parentheses() else {
        return None;
    };
    let mut values = Vec::new();
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(key) = property.key.static_name() else {
            continue;
        };
        let value = string_like_expression(&property.value)?;
        values.push(format!("{key}:{value}"));
    }
    (!values.is_empty()).then(|| values.join(";"))
}

fn jsx_attribute_static_string(attribute: &JSXAttribute<'_>) -> Option<String> {
    match attribute.value.as_ref()? {
        JSXAttributeValue::StringLiteral(literal) => Some(literal.value.to_string()),
        JSXAttributeValue::ExpressionContainer(container) => {
            string_like_jsx_expression(&container.expression)
        }
        _ => None,
    }
}

fn string_like_jsx_expression(expression: &oxc_ast::ast::JSXExpression<'_>) -> Option<String> {
    match expression {
        oxc_ast::ast::JSXExpression::StringLiteral(literal) => Some(literal.value.to_string()),
        oxc_ast::ast::JSXExpression::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.to_string())
        }
        _ => expression.as_expression().and_then(string_like_expression),
    }
}

fn string_like_expression(expression: &Expression<'_>) -> Option<String> {
    match expression.without_parentheses() {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.to_string())
        }
        _ => None,
    }
}

fn import_source_rule(
    ctx: &LintContext,
    rule_name: &'static str,
    predicate: impl Fn(&str) -> bool,
    message: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ImportDeclaration(import_decl) = node.kind() else {
                return None;
            };
            predicate(import_decl.source.value.as_str())
                .then(|| ctx.diagnostic(rule_name, message, import_decl.span, Severity::Warning))
        })
        .collect()
}

fn span_contains(outer: Span, inner: Span) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
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
    fn flags_simple_usememo() {
        let messages = rule_messages(
            "react/no-usememo-simple-expr",
            "import { useMemo } from 'react';\nexport function Demo({ a, b }) { const value = useMemo(() => a + b, [a, b]); return <div>{value}</div>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid wrapping trivial expressions in `useMemo`"]
        );
    }

    #[test]
    fn flags_unstable_motion_props() {
        let messages = rule_messages(
            "react/no-unstable-motion-props",
            "export function Demo() { return <motion.div animate={{ x: 10 }} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid recreating motion prop objects and functions on every render"]
        );
    }

    #[test]
    fn flags_layout_animation() {
        let messages = rule_messages(
            "react/no-layout-animation",
            "export function Demo() { return <motion.div layout />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid layout animations on frequently rendered React surfaces"]
        );
    }

    #[test]
    fn flags_animate_presence_in_list() {
        let messages = rule_messages(
            "react/no-animate-presence-in-list",
            "export function Demo({ items }) { return <AnimatePresence>{items.map(item => <div key={item} />)}</AnimatePresence>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid wrapping list rendering directly in `AnimatePresence`"]
        );
    }

    #[test]
    fn flags_motion_in_list() {
        let messages = rule_messages(
            "react/no-motion-in-list",
            "export function Demo({ items }) { return items.map(item => <motion.div key={item} />); }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid rendering motion components directly inside large `.map()` lists"]
        );
    }

    #[test]
    fn flags_prop_on_memo() {
        let messages = rule_messages(
            "react/no-prop-on-memo",
            "import { memo } from 'react'; const Card = memo(function Card(props) { return <div />; }); export function Demo() { return <Card options={{ dense: true }} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid passing object, array, or function literals to memoized components"]
        );
    }

    #[test]
    fn flags_hydration_flicker() {
        let messages = rule_messages(
            "react/no-hydration-flicker",
            "import { useState } from 'react'; export function Demo() { const [theme] = useState(localStorage.getItem('theme')); return <div>{theme}</div>; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid reading client-only storage during `useState` initialization"]
        );
    }

    #[test]
    fn flags_transition_all() {
        let messages = rule_messages(
            "react/no-transition-all",
            "export function Demo() { return <div className=\"transition-all duration-200\" />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid `transition-all`; transition only the properties you need"]
        );
    }

    #[test]
    fn flags_will_change() {
        let messages = rule_messages(
            "react/no-will-change",
            "export function Demo() { return <div style={{ willChange: 'transform' }} />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid broad `will-change` hints that keep the browser in a promoted state"]
        );
    }

    #[test]
    fn flags_blur_filter() {
        let messages = rule_messages(
            "react/no-blur-filter",
            "export function Demo() { return <div className=\"backdrop-blur-lg\" />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid expensive blur and backdrop-blur effects on interactive UI paths"]
        );
    }

    #[test]
    fn flags_heavy_shadow() {
        let messages = rule_messages(
            "react/no-heavy-shadow",
            "export function Demo() { return <div className=\"shadow-2xl\" />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid large shadow effects on frequently updated UI"]
        );
    }

    #[test]
    fn flags_barrel_import() {
        let messages = rule_messages(
            "react/no-barrel-import",
            "import { Button } from './components/index';\n",
        );
        assert_eq!(
            messages,
            vec!["Import from the concrete module instead of a barrel `index` file"]
        );
    }

    #[test]
    fn flags_full_lodash() {
        let messages = rule_messages("react/no-full-lodash", "import _ from 'lodash';\n");
        assert_eq!(messages, vec!["Avoid importing the full `lodash` package"]);
    }

    #[test]
    fn flags_moment() {
        let messages = rule_messages("react/no-moment", "import moment from 'moment';\n");
        assert_eq!(
            messages,
            vec!["Avoid `moment`; prefer smaller modern date utilities"]
        );
    }

    #[test]
    fn flags_prefer_dynamic_import() {
        let messages = rule_messages(
            "react/prefer-dynamic-import",
            "import * as motion from 'framer-motion';\n",
        );
        assert_eq!(
            messages,
            vec!["Consider dynamically importing heavy client-side dependencies"]
        );
    }

    #[test]
    fn flags_use_lazy_motion() {
        let messages = rule_messages(
            "react/use-lazy-motion",
            "import { motion } from 'framer-motion';\n",
        );
        assert_eq!(
            messages,
            vec!["Prefer `LazyMotion` and `m` over importing `motion` directly"]
        );
    }

    #[test]
    fn flags_undeferred_script() {
        let messages = rule_messages(
            "react/no-undeferred-script",
            "export function Demo() { return <script src=\"/main.js\" />; }\n",
        );
        assert_eq!(
            messages,
            vec!["Add `async` or `defer` to external `<script>` tags"]
        );
    }
}
