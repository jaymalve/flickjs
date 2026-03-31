use oxc_ast::AstKind;
use oxc_span::GetSpan;
use std::collections::HashMap;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

/// Detects call expressions where the number of arguments doesn't match
/// the function declaration's parameter count (same-file only).
pub struct NoWrongArgCount;

impl LintRule for NoWrongArgCount {
    fn name(&self) -> &'static str {
        "no-wrong-arg-count"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        let mut diagnostics = Vec::new();

        // Phase 1: Collect all function declarations and their parameter info
        let mut func_params: HashMap<String, FuncSignature> = HashMap::new();

        for node in ctx.semantic.nodes().iter() {
            if let AstKind::Function(func) = node.kind() {
                if let Some(id) = &func.id {
                    let name = id.name.to_string();
                    let required = func
                        .params
                        .items
                        .iter()
                        .filter(|p| {
                            !p.optional
                                && p.initializer.is_none()
                                && !matches!(
                                    p.pattern,
                                    oxc_ast::ast::BindingPattern::AssignmentPattern(_)
                                )
                        })
                        .count();
                    let total = func.params.items.len();
                    let has_rest = func.params.rest.is_some();

                    func_params.insert(
                        name,
                        FuncSignature {
                            required,
                            total,
                            has_rest,
                        },
                    );
                }
            }
        }

        if func_params.is_empty() {
            return diagnostics;
        }

        // Phase 2: Check all call expressions
        for node in ctx.semantic.nodes().iter() {
            if let AstKind::CallExpression(call) = node.kind() {
                // Get the callee name (only simple identifiers for now)
                let callee_name = match &call.callee {
                    oxc_ast::ast::Expression::Identifier(id) => id.name.to_string(),
                    _ => continue,
                };

                if let Some(sig) = func_params.get(&callee_name) {
                    let arg_count = call.arguments.len();

                    if arg_count < sig.required {
                        diagnostics.push(ctx.diagnostic_with_context(
                            self.name(),
                            format!(
                                "Function `{}` expects at least {} argument(s), but was called with {}",
                                callee_name, sig.required, arg_count
                            ),
                            call.span(),
                            Severity::Error,
                            super::RuleOrigin::BuiltIn,
                            Some("CallExpression".to_string()),
                            Some(callee_name),
                        ));
                    } else if !sig.has_rest && arg_count > sig.total {
                        diagnostics.push(ctx.diagnostic_with_context(
                            self.name(),
                            format!(
                                "Function `{}` expects at most {} argument(s), but was called with {}",
                                callee_name, sig.total, arg_count
                            ),
                            call.span(),
                            Severity::Error,
                            super::RuleOrigin::BuiltIn,
                            Some("CallExpression".to_string()),
                            Some(callee_name),
                        ));
                    }
                }
            }
        }

        diagnostics
    }
}

struct FuncSignature {
    required: usize,
    total: usize,
    has_rest: bool,
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn wrong_arg_msgs(source: &str) -> Vec<String> {
        lint_source_for_test("test.js", source)
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "no-wrong-arg-count")
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn flags_too_few_args() {
        let msgs = wrong_arg_msgs(
            "function add(a, b) { return a + b; }\nadd(1);\n",
        );
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("at least 2"));
    }

    #[test]
    fn flags_too_many_args() {
        let msgs = wrong_arg_msgs(
            "function add(a, b) { return a + b; }\nadd(1, 2, 3);\n",
        );
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("at most 2"));
    }

    #[test]
    fn ok_with_correct_args() {
        let msgs = wrong_arg_msgs(
            "function add(a, b) { return a + b; }\nadd(1, 2);\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn ok_with_optional_params() {
        let msgs = wrong_arg_msgs(
            "function greet(name, greeting = 'Hello') { return greeting + ' ' + name; }\ngreet('Jay');\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn ok_with_rest_params() {
        let msgs = wrong_arg_msgs(
            "function sum(...nums) { return nums.reduce((a, b) => a + b, 0); }\nsum(1, 2, 3, 4, 5);\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn ignores_unknown_functions() {
        let msgs = wrong_arg_msgs("unknownFunc(1, 2, 3);\n");
        assert!(msgs.is_empty());
    }
}
