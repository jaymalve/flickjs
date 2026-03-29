use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;
use oxc_span::GetSpan;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct PreferConst;

impl LintRule for PreferConst {
    fn name(&self) -> &'static str {
        "prefer-const"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| {
                let declarator = match node.kind() {
                    AstKind::VariableDeclarator(declarator) => declarator,
                    _ => return None,
                };
                if declarator.kind != VariableDeclarationKind::Let || declarator.init.is_none() {
                    return None;
                }
                if is_loop_binding(ctx, node.id()) {
                    return None;
                }
                let bindings = declarator.id.get_binding_identifiers();
                if bindings.is_empty() {
                    return None;
                }
                if bindings.iter().any(|binding| {
                    binding
                        .symbol_id
                        .get()
                        .is_none_or(|symbol_id| ctx.semantic.scoping().symbol_is_mutated(symbol_id))
                }) {
                    return None;
                }
                let message = if bindings.len() == 1 {
                    format!(
                        "`{}` is never reassigned; use `const` instead",
                        bindings[0].name
                    )
                } else {
                    let names = bindings
                        .iter()
                        .map(|binding| binding.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "Bindings `{names}` are never reassigned; use `const` instead"
                    )
                };

                Some(ctx.diagnostic(
                    self.name(),
                    message,
                    declarator.id.span(),
                    Severity::Warning,
                ))
            })
            .collect()
    }
}

fn is_loop_binding(ctx: &LintContext, declarator_id: oxc_syntax::node::NodeId) -> bool {
    let nodes = ctx.semantic.nodes();
    let declaration_id = nodes.parent_id(declarator_id);
    matches!(
        nodes.parent_kind(declaration_id),
        AstKind::ForStatement(_) | AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)
    )
}

#[cfg(test)]
mod tests {
    use crate::rules::lint_source_for_test;

    fn prefer_const_messages(source: &str) -> Vec<String> {
        lint_source_for_test("test.js", source)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "prefer-const")
            .map(|diagnostic| diagnostic.message)
            .collect()
    }

    #[test]
    fn flags_simple_let_without_reassignment() {
        let messages = prefer_const_messages("let count = 1;\nconsole.log(count);\n");
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("count"));
    }

    #[test]
    fn flags_destructured_let_without_reassignment() {
        let messages = prefer_const_messages("let { count, total } = stats;\nuse(count, total);\n");
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("count, total"));
    }

    #[test]
    fn ignores_reassigned_bindings() {
        let messages = prefer_const_messages("let count = 1;\ncount += 1;\n");
        assert!(messages.is_empty());
    }

    #[test]
    fn ignores_loop_bindings() {
        let messages = prefer_const_messages("for (let i = 0; i < 3; i++) {\nconsole.log(i);\n}\n");
        assert!(messages.is_empty());
    }
}
