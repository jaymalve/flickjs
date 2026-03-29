use oxc_ast::ast::{Expression, IdentifierReference, MemberExpression};

use super::{LintContext, LintDiagnostic, LintRule, Severity};

pub struct NoConsole;

impl LintRule for NoConsole {
    fn name(&self) -> &'static str {
        "no-console"
    }

    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic> {
        ctx.semantic
            .nodes()
            .iter()
            .filter_map(|node| {
                let call = match node.kind() {
                    oxc_ast::AstKind::CallExpression(call) => call,
                    _ => return None,
                };
                let member = call.callee.get_member_expr()?;
                let root = root_identifier(member)?;
                if root.name != "console" {
                    return None;
                }
                if is_shadowed_console(ctx, root) {
                    return None;
                }

                Some(ctx.diagnostic(
                    self.name(),
                    "Unexpected console statement",
                    call.span,
                    Severity::Warning,
                ))
            })
            .collect()
    }
}

fn root_identifier<'a>(member: &'a MemberExpression<'a>) -> Option<&'a IdentifierReference<'a>> {
    let object = member.object().without_parentheses();
    match object {
        Expression::Identifier(ident) => Some(ident),
        _ => object.get_member_expr().and_then(root_identifier),
    }
}

fn is_shadowed_console(ctx: &LintContext, ident: &IdentifierReference) -> bool {
    ident.reference_id
        .get()
        .and_then(|reference_id| ctx.semantic.scoping().get_reference(reference_id).symbol_id())
        .is_some()
}
