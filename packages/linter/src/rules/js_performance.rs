use oxc_ast::ast::{
    AssignmentExpression, CallExpression, ChainElement, Expression, IdentifierReference,
    NumericLiteral, Statement, VariableDeclaration,
};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use std::collections::{HashMap, HashSet};

use super::{LintContext, LintDiagnostic, LintRule, Severity};

const ARRAY_ITERATION_METHODS: &[&str] = &[
    "map", "filter", "flatMap", "reduce", "some", "every", "find",
];

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoChainedArrayIterations),
        Box::new(PreferToSorted),
        Box::new(NoRegexpInLoop),
        Box::new(PreferMathMinMax),
        Box::new(NoArrayIncludesInLoop),
        Box::new(NoSequentialStyleAssignment),
        Box::new(NoArrayFindInLoop),
        Box::new(NoDuplicateStorageReads),
        Box::new(NoDeepNesting),
        Box::new(PreferPromiseAll),
    ]
}

macro_rules! perf_rule {
    ($name:ident, $rule_name:literal, $run_fn:ident) => {
        pub struct $name;

        impl LintRule for $name {
            fn name(&self) -> &'static str {
                $rule_name
            }

            fn default_severity(&self) -> Severity {
                Severity::Warning
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                $run_fn(ctx, self.name())
            }
        }
    };
}

perf_rule!(
    NoChainedArrayIterations,
    "no-chained-array-iterations",
    run_no_chained_array_iterations
);
perf_rule!(PreferToSorted, "prefer-tosorted", run_prefer_tosorted);
perf_rule!(NoRegexpInLoop, "no-regexp-in-loop", run_no_regexp_in_loop);
perf_rule!(
    PreferMathMinMax,
    "prefer-math-min-max",
    run_prefer_math_min_max
);
perf_rule!(
    NoArrayIncludesInLoop,
    "no-array-includes-in-loop",
    run_no_array_includes_in_loop
);
perf_rule!(
    NoSequentialStyleAssignment,
    "no-sequential-style-assignment",
    run_no_sequential_style_assignment
);
perf_rule!(
    NoArrayFindInLoop,
    "no-array-find-in-loop",
    run_no_array_find_in_loop
);
perf_rule!(
    NoDuplicateStorageReads,
    "no-duplicate-storage-reads",
    run_no_duplicate_storage_reads
);
perf_rule!(NoDeepNesting, "no-deep-nesting", run_no_deep_nesting);
perf_rule!(
    PreferPromiseAll,
    "prefer-promise-all",
    run_prefer_promise_all
);

fn run_no_chained_array_iterations(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let member = call.callee.get_member_expr()?;
            let outer_property = member.static_property_name()?;
            if !ARRAY_ITERATION_METHODS.contains(&outer_property) {
                return None;
            }
            let inner_call = match member.object().without_parentheses() {
                Expression::CallExpression(call) => call,
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::CallExpression(call) => call,
                    _ => return None,
                },
                _ => return None,
            };
            let inner_property = inner_call
                .callee
                .get_member_expr()?
                .static_property_name()?;
            if !ARRAY_ITERATION_METHODS.contains(&inner_property) {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Collapse chained array iterations into a single pass when possible",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_prefer_tosorted(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let member = call.callee.get_member_expr()?;
            if member.static_property_name() != Some("sort") {
                return None;
            }
            let Expression::ArrayExpression(array) = member.object().without_parentheses() else {
                return None;
            };
            let has_spread = array.elements.iter().any(|element| {
                matches!(
                    element,
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(_)
                )
            });
            if !has_spread {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Prefer `toSorted()` over cloning an array and calling `sort()`",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_regexp_in_loop(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::NewExpression(new_expr) = node.kind() else {
                return None;
            };
            let Expression::Identifier(identifier) = new_expr.callee.without_parentheses() else {
                return None;
            };
            if !is_unshadowed_global_identifier(ctx, identifier, "RegExp")
                || !is_inside_loop(ctx, node.id())
            {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Move regular expression construction out of the loop",
                new_expr.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_prefer_math_min_max(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ComputedMemberExpression(member) = node.kind() else {
                return None;
            };
            if !matches_zero_index(&member.expression) {
                return None;
            }
            let sort_call = match member.object.without_parentheses() {
                Expression::CallExpression(call) => call,
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::CallExpression(call) => call,
                    _ => return None,
                },
                _ => return None,
            };
            if sort_call.callee.get_member_expr()?.static_property_name() != Some("sort") {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Use `Math.min`/`Math.max` instead of sorting to read the first element",
                member.span(),
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_array_includes_in_loop(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    loop_member_call_rule(
        ctx,
        rule_name,
        "includes",
        "Hoist repeated `includes()` lookups out of loops or use a Set",
    )
}

fn run_no_array_find_in_loop(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    loop_member_call_rule(
        ctx,
        rule_name,
        "find",
        "Hoist repeated `find()` lookups out of loops or pre-index the data",
    )
}

fn run_no_sequential_style_assignment(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    let mut seen: HashMap<(NodeId, String), usize> = HashMap::new();
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::AssignmentExpression(assignment) = node.kind() else {
            continue;
        };
        let Some(style_key) = style_assignment_key(assignment) else {
            continue;
        };
        let Some(container_id) = nearest_block_or_program(ctx, node.id()) else {
            continue;
        };
        let count = seen.entry((container_id, style_key)).or_insert(0);
        *count += 1;
        if *count >= 2 {
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Group repeated `.style` mutations into a single style update",
                assignment.span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn run_no_duplicate_storage_reads(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    let mut seen: HashSet<(NodeId, String)> = HashSet::new();
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        let Some(storage_key) = storage_read_key(call) else {
            continue;
        };
        let Some(container_id) = nearest_container(ctx, node.id()) else {
            continue;
        };
        let key = (container_id, storage_key);
        if !seen.insert(key) {
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Avoid duplicate storage reads within the same scope",
                call.span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn run_no_deep_nesting(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::IfStatement(if_stmt) = node.kind() else {
                return None;
            };
            let depth = ctx
                .semantic
                .nodes()
                .ancestor_kinds(node.id())
                .filter(|kind| matches!(kind, AstKind::IfStatement(_)))
                .count()
                + 1;
            if depth < 4 {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Reduce deeply nested conditionals",
                if_stmt.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_prefer_promise_all(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        match node.kind() {
            AstKind::BlockStatement(block) => {
                diagnostics.extend(promise_all_diagnostics_in_statements(
                    ctx,
                    rule_name,
                    &block.body,
                ));
            }
            AstKind::FunctionBody(body) => {
                diagnostics.extend(promise_all_diagnostics_in_statements(
                    ctx,
                    rule_name,
                    &body.statements,
                ));
            }
            _ => {}
        }
    }

    diagnostics
}

fn loop_member_call_rule(
    ctx: &LintContext,
    rule_name: &'static str,
    property: &str,
    message: &str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if call.callee.get_member_expr()?.static_property_name() != Some(property)
                || !is_inside_loop(ctx, node.id())
            {
                return None;
            }
            Some(ctx.diagnostic(rule_name, message, call.span, Severity::Warning))
        })
        .collect()
}

fn promise_all_diagnostics_in_statements(
    ctx: &LintContext,
    rule_name: &'static str,
    statements: &[Statement<'_>],
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut streak: Vec<Span> = Vec::new();

    for statement in statements {
        if await_statement_span(statement).is_some() {
            streak.push(statement.span());
            continue;
        }

        if streak.len() >= 3 {
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Combine sequential independent awaits with `Promise.all()` when possible",
                *streak.last().unwrap(),
                Severity::Warning,
            ));
        }
        streak.clear();
    }

    if streak.len() >= 3 {
        diagnostics.push(ctx.diagnostic(
            rule_name,
            "Combine sequential independent awaits with `Promise.all()` when possible",
            *streak.last().unwrap(),
            Severity::Warning,
        ));
    }

    diagnostics
}

fn await_statement_span(statement: &Statement<'_>) -> Option<Span> {
    match statement {
        Statement::ExpressionStatement(expr_stmt)
            if matches!(
                expr_stmt.expression.without_parentheses(),
                Expression::AwaitExpression(_)
            ) =>
        {
            Some(expr_stmt.expression.span())
        }
        Statement::VariableDeclaration(var_decl) => variable_declaration_await_span(var_decl),
        _ => None,
    }
}

fn variable_declaration_await_span(var_decl: &VariableDeclaration<'_>) -> Option<Span> {
    if var_decl.declarations.len() != 1 {
        return None;
    }
    let init = var_decl.declarations[0].init.as_ref()?;
    match init.without_parentheses() {
        Expression::AwaitExpression(await_expr) => Some(await_expr.span),
        _ => None,
    }
}

fn style_assignment_key(assignment: &AssignmentExpression<'_>) -> Option<String> {
    let left_member = assignment.left.as_member_expression()?;
    left_member.static_property_name()?;
    let style_member = left_member.object().get_member_expr()?;
    if style_member.static_property_name() != Some("style") {
        return None;
    }
    let root = expression_static_name(style_member.object())?;
    Some(root)
}

fn storage_read_key(call: &CallExpression<'_>) -> Option<String> {
    let member = call.callee.get_member_expr()?;
    let object = expression_static_name(member.object())?;
    if !matches!(object.as_str(), "localStorage" | "sessionStorage")
        || member.static_property_name() != Some("getItem")
    {
        return None;
    }
    let key = string_like_argument(call.arguments.first()?)?;
    Some(format!("{object}:{key}"))
}

fn nearest_container(ctx: &LintContext, node_id: NodeId) -> Option<NodeId> {
    ctx.semantic
        .nodes()
        .ancestor_ids(node_id)
        .find(|ancestor_id| {
            matches!(
                ctx.semantic.nodes().kind(*ancestor_id),
                AstKind::Program(_)
                    | AstKind::Function(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::BlockStatement(_)
            )
        })
}

fn nearest_block_or_program(ctx: &LintContext, node_id: NodeId) -> Option<NodeId> {
    ctx.semantic
        .nodes()
        .ancestor_ids(node_id)
        .find(|ancestor_id| {
            matches!(
                ctx.semantic.nodes().kind(*ancestor_id),
                AstKind::Program(_) | AstKind::BlockStatement(_)
            )
        })
}

fn is_inside_loop(ctx: &LintContext, node_id: NodeId) -> bool {
    ctx.semantic.nodes().ancestor_kinds(node_id).any(|kind| {
        matches!(
            kind,
            AstKind::ForStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
        )
    })
}

fn expression_static_name(expression: &Expression<'_>) -> Option<String> {
    let expression = expression.without_parentheses();
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => expression
            .get_member_expr()
            .and_then(member_expression_name),
    }
}

fn member_expression_name(member: &oxc_ast::ast::MemberExpression<'_>) -> Option<String> {
    let object = expression_static_name(member.object())?;
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}

fn string_like_argument(argument: &oxc_ast::ast::Argument<'_>) -> Option<String> {
    match argument {
        oxc_ast::ast::Argument::StringLiteral(lit) => Some(lit.value.to_string()),
        oxc_ast::ast::Argument::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.to_string())
        }
        _ => None,
    }
}

fn matches_zero_index(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::NumericLiteral(number) => numeric_literal_is_zero(number),
        _ => false,
    }
}

fn numeric_literal_is_zero(number: &NumericLiteral<'_>) -> bool {
    number.value == 0.0
}

fn is_unshadowed_global_identifier(
    ctx: &LintContext,
    identifier: &IdentifierReference<'_>,
    name: &str,
) -> bool {
    identifier.name == name
        && identifier
            .reference_id
            .get()
            .and_then(|reference_id| {
                ctx.semantic
                    .scoping()
                    .get_reference(reference_id)
                    .symbol_id()
            })
            .is_none()
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn rule_messages(rule_name: &str, path: &str, source: &str) -> Vec<String> {
        lint_source_for_test(path, source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_chained_array_iterations() {
        let messages = rule_messages(
            "no-chained-array-iterations",
            "test.ts",
            "items.map(fn).filter(Boolean);\n",
        );
        assert_eq!(
            messages,
            vec!["Collapse chained array iterations into a single pass when possible"]
        );
    }

    #[test]
    fn flags_clone_then_sort() {
        let messages = rule_messages("prefer-tosorted", "test.ts", "[...items].sort();\n");
        assert_eq!(
            messages,
            vec!["Prefer `toSorted()` over cloning an array and calling `sort()`"]
        );
    }

    #[test]
    fn flags_regexp_in_loop() {
        let messages = rule_messages(
            "no-regexp-in-loop",
            "test.ts",
            "for (const item of items) { new RegExp(item); }\n",
        );
        assert_eq!(
            messages,
            vec!["Move regular expression construction out of the loop"]
        );
    }

    #[test]
    fn flags_sort_then_zero_index() {
        let messages = rule_messages(
            "prefer-math-min-max",
            "test.ts",
            "const first = values.sort()[0];\n",
        );
        assert_eq!(
            messages,
            vec!["Use `Math.min`/`Math.max` instead of sorting to read the first element"]
        );
    }

    #[test]
    fn flags_includes_in_loop() {
        let messages = rule_messages(
            "no-array-includes-in-loop",
            "test.ts",
            "for (const item of items) { selected.includes(item); }\n",
        );
        assert_eq!(
            messages,
            vec!["Hoist repeated `includes()` lookups out of loops or use a Set"]
        );
    }

    #[test]
    fn flags_find_in_loop() {
        let messages = rule_messages(
            "no-array-find-in-loop",
            "test.ts",
            "while (ready) { collection.find(match); }\n",
        );
        assert_eq!(
            messages,
            vec!["Hoist repeated `find()` lookups out of loops or pre-index the data"]
        );
    }

    #[test]
    fn flags_repeated_style_assignment() {
        let messages = rule_messages(
            "no-sequential-style-assignment",
            "test.ts",
            "el.style.color = 'red';\nel.style.display = 'none';\n",
        );
        assert_eq!(
            messages,
            vec!["Group repeated `.style` mutations into a single style update"]
        );
    }

    #[test]
    fn flags_duplicate_storage_reads() {
        let messages = rule_messages(
            "no-duplicate-storage-reads",
            "test.ts",
            "const a = localStorage.getItem('theme');\nconst b = localStorage.getItem('theme');\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid duplicate storage reads within the same scope"]
        );
    }

    #[test]
    fn flags_deep_if_nesting() {
        let messages = rule_messages(
            "no-deep-nesting",
            "test.ts",
            "if (a) { if (b) { if (c) { if (d) { work(); } } } }\n",
        );
        assert_eq!(messages, vec!["Reduce deeply nested conditionals"]);
    }

    #[test]
    fn flags_three_sequential_awaits() {
        let messages = rule_messages(
            "prefer-promise-all",
            "test.ts",
            "async function run() {\n  const a = await fetchA();\n  const b = await fetchB();\n  const c = await fetchC();\n}\n",
        );
        assert_eq!(
            messages,
            vec!["Combine sequential independent awaits with `Promise.all()` when possible"]
        );
    }
}
