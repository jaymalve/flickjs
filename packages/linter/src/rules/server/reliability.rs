use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{Argument, Expression, ObjectExpression, ObjectPropertyKind, ThrowStatement};
use oxc_ast::AstKind;
use oxc_span::Span;

use super::helpers::{
    enclosing_route_handler_span, expression_static_name, is_inside_route_handler,
    is_route_registration, span_contains,
};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoUnhandledAsyncRoute),
        Box::new(NoSwallowedError),
        Box::new(NoProcessExitInHandler),
        Box::new(RequireErrorStatus),
        Box::new(NoThrowString),
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
    NoUnhandledAsyncRoute,
    "server/no-unhandled-async-route",
    run_no_unhandled_async_route
);
server_rule!(
    NoSwallowedError,
    "server/no-swallowed-error",
    run_no_swallowed_error
);
server_rule!(
    NoProcessExitInHandler,
    "server/no-process-exit-in-handler",
    run_no_process_exit_in_handler
);
server_rule!(
    RequireErrorStatus,
    "server/require-error-status",
    run_require_error_status
);
server_rule!(NoThrowString, "server/no-throw-string", run_no_throw_string);

fn run_no_unhandled_async_route(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();

    for node in ctx.semantic.nodes().iter() {
        let AstKind::CallExpression(call) = node.kind() else {
            continue;
        };
        if !is_route_registration(call) {
            continue;
        }

        for argument in &call.arguments {
            let Some((handler_span, body_span)) = async_handler_spans(argument) else {
                continue;
            };
            if handler_has_try(ctx, body_span) {
                continue;
            }
            diagnostics.push(ctx.diagnostic(
                rule_name,
                "Async route handlers should catch and translate errors explicitly",
                handler_span,
                Severity::Warning,
            ));
        }
    }

    diagnostics
}

fn run_no_swallowed_error(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CatchClause(clause) = node.kind() else {
                return None;
            };
            let is_empty = clause.body.body.is_empty();
            let is_console_only =
                !is_empty && clause.body.body.iter().all(is_console_only_statement);
            (is_empty || is_console_only).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Catch blocks should handle, rethrow, or surface the error",
                    clause.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_no_process_exit_in_handler(
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
            if !is_inside_route_handler(ctx, node.id()) {
                return None;
            }
            (expression_static_name(&call.callee).as_deref() == Some("process.exit")).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid terminating the process from inside a request handler",
                    call.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn run_require_error_status(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            let member = call.callee.get_member_expr()?;
            let response_method = member.static_property_name()?;
            if !matches!(response_method, "json" | "send") {
                return None;
            }
            let Some(root_name) = member_root_identifier(member) else {
                return None;
            };
            if !matches!(root_name, "res" | "reply") {
                return None;
            }
            let handler_span = enclosing_route_handler_span(ctx, node.id())?;
            let Expression::ObjectExpression(object) = first_argument_expression(call)? else {
                return None;
            };
            if !object_has_error_payload(object) || handler_has_status_call(ctx, handler_span) {
                return None;
            }

            Some(ctx.diagnostic(
                rule_name,
                "Set an HTTP error status before returning error payloads",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn run_no_throw_string(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::ThrowStatement(throw_stmt) = node.kind() else {
                return None;
            };
            is_string_throw(throw_stmt).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Throw `Error` objects instead of string literals",
                    throw_stmt.span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

fn async_handler_spans(argument: &Argument<'_>) -> Option<(Span, Span)> {
    match argument {
        Argument::ArrowFunctionExpression(function) if function.r#async => {
            Some((function.span, function.body.span))
        }
        Argument::FunctionExpression(function) if function.r#async => {
            Some((function.span, function.body.as_ref()?.span))
        }
        _ => None,
    }
}

fn handler_has_try(ctx: &LintContext, body_span: Span) -> bool {
    ctx.semantic
        .nodes()
        .iter()
        .any(|node| matches!(node.kind(), AstKind::TryStatement(try_stmt) if span_contains(body_span, try_stmt.span)))
}

fn is_console_only_statement(statement: &oxc_ast::ast::Statement<'_>) -> bool {
    let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = statement else {
        return false;
    };
    let Expression::CallExpression(call) = expr_stmt.expression.without_parentheses() else {
        return false;
    };
    expression_static_name(&call.callee).is_some_and(|name| {
        name == "console.log" || name == "console.error" || name == "console.warn"
    })
}

fn member_root_identifier<'a>(member: &'a oxc_ast::ast::MemberExpression<'a>) -> Option<&'a str> {
    let mut object = member.object().without_parentheses();
    loop {
        match object {
            Expression::Identifier(identifier) => return Some(identifier.name.as_str()),
            _ => {
                let parent = object.get_member_expr()?;
                object = parent.object().without_parentheses();
            }
        }
    }
}

fn handler_has_status_call(ctx: &LintContext, handler_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(handler_span, call.span)
            && call
                .callee
                .get_member_expr()
                .and_then(|member| member.static_property_name())
                .is_some_and(|name| matches!(name, "status" | "sendStatus" | "code"))
    })
}

fn first_argument_expression<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a Expression<'a>> {
    call.arguments.first()?.as_expression()
}

fn object_has_error_payload(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.key.is_specific_static_name("error")
            || property.key.is_specific_static_name("message")
    })
}

fn is_string_throw(throw_stmt: &ThrowStatement<'_>) -> bool {
    match throw_stmt.argument.without_parentheses() {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("test.ts", source, &ProjectInfo::test_server())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_async_route_without_try() {
        let messages = rule_messages(
            "server/no-unhandled-async-route",
            "app.get('/users', async (req, res) => { await loadUsers(); res.json({ ok: true }); });\n",
        );
        assert_eq!(
            messages,
            vec!["Async route handlers should catch and translate errors explicitly"]
        );
    }

    #[test]
    fn flags_swallowed_error() {
        let messages = rule_messages(
            "server/no-swallowed-error",
            "try { run(); } catch (error) { console.error(error); }\n",
        );
        assert_eq!(
            messages,
            vec!["Catch blocks should handle, rethrow, or surface the error"]
        );
    }

    #[test]
    fn flags_process_exit_in_handler() {
        let messages = rule_messages(
            "server/no-process-exit-in-handler",
            "app.get('/kill', (req, res) => { process.exit(1); });\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid terminating the process from inside a request handler"]
        );
    }

    #[test]
    fn flags_missing_error_status() {
        let messages = rule_messages(
            "server/require-error-status",
            "app.get('/users', (req, res) => { res.json({ error: 'nope' }); });\n",
        );
        assert_eq!(
            messages,
            vec!["Set an HTTP error status before returning error payloads"]
        );
    }

    #[test]
    fn ignores_handlers_with_status_call() {
        let messages = rule_messages(
            "server/require-error-status",
            "app.get('/users', (req, res) => { res.status(400); res.json({ error: 'nope' }); });\n",
        );
        assert!(messages.is_empty());
    }

    #[test]
    fn flags_throw_string() {
        let messages = rule_messages("server/no-throw-string", "throw 'boom';\n");
        assert_eq!(
            messages,
            vec!["Throw `Error` objects instead of string literals"]
        );
    }
}
