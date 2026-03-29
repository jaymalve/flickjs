use oxc_ast::AstKind;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct NoEmptyCatch;

impl LintRule for NoEmptyCatch {
    fn name(&self) -> &'static str {
        "no-empty-catch"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::CatchClause(clause) if clause.body.body.is_empty() => Some(ctx.diagnostic(
                    self.name(),
                    "Empty catch block — handle or rethrow the error",
                    clause.span,
                    Severity::Error,
                )),
                _ => None,
            })
            .collect()
    }
}
