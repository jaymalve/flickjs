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
