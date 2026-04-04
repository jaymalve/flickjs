use oxc_ast::ast::{
    BindingPattern, Expression, IdentifierReference, NewExpression, TemplateLiteral,
};
use oxc_ast::AstKind;
use oxc_span::GetSpan;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct NoEval;
pub struct NoHardcodedSecrets;

impl LintRule for NoEval {
    fn name(&self) -> &'static str {
        "no-eval"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::CallExpression(call) => {
                    if is_unshadowed_global_call(ctx, &call.callee, "eval") {
                        return Some(ctx.diagnostic(
                            self.name(),
                            "Avoid dynamic code execution with eval",
                            call.span,
                            Severity::Error,
                        ));
                    }

                    let Some(member) = call.callee.get_member_expr() else {
                        return string_timer_diagnostic(
                            ctx,
                            self.name(),
                            &call.callee,
                            &call.arguments,
                        );
                    };

                    if member.object().is_specific_id("window")
                        && member.static_property_name() == Some("eval")
                    {
                        return Some(ctx.diagnostic(
                            self.name(),
                            "Avoid dynamic code execution with eval",
                            call.span,
                            Severity::Error,
                        ));
                    }

                    string_timer_diagnostic(ctx, self.name(), &call.callee, &call.arguments)
                }
                AstKind::NewExpression(new_expr) => {
                    new_function_diagnostic(ctx, self.name(), new_expr)
                }
                _ => None,
            })
            .collect()
    }
}

impl LintRule for NoHardcodedSecrets {
    fn name(&self) -> &'static str {
        "no-hardcoded-secrets"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| {
                let decl = match node.kind() {
                    AstKind::VariableDeclarator(decl) => decl,
                    _ => return None,
                };
                let BindingPattern::BindingIdentifier(identifier) = &decl.id else {
                    return None;
                };
                if !looks_like_secret_name(identifier.name.as_str()) {
                    return None;
                }
                let init = decl.init.as_ref()?;
                let value = string_like_literal(init)?;
                if !looks_like_secret_value(&value) {
                    return None;
                }

                Some(ctx.diagnostic(
                    self.name(),
                    format!(
                        "`{}` appears to contain a hardcoded secret",
                        identifier.name
                    ),
                    decl.span,
                    Severity::Warning,
                ))
            })
            .collect()
    }
}

fn string_timer_diagnostic<'a>(
    ctx: &LintContext,
    rule_name: &'static str,
    callee: &Expression<'a>,
    arguments: &[oxc_ast::ast::Argument<'a>],
) -> Option<LintDiagnostic> {
    let Expression::Identifier(identifier) = callee.without_parentheses() else {
        return None;
    };
    if !is_unshadowed_global_identifier(ctx, identifier, "setTimeout")
        && !is_unshadowed_global_identifier(ctx, identifier, "setInterval")
    {
        return None;
    }
    let first_arg = arguments.first()?;
    string_like_argument(first_arg)?;
    Some(ctx.diagnostic(
        rule_name,
        "Avoid string-based timers that behave like eval",
        first_arg.span(),
        Severity::Error,
    ))
}

fn new_function_diagnostic(
    ctx: &LintContext,
    rule_name: &'static str,
    new_expr: &NewExpression<'_>,
) -> Option<LintDiagnostic> {
    let Expression::Identifier(identifier) = new_expr.callee.without_parentheses() else {
        return None;
    };
    if !is_unshadowed_global_identifier(ctx, identifier, "Function") {
        return None;
    }
    Some(ctx.diagnostic(
        rule_name,
        "Avoid dynamic code execution with the Function constructor",
        new_expr.span,
        Severity::Error,
    ))
}

fn is_unshadowed_global_call(ctx: &LintContext, callee: &Expression<'_>, name: &str) -> bool {
    match callee.without_parentheses() {
        Expression::Identifier(identifier) => {
            is_unshadowed_global_identifier(ctx, identifier, name)
        }
        _ => false,
    }
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

fn string_like_literal(expression: &Expression<'_>) -> Option<String> {
    match expression.without_parentheses() {
        Expression::StringLiteral(lit) => Some(lit.value.to_string()),
        Expression::TemplateLiteral(template) => template_literal_value(template),
        _ => None,
    }
}

fn string_like_argument(argument: &oxc_ast::ast::Argument<'_>) -> Option<String> {
    match argument {
        oxc_ast::ast::Argument::StringLiteral(lit) => Some(lit.value.to_string()),
        oxc_ast::ast::Argument::TemplateLiteral(template) => template_literal_value(template),
        _ => None,
    }
}

fn template_literal_value(template: &TemplateLiteral<'_>) -> Option<String> {
    template.single_quasi().map(|value| value.to_string())
}

fn looks_like_secret_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "passwd",
        "api_key",
        "apikey",
        "client_secret",
        "private_key",
        "jwt",
    ]
    .iter()
    .any(|pattern| name.contains(pattern))
}

fn looks_like_secret_value(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() < 8 {
        return false;
    }
    let lowercase = trimmed.to_ascii_lowercase();
    if lowercase.starts_with("http://")
        || lowercase.starts_with("https://")
        || lowercase.contains("example")
        || lowercase.contains("localhost")
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn rule_messages(rule_name: &str, source: &str) -> Vec<String> {
        lint_source_for_test("test.ts", source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == rule_name)
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_eval_calls() {
        let messages = rule_messages("no-eval", "eval(code);\n");
        assert_eq!(messages, vec!["Avoid dynamic code execution with eval"]);
    }

    #[test]
    fn flags_function_constructor() {
        let messages = rule_messages("no-eval", "new Function('return 1');\n");
        assert_eq!(
            messages,
            vec!["Avoid dynamic code execution with the Function constructor"]
        );
    }

    #[test]
    fn flags_string_timers() {
        let messages = rule_messages("no-eval", "setTimeout('work()', 100);\n");
        assert_eq!(
            messages,
            vec!["Avoid string-based timers that behave like eval"]
        );
    }

    #[test]
    fn ignores_shadowed_eval() {
        let messages = rule_messages(
            "no-eval",
            "function run(eval: (x: string) => void) { eval('x'); }\n",
        );
        assert!(messages.is_empty());
    }

    #[test]
    fn flags_hardcoded_secret_variable() {
        let messages = rule_messages(
            "no-hardcoded-secrets",
            "const apiKey = 'sk_live_super_secret';\n",
        );
        assert_eq!(
            messages,
            vec!["`apiKey` appears to contain a hardcoded secret"]
        );
    }

    #[test]
    fn ignores_short_or_example_secret_values() {
        let messages = rule_messages(
            "no-hardcoded-secrets",
            "const token = 'short';\nconst clientSecret = 'example-secret-value';\n",
        );
        assert!(messages.is_empty());
    }
}
