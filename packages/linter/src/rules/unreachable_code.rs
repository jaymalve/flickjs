use oxc_ast::ast::Statement;
use oxc_ast::AstKind;
use oxc_span::GetSpan;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct UnreachableCode;

impl LintRule for UnreachableCode {
    fn name(&self) -> &'static str {
        "unreachable-code"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        let mut diagnostics = Vec::new();

        for node in ctx.semantic.nodes().iter() {
            let block_body = match node.kind() {
                AstKind::FunctionBody(body) => &body.statements,
                AstKind::BlockStatement(block) => &block.body,
                AstKind::SwitchCase(case) => &case.consequent,
                _ => continue,
            };

            let mut found_terminator = false;
            for stmt in block_body.iter() {
                if found_terminator {
                    diagnostics.push(ctx.diagnostic_with_context(
                        self.name(),
                        "Unreachable code after return/throw/break/continue",
                        stmt.span(),
                        Severity::Error,
                        super::RuleOrigin::BuiltIn,
                        Some(ast_kind_name(stmt)),
                        None,
                    ));
                    break; // Only report the first unreachable statement per block
                }

                if is_terminator(stmt) {
                    found_terminator = true;
                }
            }
        }

        diagnostics
    }
}

fn is_terminator(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_) => true,

        // if/else is terminating when both branches terminate
        Statement::IfStatement(if_stmt) => {
            let then_terminates = is_terminator(&if_stmt.consequent);
            let else_terminates = if_stmt
                .alternate
                .as_ref()
                .map(|s| is_terminator(s))
                .unwrap_or(false); // no else = control falls through
            then_terminates && else_terminates
        }

        // block is terminating if its last statement terminates
        Statement::BlockStatement(block) => {
            block.body.last().map(|s| is_terminator(s)).unwrap_or(false)
        }

        _ => false,
    }
}

fn ast_kind_name(stmt: &Statement<'_>) -> String {
    match stmt {
        Statement::VariableDeclaration(_) => "VariableDeclaration",
        Statement::ExpressionStatement(_) => "ExpressionStatement",
        Statement::ReturnStatement(_) => "ReturnStatement",
        Statement::IfStatement(_) => "IfStatement",
        Statement::ForStatement(_) => "ForStatement",
        Statement::WhileStatement(_) => "WhileStatement",
        Statement::BlockStatement(_) => "BlockStatement",
        Statement::FunctionDeclaration(_) => "FunctionDeclaration",
        _ => "Statement",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn unreachable_spans(source: &str) -> Vec<String> {
        lint_source_for_test("test.js", source)
            .diagnostics
            .into_iter()
            .filter(|d| d.rule_name == "unreachable-code")
            .map(|d| d.span)
            .collect()
    }

    #[test]
    fn flags_code_after_return() {
        let spans = unreachable_spans("function foo() {\n  return 1;\n  const x = 2;\n}\n");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], "3:3");
    }

    #[test]
    fn flags_code_after_throw() {
        let spans = unreachable_spans(
            "function foo() {\n  throw new Error('fail');\n  console.log('never');\n}\n",
        );
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], "3:3");
    }

    #[test]
    fn ignores_code_with_no_terminator() {
        let spans = unreachable_spans("function foo() {\n  const x = 1;\n  return x;\n}\n");
        assert!(spans.is_empty());
    }

    #[test]
    fn flags_code_after_break_in_loop() {
        let spans =
            unreachable_spans("for (let i = 0; i < 10; i++) {\n  break;\n  console.log(i);\n}\n");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn only_reports_first_unreachable_per_block() {
        let spans =
            unreachable_spans("function foo() {\n  return 1;\n  const a = 1;\n  const b = 2;\n}\n");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn flags_code_after_exhaustive_if_else_return() {
        let spans = unreachable_spans(
            r#"function foo(x) {
  if (x > 0) {
    return true;
  } else {
    return false;
  }
  return false;
}
"#,
        );
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn no_flag_if_without_else() {
        let spans = unreachable_spans(
            r#"function foo(x) {
  if (x > 0) {
    return true;
  }
  return false;
}
"#,
        );
        assert!(spans.is_empty());
    }

    #[test]
    fn flags_code_after_nested_exhaustive_if_else() {
        let spans = unreachable_spans(
            r#"function foo(x) {
  if (x > 0) {
    if (x > 10) {
      return "big";
    } else {
      return "small";
    }
  } else {
    return "negative";
  }
  console.log("unreachable");
}
"#,
        );
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn no_flag_nested_if_missing_else() {
        let spans = unreachable_spans(
            r#"function foo(x) {
  if (x > 0) {
    if (x > 10) {
      return "big";
    }
  } else {
    return "negative";
  }
  console.log("reachable");
}
"#,
        );
        assert!(spans.is_empty());
    }
}
