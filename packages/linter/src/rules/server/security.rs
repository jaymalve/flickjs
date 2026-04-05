use crate::project::ProjectInfo;
use crate::rules::{LintContext, LintDiagnostic, LintRule, Severity};
use oxc_ast::ast::{
    Argument, BinaryExpression, Expression, ObjectExpression, ObjectPropertyKind, TemplateLiteral,
};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

use super::helpers::{expression_static_name, is_inside_route_handler};

pub fn all_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(NoSqlInjection),
        Box::new(NoShellInjection),
        Box::new(NoPathTraversal),
        Box::new(NoUnsafeRedirect),
        Box::new(NoCorsWildcard),
        Box::new(NoHardcodedJwtSecret),
        Box::new(NoJwtNoneAlgorithm),
    ]
}

macro_rules! server_rule {
    ($name:ident, $rule_name:literal, $severity:expr, $run_fn:ident) => {
        pub struct $name;

        impl LintRule for $name {
            fn name(&self) -> &'static str {
                $rule_name
            }

            fn default_severity(&self) -> Severity {
                $severity
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
    NoSqlInjection,
    "server/no-sql-injection",
    Severity::Error,
    run_no_sql_injection
);
server_rule!(
    NoShellInjection,
    "server/no-shell-injection",
    Severity::Error,
    run_no_shell_injection
);
server_rule!(
    NoPathTraversal,
    "server/no-path-traversal",
    Severity::Error,
    run_no_path_traversal
);
server_rule!(
    NoUnsafeRedirect,
    "server/no-unsafe-redirect",
    Severity::Error,
    run_no_unsafe_redirect
);
server_rule!(
    NoCorsWildcard,
    "server/no-cors-wildcard",
    Severity::Error,
    run_no_cors_wildcard
);
server_rule!(
    NoHardcodedJwtSecret,
    "server/no-hardcoded-jwt-secret",
    Severity::Error,
    run_no_hardcoded_jwt_secret
);
server_rule!(
    NoJwtNoneAlgorithm,
    "server/no-jwt-none-algorithm",
    Severity::Warning,
    run_no_jwt_none_algorithm
);

fn run_no_sql_injection(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            is_sql_query_call(call)
                .then(|| first_argument_expression(call))
                .flatten()
                .filter(|expression| is_dynamic_string_expression(expression))
                .map(|expression| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid building SQL queries with dynamic string interpolation",
                        expression.span(),
                        Severity::Error,
                    )
                })
        })
        .collect()
}

fn run_no_shell_injection(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_shell_call(call) {
                return None;
            }
            dynamic_shell_argument(call).map(|expression| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid passing dynamic input into shell execution APIs",
                    expression.span(),
                    Severity::Error,
                )
            })
        })
        .collect()
}

fn run_no_path_traversal(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_path_sensitive_fs_call(call) {
                return None;
            }
            let expression = first_argument_expression(call)?;
            contains_request_data(expression).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid using request data directly in filesystem paths",
                    expression.span(),
                    Severity::Error,
                )
            })
        })
        .collect()
}

fn run_no_unsafe_redirect(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_redirect_call(call) || !is_inside_route_handler(ctx, node.id()) {
                return None;
            }
            let expression = first_argument_expression(call)?;
            contains_request_data(expression).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid redirecting to request-controlled URLs",
                    expression.span(),
                    Severity::Error,
                )
            })
        })
        .collect()
}

fn run_no_cors_wildcard(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_cors_call(call) {
                return None;
            }
            let Expression::ObjectExpression(object) = first_argument_expression(call)? else {
                return None;
            };
            let origin = object_property(object, "origin")?;
            let credentials = object_property(object, "credentials")?;
            (is_string_literal(origin, "*") && is_boolean_literal(credentials, true)).then(|| {
                ctx.diagnostic(
                    rule_name,
                    "Avoid `origin: \"*\"` when CORS credentials are enabled",
                    object.span,
                    Severity::Error,
                )
            })
        })
        .collect()
}

fn run_no_hardcoded_jwt_secret(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_jwt_sign_call(call) {
                return None;
            }
            let secret = argument_expression(call.arguments.get(1)?)?;
            string_like_literal(secret)
                .filter(|value| looks_like_secret_value(value))
                .map(|_| {
                    ctx.diagnostic(
                        rule_name,
                        "Avoid hardcoding JWT signing secrets",
                        secret.span(),
                        Severity::Error,
                    )
                })
        })
        .collect()
}

fn run_no_jwt_none_algorithm(ctx: &LintContext, rule_name: &'static str) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| {
            let AstKind::CallExpression(call) = node.kind() else {
                return None;
            };
            if !is_jwt_verify_call(call) || jwt_verify_has_algorithms(call) {
                return None;
            }
            Some(ctx.diagnostic(
                rule_name,
                "Specify allowed JWT verification algorithms explicitly",
                call.span,
                Severity::Warning,
            ))
        })
        .collect()
}

fn is_sql_query_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    call.callee
        .get_member_expr()
        .and_then(|member| member.static_property_name())
        == Some("query")
}

fn is_shell_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "exec"
                | "spawn"
                | "execSync"
                | "spawnSync"
                | "child_process.exec"
                | "child_process.spawn"
                | "child_process.execSync"
                | "child_process.spawnSync"
        )
    })
}

fn is_path_sensitive_fs_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        matches!(
            name.as_str(),
            "fs.readFile"
                | "fs.readFileSync"
                | "fs.open"
                | "fs.openSync"
                | "fs.createReadStream"
                | "fs.stat"
                | "fs.statSync"
                | "fs.access"
                | "fs.accessSync"
                | "fs.promises.readFile"
                | "fs.promises.open"
                | "fs.promises.stat"
                | "fs.promises.access"
        )
    })
}

fn is_redirect_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee).is_some_and(|name| {
        name.ends_with(".redirect") && (name.starts_with("res.") || name.starts_with("reply."))
    })
}

fn is_cors_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee)
        .is_some_and(|name| name == "cors" || name.ends_with(".cors"))
}

fn is_jwt_sign_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee)
        .is_some_and(|name| name == "jwt.sign" || name == "jsonwebtoken.sign")
}

fn is_jwt_verify_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    expression_static_name(&call.callee)
        .is_some_and(|name| name == "jwt.verify" || name == "jsonwebtoken.verify")
}

fn dynamic_shell_argument<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a Expression<'a>> {
    call.arguments
        .iter()
        .take(2)
        .filter_map(argument_expression)
        .find(|expression| is_dynamic_string_expression(expression))
}

fn first_argument_expression<'a>(
    call: &'a oxc_ast::ast::CallExpression<'a>,
) -> Option<&'a Expression<'a>> {
    call.arguments.first().and_then(argument_expression)
}

fn argument_expression<'a>(argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    match argument {
        Argument::SpreadElement(_) => None,
        argument => argument.as_expression(),
    }
}

fn object_property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        property
            .key
            .is_specific_static_name(name)
            .then_some(&property.value)
    })
}

fn is_boolean_literal(expression: &Expression<'_>, expected: bool) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::BooleanLiteral(literal) if literal.value == expected
    )
}

fn is_string_literal(expression: &Expression<'_>, expected: &str) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::StringLiteral(literal) if literal.value == expected
    )
}

fn string_like_literal(expression: &Expression<'_>) -> Option<String> {
    match expression.without_parentheses() {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => template_literal_value(template),
        _ => None,
    }
}

fn template_literal_value(template: &TemplateLiteral<'_>) -> Option<String> {
    template.single_quasi().map(|value| value.to_string())
}

fn looks_like_secret_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() < 8 {
        return false;
    }
    !trimmed.to_ascii_lowercase().contains("example")
}

fn is_dynamic_string_expression(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),
        Expression::BinaryExpression(binary) => is_dynamic_string_concat(binary),
        _ => false,
    }
}

fn is_dynamic_string_concat(binary: &BinaryExpression<'_>) -> bool {
    binary.operator == BinaryOperator::Addition
        && (contains_string_literal(&binary.left)
            || contains_string_literal(&binary.right)
            || contains_request_data(&binary.left)
            || contains_request_data(&binary.right))
}

fn contains_string_literal(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_) => true,
        Expression::BinaryExpression(binary) => {
            contains_string_literal(&binary.left) || contains_string_literal(&binary.right)
        }
        _ => false,
    }
}

fn contains_request_data(expression: &Expression<'_>) -> bool {
    let expression = expression.without_parentheses();
    if expression_static_name(expression).is_some_and(|name| {
        matches!(
            name.as_str(),
            "req.params"
                | "req.query"
                | "req.body"
                | "request.params"
                | "request.query"
                | "request.body"
        ) || name.starts_with("req.params.")
            || name.starts_with("req.query.")
            || name.starts_with("req.body.")
            || name.starts_with("request.params.")
            || name.starts_with("request.query.")
            || name.starts_with("request.body.")
    }) {
        return true;
    }

    match expression {
        Expression::TemplateLiteral(template) => {
            template.expressions.iter().any(contains_request_data)
        }
        Expression::BinaryExpression(binary) => {
            contains_request_data(&binary.left) || contains_request_data(&binary.right)
        }
        Expression::CallExpression(call) => {
            contains_request_data(&call.callee)
                || call
                    .arguments
                    .iter()
                    .filter_map(argument_expression)
                    .any(contains_request_data)
        }
        _ => false,
    }
}

fn jwt_verify_has_algorithms(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    call.arguments.iter().enumerate().any(|(index, argument)| {
        let Some(expression) = argument_expression(argument) else {
            return false;
        };
        match expression.without_parentheses() {
            Expression::ObjectExpression(object) => object_property(object, "algorithms").is_some(),
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => index > 2,
            _ => false,
        }
    })
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
    fn flags_sql_injection() {
        let messages = rule_messages(
            "server/no-sql-injection",
            "db.query(`SELECT * FROM users WHERE id = ${req.params.id}`);\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid building SQL queries with dynamic string interpolation"]
        );
    }

    #[test]
    fn flags_shell_injection() {
        let messages = rule_messages(
            "server/no-shell-injection",
            "exec(`git show ${req.query.ref}`);\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid passing dynamic input into shell execution APIs"]
        );
    }

    #[test]
    fn flags_path_traversal() {
        let messages = rule_messages(
            "server/no-path-traversal",
            "fs.readFile(req.params.file);\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid using request data directly in filesystem paths"]
        );
    }

    #[test]
    fn flags_unsafe_redirect_in_handler() {
        let messages = rule_messages(
            "server/no-unsafe-redirect",
            "app.get('/login', (req, res) => { res.redirect(req.query.url); });\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid redirecting to request-controlled URLs"]
        );
    }

    #[test]
    fn flags_cors_wildcard_with_credentials() {
        let messages = rule_messages(
            "server/no-cors-wildcard",
            "app.use(cors({ origin: \"*\", credentials: true }));\n",
        );
        assert_eq!(
            messages,
            vec!["Avoid `origin: \"*\"` when CORS credentials are enabled"]
        );
    }

    #[test]
    fn flags_hardcoded_jwt_secret() {
        let messages = rule_messages(
            "server/no-hardcoded-jwt-secret",
            "jwt.sign(payload, 'super-secret-value');\n",
        );
        assert_eq!(messages, vec!["Avoid hardcoding JWT signing secrets"]);
    }

    #[test]
    fn flags_missing_jwt_algorithms() {
        let messages = rule_messages(
            "server/no-jwt-none-algorithm",
            "jwt.verify(token, secret);\n",
        );
        assert_eq!(
            messages,
            vec!["Specify allowed JWT verification algorithms explicitly"]
        );
    }

    #[test]
    fn allows_explicit_jwt_algorithms() {
        let messages = rule_messages(
            "server/no-jwt-none-algorithm",
            "jwt.verify(token, secret, { algorithms: ['HS256'] });\n",
        );
        assert!(messages.is_empty());
    }
}
