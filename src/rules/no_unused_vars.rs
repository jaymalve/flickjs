use oxc_ast::AstKind;

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct NoUnusedVars;

impl LintRule for NoUnusedVars {
    fn name(&self) -> &'static str {
        "no-unused-vars"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .scoping()
            .symbol_ids()
            .filter_map(|symbol_id| {
                let scoping = ctx.semantic.scoping();
                if !scoping.symbol_is_unused(symbol_id) {
                    return None;
                }
                let name = scoping.symbol_name(symbol_id);
                if name.starts_with('_') {
                    return None;
                }
                let declaration_id = scoping.symbol_declaration(symbol_id);
                if !is_supported_unused_var_declaration(ctx, declaration_id) {
                    return None;
                }

                Some(ctx.diagnostic(
                    self.name(),
                    format!("`{name}` is declared but never used"),
                    scoping.symbol_span(symbol_id),
                    Severity::Error,
                ))
            })
            .collect()
    }
}

fn is_supported_unused_var_declaration(
    ctx: &LintContext,
    declaration_id: oxc_syntax::node::NodeId,
) -> bool {
    matches!(
        ctx.semantic.nodes().parent_kind(declaration_id),
        AstKind::VariableDeclarator(_)
            | AstKind::FormalParameter(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_)
    )
}
