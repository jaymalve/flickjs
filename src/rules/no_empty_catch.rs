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

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn empty_catch_spans(source: &str) -> Vec<String> {
        lint_source_for_test("test.js", source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "no-empty-catch")
            .map(|diagnostic| diagnostic.span)
            .collect()
    }

    #[test]
    fn flags_empty_catch_blocks() {
        let spans = empty_catch_spans("try { work(); } catch (error) {}\n");
        assert_eq!(spans, vec!["1:17"]);
    }

    #[test]
    fn ignores_non_empty_catch_blocks() {
        let spans = empty_catch_spans("try { work(); } catch (error) { recover(error); }\n");
        assert!(spans.is_empty());
    }
}
