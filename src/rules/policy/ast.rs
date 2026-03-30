use crate::rules::policy_ir::{AstRule, CompiledPolicyRule, ForbiddenSyntaxKind};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_ast::AstKind;
use oxc_span::GetSpan;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &AstRule,
) -> Vec<LintDiagnostic> {
    match rule {
        AstRule::MaxFunctionParams { max, .. } => ctx
            .semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::Function(function) if function.body.is_some() => {
                    let count = function.params.items.len() + usize::from(function.params.rest.is_some());
                    (count > *max).then(|| {
                        let span = function
                            .id
                            .as_ref()
                            .map(|identifier| identifier.span)
                            .unwrap_or_else(|| function.span());
                        ctx.diagnostic_with_origin(
                            compiled_rule.id.clone(),
                            format!("Function has {count} parameters; maximum allowed is {max}"),
                            span,
                            compiled_rule.severity.clone(),
                            RuleOrigin::English,
                        )
                    })
                }
                AstKind::ArrowFunctionExpression(function) => {
                    let count = function.params.items.len() + usize::from(function.params.rest.is_some());
                    (count > *max).then(|| {
                        ctx.diagnostic_with_origin(
                            compiled_rule.id.clone(),
                            format!("Function has {count} parameters; maximum allowed is {max}"),
                            function.span(),
                            compiled_rule.severity.clone(),
                            RuleOrigin::English,
                        )
                    })
                }
                _ => None,
            })
            .collect(),
        AstRule::ForbiddenSyntax { syntax, .. } => ctx
            .semantic
            .nodes()
            .iter()
            .filter_map(|node| syntax_match(ctx, node.id(), node.kind(), syntax).then(|| node.kind().span()))
            .map(|span| {
                ctx.diagnostic_with_origin(
                    compiled_rule.id.clone(),
                    compiled_rule.message.clone(),
                    span,
                    compiled_rule.severity.clone(),
                    RuleOrigin::English,
                )
            })
            .collect(),
    }
}

fn syntax_match(
    ctx: &LintContext,
    node_id: oxc_syntax::node::NodeId,
    kind: AstKind<'_>,
    syntax: &ForbiddenSyntaxKind,
) -> bool {
    match syntax {
        ForbiddenSyntaxKind::TryCatch => matches!(kind, AstKind::TryStatement(_)),
        ForbiddenSyntaxKind::Switch => matches!(kind, AstKind::SwitchStatement(_)),
        ForbiddenSyntaxKind::DefaultExport => matches!(kind, AstKind::ExportDefaultDeclaration(_)),
        ForbiddenSyntaxKind::Debugger => matches!(kind, AstKind::DebuggerStatement(_)),
        ForbiddenSyntaxKind::NestedTernary => {
            matches!(kind, AstKind::ConditionalExpression(_))
                && ctx
                    .semantic
                    .nodes()
                    .ancestor_kinds(node_id)
                    .any(|ancestor| matches!(ancestor, AstKind::ConditionalExpression(_)))
        }
    }
}
