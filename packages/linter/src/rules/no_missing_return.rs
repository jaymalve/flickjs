use oxc_ast::ast::Statement;
use oxc_ast::AstKind;
use oxc_span::GetSpan;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

/// Detects functions with an explicit return type annotation that don't
/// return a value on every code path. Only fires for TypeScript files.
pub struct NoMissingReturn;

impl LintRule for NoMissingReturn {
    fn name(&self) -> &'static str {
        "no-missing-return"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        // Only relevant for TypeScript files
        if !ctx.source_type.is_typescript() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        for node in ctx.semantic.nodes().iter() {
            let func = match node.kind() {
                AstKind::Function(f) => f,
                _ => continue,
            };

            // Only check functions with an explicit return type that isn't void/undefined/never
            let return_type = match &func.return_type {
                Some(rt) => rt,
                None => continue,
            };

            if is_void_or_never(&return_type.type_annotation) {
                continue;
            }

            // Check the function body
            let body = match &func.body {
                Some(b) => b,
                None => continue, // abstract or declaration — no body to check
            };

            if body.statements.is_empty() {
                // Empty body with non-void return type
                diagnostics.push(ctx.diagnostic_with_context(
                    self.name(),
                    format!(
                        "Function{} has return type annotation but body is empty",
                        func_name_suffix(func),
                    ),
                    func.span(),
                    Severity::Error,
                    super::RuleOrigin::BuiltIn,
                    Some("Function".to_string()),
                    func.id.as_ref().map(|id| id.name.to_string()),
                ));
                continue;
            }

            if !all_paths_return(&body.statements) {
                diagnostics.push(ctx.diagnostic_with_context(
                    self.name(),
                    format!(
                        "Function{} has return type annotation but not all code paths return a value",
                        func_name_suffix(func),
                    ),
                    func.span(),
                    Severity::Error,
                    super::RuleOrigin::BuiltIn,
                    Some("Function".to_string()),
                    func.id.as_ref().map(|id| id.name.to_string()),
                ));
            }
        }

        diagnostics
    }
}

fn func_name_suffix(func: &oxc_ast::ast::Function) -> String {
    match &func.id {
        Some(id) => format!(" `{}`", id.name),
        None => String::new(),
    }
}

fn is_void_or_never(ts_type: &oxc_ast::ast::TSType) -> bool {
    use oxc_ast::ast::TSType;
    match ts_type {
        TSType::TSVoidKeyword(_) => true,
        TSType::TSNeverKeyword(_) => true,
        TSType::TSUndefinedKeyword(_) => true,
        // Handle Promise<void>, Promise<never>, Promise<undefined>
        TSType::TSTypeReference(type_ref) => {
            if let oxc_ast::ast::TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                if id.name == "Promise" {
                    if let Some(args) = &type_ref.type_arguments {
                        return args.params.len() == 1 && is_void_or_never(&args.params[0]);
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Check if all code paths in a statement list end with a return/throw.
/// This is a conservative check — it may produce false positives for
/// complex control flow, but won't miss genuinely missing returns.
fn all_paths_return(stmts: &[Statement<'_>]) -> bool {
    if stmts.is_empty() {
        return false;
    }

    let last = &stmts[stmts.len() - 1];
    stmt_always_returns(last)
}

fn stmt_always_returns(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,

        Statement::IfStatement(if_stmt) => {
            // Both branches must return
            let then_returns = stmt_always_returns(&if_stmt.consequent);
            let else_returns = if_stmt
                .alternate
                .as_ref()
                .map(|s| stmt_always_returns(s))
                .unwrap_or(false);
            then_returns && else_returns
        }

        Statement::BlockStatement(block) => all_paths_return(&block.body),

        Statement::SwitchStatement(switch) => {
            // Every case (including default) must return
            if switch.cases.is_empty() {
                return false;
            }
            let has_default = switch.cases.iter().any(|c| c.test.is_none());
            if !has_default {
                return false;
            }
            // Check each case, but allow fall-through: a case with no
            // statements falls through to the next case, so only the
            // case that actually has statements needs to return.
            switch.cases.iter().all(|c| {
                c.consequent.is_empty() || all_paths_return(&c.consequent)
            })
        }

        Statement::TryStatement(try_stmt) => {
            let try_returns = all_paths_return(&try_stmt.block.body);
            let catch_returns = try_stmt
                .handler
                .as_ref()
                .map(|h| all_paths_return(&h.body.body))
                .unwrap_or(true); // No catch = try must return
            // If finally exists and returns, the whole thing returns
            let finally_returns = try_stmt
                .finalizer
                .as_ref()
                .map(|f| all_paths_return(&f.body))
                .unwrap_or(false);
            finally_returns || (try_returns && catch_returns)
        }

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn missing_return_spans(source: &str) -> Vec<String> {
        lint_source_for_test("test.ts", source)
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "no-missing-return")
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn flags_missing_return_in_simple_function() {
        let msgs = missing_return_spans(
            "function foo(): number {\n  const x = 1;\n}\n",
        );
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("not all code paths return"));
    }

    #[test]
    fn flags_empty_body_with_return_type() {
        let msgs = missing_return_spans("function foo(): string {}\n");
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].contains("body is empty"));
    }

    #[test]
    fn ok_when_all_paths_return() {
        let msgs = missing_return_spans(
            "function foo(x: boolean): number {\n  if (x) {\n    return 1;\n  } else {\n    return 2;\n  }\n}\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn flags_if_without_else() {
        let msgs = missing_return_spans(
            "function foo(x: boolean): number {\n  if (x) {\n    return 1;\n  }\n}\n",
        );
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn ok_for_void_return() {
        let msgs = missing_return_spans("function foo(): void {\n  console.log('hi');\n}\n");
        assert!(msgs.is_empty());
    }

    #[test]
    fn ok_for_simple_return() {
        let msgs = missing_return_spans(
            "function foo(): number {\n  return 42;\n}\n",
        );
        assert!(msgs.is_empty());
    }

    #[test]
    fn ignores_js_files() {
        let result = crate::rules::lint_source_for_test("test.js", "function foo() { const x = 1; }\n");
        let msgs: Vec<_> = result
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "no-missing-return")
            .collect();
        assert!(msgs.is_empty());
    }

    #[test]
    fn ok_for_promise_void_return() {
        let msgs = missing_return_spans(
            r#"
            async function unlinkWallet(addr: string): Promise<void> {
                try {
                    const r = await fetch("/api");
                    if (!r.ok) { throw new Error("fail"); }
                } catch (e) {
                    throw e;
                }
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_for_promise_never_return() {
        let msgs = missing_return_spans(
            r#"
            async function fail(): Promise<never> {
                throw new Error("always fails");
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_switch_fallthrough() {
        // Fall-through cases (empty consequent) should not be flagged
        let msgs = missing_return_spans(
            r#"
            function foo(x: string): number {
                switch (x) {
                    case 'a':
                    case 'b':
                        return 1;
                    case 'c':
                        return 2;
                    default:
                        return 3;
                }
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_switch_default_throw() {
        // default with throw counts as returning
        let msgs = missing_return_spans(
            r#"
            function foo(x: string): number {
                switch (x) {
                    case 'a':
                        return 1;
                    default:
                        throw new Error("unknown");
                }
            }
            "#,
        );
        assert!(msgs.is_empty(), "Expected no diagnostics but got: {:?}", msgs);
    }

    #[test]
    fn ok_with_try_catch_return() {
        let msgs = missing_return_spans(
            "function foo(): number {\n  try {\n    return 1;\n  } catch (e) {\n    return 2;\n  }\n}\n",
        );
        assert!(msgs.is_empty());
    }
}
