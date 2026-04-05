use oxc_ast::AstKind;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

/// Flags usage of `any` type annotation in TypeScript.
///
/// Bad:  `let x: any = 5;`
/// Good: `let x: unknown = 5;`
pub struct NoExplicitAny;

impl LintRule for NoExplicitAny {
    fn name(&self) -> &'static str {
        "no-explicit-any"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::TSAnyKeyword(keyword) => Some(ctx.diagnostic(
                    self.name(),
                    "Unexpected `any` type — use `unknown` for type-safe alternative",
                    keyword.span,
                    Severity::Warning,
                )),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn explicit_any_spans(source: &str) -> Vec<String> {
        lint_source_for_test("test.ts", source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "no-explicit-any")
            .map(|diagnostic| diagnostic.span)
            .collect()
    }

    #[test]
    fn flags_any_in_multiple_type_positions() {
        let spans = explicit_any_spans(
            "type Box = any;\nfunction wrap(value: any): any { return value as any; }\n",
        );
        assert_eq!(spans, vec!["1:12", "2:22", "2:28", "2:50"]);
    }

    #[test]
    fn ignores_unknown_and_plain_identifiers() {
        let spans = explicit_any_spans(
            "const any = 'name only';\nfunction wrap(value: unknown): unknown { return value; }\n",
        );
        assert!(spans.is_empty());
    }
}
