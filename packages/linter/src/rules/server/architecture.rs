use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::Argument;
use oxc_ast::AstKind;
use oxc_span::Span;

use super::helpers::{
    enclosing_function_span, expression_static_name, is_orm_mutate_call, is_route_registration,
    span_contains,
};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(RequireInputValidation),
        Box::new(NoFloatingTransaction),
        Box::new(NoBusinessLogicInRoute),
    ]
}

macro_rules! server_rule {
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
                project.has_server_framework()
            }

            fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
                if !ctx.project.has_server_framework() {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

server_rule!(
    RequireInputValidation,
    "server/require-input-validation",
    run_require_input_validation
);
server_rule!(
    NoFloatingTransaction,
    "server/no-floating-transaction",
    run_no_floating_transaction
);
server_rule!(
    NoBusinessLogicInRoute,
    "server/no-business-logic-in-route",
    run_no_business_logic_in_route
);

fn run_require_input_validation(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        if !is_route_registration(call) {
            continue;
        }
        for (name_span, body_span) in route_handler_spans(call) {
            if !handler_uses_request_body(ctx, body_span) || handler_has_validation(ctx, body_span)
            {
                continue;
            }
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Validate request bodies before using them in route handlers",
                name_span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn run_no_floating_transaction(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let mut spans = Vec::<Span>::new();
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        if !is_orm_mutate_call(call) {
            continue;
        }
        let Some(function_span) = enclosing_function_span(ctx, node.id()) else {
            continue;
        };
        if spans.iter().any(|existing| *existing == function_span) {
            continue;
        }
        let mutation_count = count_mutations_in_span(ctx, function_span);
        if mutation_count < 2 || span_has_transaction(ctx, function_span) {
            continue;
        }
        spans.push(function_span);
        diagnostics.push(ctx.diagnostic(
            rule_name,
            "Wrap multiple ORM mutations in a transaction",
            call.span,
            Severity::Warning,
        ));
    }

    diagnostics
}

fn run_no_business_logic_in_route(
    ctx: &LintContext,
    rule_name: &'static str,
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        if !is_route_registration(call) {
            continue;
        }
        for (name_span, body_span) in route_handler_spans(call) {
            let start_line = ctx.offset_to_line_col(body_span.start as usize).0;
            let end_line = ctx.offset_to_line_col(body_span.end as usize).0;
            let lines = end_line.saturating_sub(start_line) + 1;
            if lines <= 50 {
                continue;
            }
            diagnostics.push(ctx.diagnostic(
                rule_name,
                format!("Route handlers over 50 lines should move business logic into services ({lines} lines)"),
                name_span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn handler_has_validation(ctx: &LintContext, handler_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(handler_span, call.span)
            && expression_static_name(&call.callee).is_some_and(|name| {
                name.ends_with(".parse")
                    || name.ends_with(".safeParse")
                    || name.ends_with(".validate")
                    || name == "validate"
                    || name == "z.parse"
                    || name == "Joi.attempt"
            })
    })
}

fn handler_uses_request_body(ctx: &LintContext, handler_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| match node.kind() {
        AstKind::StaticMemberExpression(member) if span_contains(handler_span, member.span) => {
            let object_name = expression_static_name(&member.object);
            object_name
                .as_deref()
                .is_some_and(|name| matches!(name, "req.body" | "request.body"))
                || (matches!(object_name.as_deref(), Some("req" | "request"))
                    && member.property.name == "body")
        }
        AstKind::ComputedMemberExpression(member) if span_contains(handler_span, member.span) => {
            let object_name = expression_static_name(&member.object);
            object_name
                .as_deref()
                .is_some_and(|name| matches!(name, "req.body" | "request.body"))
                || (matches!(object_name.as_deref(), Some("req" | "request"))
                    && member
                        .static_property_name()
                        .is_some_and(|name| name == "body"))
        }
        _ => false,
    })
}

fn count_mutations_in_span(ctx: &LintContext, span: Span) -> usize {
    ctx.semantic
        .nodes()
        .iter()
        .filter(|node| {
            matches!(
                node.kind(),
                AstKind::CallExpression(call) if span_contains(span, call.span) && is_orm_mutate_call(call)
            )
        })
        .count()
}

fn span_has_transaction(ctx: &LintContext, span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(span, call.span)
            && expression_static_name(&call.callee).is_some_and(|name| {
                name.ends_with(".transaction")
                    || name.ends_with(".$transaction")
                    || name == "transaction"
            })
    })
}

fn route_handler_spans(call: &oxc_ast::ast::CallExpression<'_>) -> Vec<(Span, Span)> {
    call.arguments
        .iter()
        .filter_map(|argument| match argument {
            Argument::ArrowFunctionExpression(function) => {
                Some((function.span, function.body.span))
            }
            Argument::FunctionExpression(function) => {
                Some((function.span, function.body.as_ref()?.span))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("server.ts", source, &ProjectInfo::test_server())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_missing_input_validation() {
        let messages = rule_messages(
            "server/require-input-validation",
            "app.post('/users', (req, res) => { save(req.body.email); res.json({ ok: true }); });\n",
        );
        assert_eq!(
            messages,
            vec!["Validate request bodies before using them in route handlers"]
        );
    }

    #[test]
    fn flags_floating_transaction() {
        let messages = rule_messages(
            "server/no-floating-transaction",
            "async function save() { await prisma.user.create({ data: {} }); await prisma.post.update({ data: {} }); }\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap multiple ORM mutations in a transaction"]
        );
    }

    #[test]
    fn flags_business_logic_in_route() {
        let mut source = String::from("app.get('/users', (req, res) => {\n");
        for _ in 0..52 {
            source.push_str("  doWork();\n");
        }
        source.push_str("  res.json({ ok: true });\n});\n");

        let messages = rule_messages("server/no-business-logic-in-route", &source);
        assert_eq!(
            messages,
            vec![
                "Route handlers over 50 lines should move business logic into services (55 lines)"
            ]
        );
    }
}
