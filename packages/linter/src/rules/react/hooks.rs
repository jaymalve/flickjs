use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{
    Argument, ArrayExpression, CallExpression, Expression, FunctionBody, Statement,
};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use std::collections::HashSet;

use super::helpers::{
    callee_static_name, collect_dependency_names, collect_param_names, count_set_state_calls,
    dependency_element_name, file_uses_react, identifiers_in_span, is_component_name, is_hook_call,
    setter_name, span_contains, DEPENDENCY_HOOKS, EFFECT_HOOKS,
};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoDerivedStateEffect),
        Box::new(NoFetchInEffect),
        Box::new(NoCascadingSetState),
        Box::new(NoEffectEventHandler),
        Box::new(NoDerivedUseState),
        Box::new(PreferUseReducer),
        Box::new(LazyStateInit),
        Box::new(FunctionalSetState),
        Box::new(UnstableDeps),
    ]
}

macro_rules! react_hook_rule {
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

react_hook_rule!(
    NoDerivedStateEffect,
    "react/no-derived-state-effect",
    run_no_derived_state_effect
);
react_hook_rule!(
    NoFetchInEffect,
    "react/no-fetch-in-effect",
    run_no_fetch_in_effect
);
react_hook_rule!(
    NoCascadingSetState,
    "react/no-cascading-set-state",
    run_no_cascading_set_state
);
react_hook_rule!(
    NoEffectEventHandler,
    "react/no-effect-event-handler",
    run_no_effect_event_handler
);
react_hook_rule!(
    NoDerivedUseState,
    "react/no-derived-use-state",
    run_no_derived_use_state
);
react_hook_rule!(
    PreferUseReducer,
    "react/prefer-use-reducer",
    run_prefer_use_reducer
);
react_hook_rule!(LazyStateInit, "react/lazy-state-init", run_lazy_state_init);
react_hook_rule!(
    FunctionalSetState,
    "react/functional-set-state",
    run_functional_set_state
);
react_hook_rule!(UnstableDeps, "react/unstable-deps", run_unstable_deps);

fn run_no_derived_state_effect(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    iter_effect_calls(ctx)
        .filter_map(|(_, call, body, deps)| {
            let setter_call = sole_setter_call(body)?;
            let dependency_names = collect_dependency_names(deps);
            if dependency_names.is_empty() {
                return None;
            }
            let identifiers = identifiers_in_span(ctx, setter_argument_span(setter_call)?);
            if identifiers.is_empty() || !identifiers.is_subset(&dependency_names) {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Avoid deriving state from effect dependencies; derive it during render instead",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_fetch_in_effect(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    iter_effect_calls(ctx)
        .filter_map(|(_, call, body, _)| {
            callback_contains_fetch(ctx, body).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Move data fetching out of `useEffect` when possible",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_cascading_set_state(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    iter_effect_calls(ctx)
        .filter_map(|(_, call, body, _)| {
            (count_set_state_calls(body.node_id.get(), ctx) >= 3).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid cascading multiple state updates inside a single effect",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_effect_event_handler(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    iter_effect_calls(ctx)
        .filter_map(|(_, call, body, deps)| {
            if deps.elements.len() != 1 || body.statements.len() != 1 {
                return None;
            }
            let Statement::IfStatement(if_stmt) = &body.statements[0] else {
                return None;
            };
            let dep_name = deps.elements.iter().find_map(dependency_element_name)?;
            let test_identifiers = identifiers_in_span(ctx, if_stmt.test.span());
            if !test_identifiers.contains(&dep_name) {
                return None;
            }
            let consequent_span = if_stmt.consequent.span();
            let has_handler_like_call = ctx.semantic.nodes().iter().any(|node| {
                span_contains(consequent_span, node.kind().span())
                    && matches!(
                        node.kind(),
                        AstKind::CallExpression(effect_call)
                            if setter_name(effect_call).is_some()
                                || callee_static_name(&effect_call.callee)
                                    .is_some_and(|name| name.starts_with("on"))
                    )
            });
            has_handler_like_call.then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid using `useEffect` as an event handler wrapper",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_derived_use_state(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if callee_static_name(&call.callee).as_deref() != Some("useState") {
                return None;
            }
            let component = enclosing_component(ctx, node.id())?;
            let initial_arg = first_non_spread_argument(call)?;
            let derived = match initial_arg {
                Argument::Identifier(identifier) => {
                    component.param_names.contains(identifier.name.as_str())
                }
                Argument::StaticMemberExpression(member) => component
                    .param_names
                    .contains(callee_static_name(&member.object)?.as_str()),
                Argument::ComputedMemberExpression(member) => component
                    .param_names
                    .contains(callee_static_name(&member.object)?.as_str()),
                _ => false,
            };
            derived.then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid initializing state directly from component props",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_prefer_use_reducer(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let mut seen = HashSet::new();
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let component = match node.kind() {
            AstKind::Function(function) if function.body.is_some() => {
                named_component_from_function(function).map(|name| {
                    (
                        node.id(),
                        function.span,
                        component_param_names(&function.params),
                        name,
                    )
                })
            }
            AstKind::ArrowFunctionExpression(function) => {
                named_component_from_arrow(ctx, node.id()).map(|name| {
                    (
                        node.id(),
                        function.span,
                        component_param_names(&function.params),
                        name,
                    )
                })
            }
            _ => None,
        };
        let Some((component_id, span, _, _)) = component else {
            continue;
        };
        if !seen.insert(component_id) {
            continue;
        }
        let use_state_calls = count_named_calls_in_span(ctx, span, "useState");
        if use_state_calls >= 5 {
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Consider `useReducer` when a component owns many `useState` hooks",
                span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn run_lazy_state_init(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if callee_static_name(&call.callee).as_deref() != Some("useState") {
                return None;
            }
            matches!(
                first_non_spread_argument(call)?,
                Argument::CallExpression(_)
            )
            .then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Wrap expensive `useState` initialization in a lazy initializer",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_functional_set_state(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let setter = setter_name(call)?;
            let arg = first_non_spread_argument(call)?;
            if matches!(arg, Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_))
            {
                return None;
            }
            let state_name = state_name_from_setter(&setter)?;
            let identifiers = identifiers_in_span(ctx, arg.span());
            identifiers.contains(&state_name).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Use functional `setState` updates when the next value depends on previous state",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_unstable_deps(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_hook_call(call, &DEPENDENCY_HOOKS) {
                return None;
            }
            let deps = dependency_array(call)?;
            let unstable = deps.elements.iter().any(|element| match element {
                oxc_ast::ast::ArrayExpressionElement::ArrayExpression(_)
                | oxc_ast::ast::ArrayExpressionElement::ObjectExpression(_)
                | oxc_ast::ast::ArrayExpressionElement::ArrowFunctionExpression(_)
                | oxc_ast::ast::ArrayExpressionElement::FunctionExpression(_)
                | oxc_ast::ast::ArrayExpressionElement::NewExpression(_) => true,
                _ => false,
            });
            unstable.then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid object, array, or function literals in hook dependency arrays",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn iter_effect_calls<'a>(
    ctx: &'a LintContext<'a>,
) -> impl Iterator<
    Item = (
        NodeId,
        &'a CallExpression<'a>,
        &'a FunctionBody<'a>,
        &'a ArrayExpression<'a>,
    ),
> + 'a {
    ctx.semantic.nodes().iter().filter_map(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return None;
        };
        if !is_hook_call(call, &EFFECT_HOOKS) {
            return None;
        }
        let body = effect_callback_body(call)?;
        let deps = dependency_array(call)?;
        Some((node.id(), call, body, deps))
    })
}

fn effect_callback_body<'a>(call: &'a CallExpression<'a>) -> Option<&'a FunctionBody<'a>> {
    match call.arguments.first()? {
        Argument::ArrowFunctionExpression(function) => Some(&function.body),
        Argument::FunctionExpression(function) => function.body.as_deref(),
        _ => None,
    }
}

fn dependency_array<'a>(call: &'a CallExpression<'a>) -> Option<&'a ArrayExpression<'a>> {
    match call.arguments.get(1)? {
        Argument::ArrayExpression(array) => Some(array),
        _ => None,
    }
}

fn first_non_spread_argument<'a>(call: &'a CallExpression<'a>) -> Option<&'a Argument<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        argument => Some(argument),
    }
}

fn callback_contains_fetch(ctx: &LintContext, body: &FunctionBody<'_>) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        span_contains(body.span, node.kind().span())
            && matches!(
                node.kind(),
                AstKind::CallExpression(call)
                    if is_fetch_like_call(call)
            )
    })
}

fn is_fetch_like_call(call: &CallExpression<'_>) -> bool {
    callee_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "fetch" | "axios" | "axios.get" | "axios.post" | "axios.request" | "ky" | "ky.get"
        )
    })
}

fn sole_setter_call<'a>(body: &'a FunctionBody<'a>) -> Option<&'a CallExpression<'a>> {
    if body.statements.len() != 1 {
        return None;
    }
    let Statement::ExpressionStatement(expr_stmt) = &body.statements[0] else {
        return None;
    };
    let Expression::CallExpression(call) = expr_stmt.expression.without_parentheses() else {
        return None;
    };
    setter_name(call)?;
    Some(call)
}

fn setter_argument_span(call: &CallExpression<'_>) -> Option<Span> {
    Some(first_non_spread_argument(call)?.span())
}

struct ComponentInfo {
    param_names: HashSet<String>,
}

fn enclosing_component(ctx: &LintContext, node_id: NodeId) -> Option<ComponentInfo> {
    for ancestor_id in ctx.semantic.nodes().ancestor_ids(node_id) {
        match ctx.semantic.nodes().kind(ancestor_id) {
            AstKind::Function(function) if function.body.is_some() => {
                if named_component_from_function(function).is_some() {
                    return Some(ComponentInfo {
                        param_names: component_param_names(&function.params),
                    });
                }
            }
            AstKind::ArrowFunctionExpression(function) => {
                if named_component_from_arrow(ctx, ancestor_id).is_some() {
                    return Some(ComponentInfo {
                        param_names: component_param_names(&function.params),
                    });
                }
            }
            _ => {}
        }
    }
    None
}

fn named_component_from_function(function: &oxc_ast::ast::Function<'_>) -> Option<String> {
    let name = function.id.as_ref()?.name.to_string();
    is_component_name(&name).then_some(name)
}

fn named_component_from_arrow(ctx: &LintContext, node_id: NodeId) -> Option<String> {
    let AstKind::VariableDeclarator(decl) = ctx.semantic.nodes().parent_kind(node_id) else {
        return None;
    };
    let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = &decl.id else {
        return None;
    };
    let name = identifier.name.to_string();
    is_component_name(&name).then_some(name)
}

fn component_param_names(params: &oxc_ast::ast::FormalParameters<'_>) -> HashSet<String> {
    let mut names = collect_param_names(params);
    names.insert("props".to_string());
    names
}

fn count_named_calls_in_span(ctx: &LintContext, span: Span, name: &str) -> usize {
    ctx.semantic
        .nodes()
        .iter()
        .filter(|node| {
            span_contains(span, node.kind().span())
                && matches!(
                    node.kind(),
                    AstKind::CallExpression(call)
                        if callee_static_name(&call.callee).as_deref() == Some(name)
                )
        })
        .count()
}

fn state_name_from_setter(setter: &str) -> Option<String> {
    let rest = setter.strip_prefix("set")?;
    let mut chars = rest.chars();
    let first = chars.next()?.to_ascii_lowercase();
    Some(format!("{first}{}", chars.as_str()))
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
    fn flags_derived_state_effect() {
        let messages = rule_messages(
            "react/no-derived-state-effect",
            "import { useEffect, useState } from 'react';\nfunction Demo({ value }) {\n  const [derived, setDerived] = useState(0);\n  useEffect(() => { setDerived(value); }, [value]);\n  return derived;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid deriving state from effect dependencies; derive it during render instead"]
        );
    }

    #[test]
    fn flags_fetch_in_effect() {
        let messages = rule_messages(
            "react/no-fetch-in-effect",
            "import { useEffect } from 'react';\nfunction Demo() {\n  useEffect(() => { fetch('/api'); }, []);\n  return null;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Move data fetching out of `useEffect` when possible"]
        );
    }

    #[test]
    fn flags_cascading_set_state() {
        let messages = rule_messages(
            "react/no-cascading-set-state",
            "import { useEffect, useState } from 'react';\nfunction Demo() {\n  const [a, setA] = useState(0);\n  const [b, setB] = useState(0);\n  const [c, setC] = useState(0);\n  useEffect(() => { setA(1); setB(2); setC(3); }, []);\n  return a + b + c;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid cascading multiple state updates inside a single effect"]
        );
    }

    #[test]
    fn flags_effect_event_handler_shape() {
        let messages = rule_messages(
            "react/no-effect-event-handler",
            "import { useEffect, useState } from 'react';\nfunction Demo({ open }) {\n  const [value, setValue] = useState(0);\n  useEffect(() => { if (open) { setValue(1); } }, [open]);\n  return value;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid using `useEffect` as an event handler wrapper"]
        );
    }

    #[test]
    fn flags_derived_use_state() {
        let messages = rule_messages(
            "react/no-derived-use-state",
            "import { useState } from 'react';\nfunction Demo({ value }) {\n  const [derived] = useState(value);\n  return derived;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid initializing state directly from component props"]
        );
    }

    #[test]
    fn flags_many_use_state_calls() {
        let messages = rule_messages(
            "react/prefer-use-reducer",
            "import { useState } from 'react';\nfunction Demo() {\n  const [a] = useState(0);\n  const [b] = useState(0);\n  const [c] = useState(0);\n  const [d] = useState(0);\n  const [e] = useState(0);\n  return a + b + c + d + e;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Consider `useReducer` when a component owns many `useState` hooks"]
        );
    }

    #[test]
    fn flags_eager_state_init() {
        let messages = rule_messages(
            "react/lazy-state-init",
            "import { useState } from 'react';\nfunction Demo() {\n  const [value] = useState(expensive());\n  return value;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap expensive `useState` initialization in a lazy initializer"]
        );
    }

    #[test]
    fn flags_non_functional_set_state() {
        let messages = rule_messages(
            "react/functional-set-state",
            "import { useState } from 'react';\nfunction Demo() {\n  const [count, setCount] = useState(0);\n  setCount(count + 1);\n  return count;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Use functional `setState` updates when the next value depends on previous state"]
        );
    }

    #[test]
    fn flags_unstable_deps() {
        let messages = rule_messages(
            "react/unstable-deps",
            "import { useEffect } from 'react';\nfunction Demo() {\n  useEffect(() => {}, [{}]);\n  return null;\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid object, array, or function literals in hook dependency arrays"]
        );
    }
}
