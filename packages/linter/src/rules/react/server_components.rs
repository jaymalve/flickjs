use crate::project::ProjectInfo;
use crate::rules::server::helpers::{
    expression_static_name as server_expression_static_name, is_orm_mutate_call,
};
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast::AstKind;
use oxc_span::Span;

use super::helpers::{file_has_directive, span_contains};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(ServerAuthActions),
        Box::new(ServerAfterNonblocking),
    ]
}

macro_rules! server_component_rule {
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
                if !ctx.project.has_next || !file_has_directive(ctx, "use server") {
                    return Vec::new();
                }
                $run_fn(ctx, self.name())
            }
        }
    };
}

server_component_rule!(
    ServerAuthActions,
    "react/server-auth-actions",
    run_server_auth_actions
);
server_component_rule!(
    ServerAfterNonblocking,
    "react/server-after-nonblocking",
    run_server_after_nonblocking
);

fn run_server_auth_actions(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    exported_function_spans(ctx)
        .into_iter()
        .filter_map(|function| {
            (!function_has_auth_check(ctx, function.body_span)
                && (function_has_server_mutation(ctx, function.body_span)
                    || function.has_action_params))
                .then(|| {
                    ctx.diagnostic(
                        rule_name,
                        format!(
                            "Server action `{}` should perform an auth check before mutating data",
                            function.name
                        ),
                        function.name_span,
                        Severity::Warning,
                    )
                })
        })
        .collect()
}

fn run_server_after_nonblocking(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    exported_function_spans(ctx)
        .into_iter()
        .filter_map(|function| {
            (function_has_nonblocking_side_effect(ctx, function.body_span)
                && !function_has_after_call(ctx, function.body_span))
            .then(|| {
                ctx.diagnostic(
                    rule_name,
                    format!(
                        "Wrap non-blocking side effects in `after()` inside server action `{}`",
                        function.name
                    ),
                    function.name_span,
                    Severity::Warning,
                )
            })
        })
        .collect()
}

struct ExportedFunction {
    name: String,
    name_span: Span,
    body_span: Span,
    has_action_params: bool,
}

fn exported_function_spans(ctx: &LintContext) -> Vec<ExportedFunction> {
    let mut functions = Vec::new();

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
                functions.push(ExportedFunction {
                    name: identifier.name.to_string(),
                    name_span: identifier.span,
                    body_span: body.span,
                    has_action_params: params_look_like_action(&function.params),
                });
            }
            oxc_ast::ast::Declaration::VariableDeclaration(decl) => {
                for declarator in &decl.declarations {
                    let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                        continue;
                    };
                    let Some(init) = &declarator.init else {
                        continue;
                    };
                    match init.without_parentheses() {
                        Expression::ArrowFunctionExpression(function) => {
                            functions.push(ExportedFunction {
                                name: identifier.name.to_string(),
                                name_span: identifier.span,
                                body_span: function.body.span,
                                has_action_params: params_look_like_action(&function.params),
                            });
                        }
                        Expression::FunctionExpression(function) => {
                            let Some(body) = &function.body else {
                                continue;
                            };
                            functions.push(ExportedFunction {
                                name: identifier.name.to_string(),
                                name_span: identifier.span,
                                body_span: body.span,
                                has_action_params: params_look_like_action(&function.params),
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    functions
}

fn params_look_like_action(params: &oxc_ast::ast::FormalParameters<'_>) -> bool {
    params.items.iter().any(|param| {
        param
            .pattern
            .get_identifier_name()
            .is_some_and(|name| matches!(name.as_str(), "formData" | "input" | "data" | "payload"))
    })
}

fn function_has_auth_check(ctx: &LintContext, body_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(body_span, call.span)
            && server_expression_static_name(&call.callee).is_some_and(|name| {
                matches!(
                    name.as_str(),
                    "auth"
                        | "auth.requireUser"
                        | "requireAuth"
                        | "requireUser"
                        | "getServerSession"
                        | "assertUser"
                        | "verifySession"
                )
            })
    })
}

fn function_has_server_mutation(ctx: &LintContext, body_span: Span) -> bool {
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
                    "revalidatePath" | "revalidateTag" | "redirect"
                )
            })
    })
}

fn function_has_nonblocking_side_effect(ctx: &LintContext, body_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(body_span, call.span)
            && server_expression_static_name(&call.callee).is_some_and(|name| {
                matches!(
                    name.as_str(),
                    "analytics.track"
                        | "analytics.identify"
                        | "posthog.capture"
                        | "captureException"
                        | "sentry.captureException"
                        | "logEvent"
                        | "console.log"
                )
            })
    })
}

fn function_has_after_call(ctx: &LintContext, body_span: Span) -> bool {
    ctx.semantic.nodes().iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        span_contains(body_span, call.span)
            && server_expression_static_name(&call.callee).is_some_and(|name| name == "after")
    })
}

#[cfg(test)]
mod tests {
    use crate::project::ProjectInfo;
    use crate::rules::lint_source_for_test_with_project;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test_with_project("app/actions.ts", source, &ProjectInfo::test_all())
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_server_action_without_auth() {
        let messages = rule_messages(
            "react/server-auth-actions",
            "\"use server\";\nexport async function saveUser(formData) { await prisma.user.create({ data: {} }); }\n",
        );
        assert_eq!(
            messages,
            vec!["Server action `saveUser` should perform an auth check before mutating data"]
        );
    }

    #[test]
    fn flags_nonblocking_side_effect_without_after() {
        let messages = rule_messages(
            "react/server-after-nonblocking",
            "\"use server\";\nexport async function submit() { analytics.track('submitted'); }\n",
        );
        assert_eq!(
            messages,
            vec!["Wrap non-blocking side effects in `after()` inside server action `submit`"]
        );
    }
}
