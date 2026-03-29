use oxc_ast::AstKind;
use oxc_ast::ast::VariableDeclarationKind;

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
                if !declarator.id.is_binding_identifier() || is_loop_binding(ctx, node.id()) {
                    return None;
                }
                let binding = declarator.id.get_binding_identifier()?;
                let symbol_id = binding.symbol_id.get()?;
                if ctx.semantic.scoping().symbol_is_mutated(symbol_id) {
                    return None;
                }

                Some(ctx.diagnostic(
                    self.name(),
                    format!("`{}` is never reassigned; use `const` instead", binding.name),
                    binding.span,
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
