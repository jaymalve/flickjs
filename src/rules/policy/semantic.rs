use crate::rules::policy_ir::{CompiledPolicyRule, SemanticRule};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_ast::ast::{Expression, IdentifierReference, MemberExpression};
use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::symbol::SymbolFlags;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &SemanticRule,
) -> Vec<LintDiagnostic> {
    match rule {
        SemanticRule::BannedUsage {
            target,
            require_call,
            require_unshadowed_root,
            ..
        } => ctx
            .semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::CallExpression(call) => {
                    let callee = expression_static_name(&call.callee)?;
                    if &callee != target {
                        return None;
                    }
                    if *require_unshadowed_root {
                        let member = call.callee.get_member_expr()?;
                        let root = root_identifier(member)?;
                        if is_shadowed_root(ctx, root) {
                            return None;
                        }
                    }
                    Some(ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        call.span,
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    ))
                }
                AstKind::StaticMemberExpression(member) if !require_call => {
                    if member_is_call_callee(ctx, node.id()) {
                        return None;
                    }
                    let name = static_member_expression_name(member)?;
                    if &name != target {
                        return None;
                    }
                    if *require_unshadowed_root {
                        let root = root_identifier_from_expression(&member.object)?;
                        if is_shadowed_root(ctx, root) {
                            return None;
                        }
                    }
                    Some(ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        member.span(),
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    ))
                }
                AstKind::ComputedMemberExpression(member) if !require_call => {
                    if member_is_call_callee(ctx, node.id()) {
                        return None;
                    }
                    let name = computed_member_expression_name(member)?;
                    if &name != target {
                        return None;
                    }
                    if *require_unshadowed_root {
                        let root = root_identifier_from_expression(&member.object)?;
                        if is_shadowed_root(ctx, root) {
                            return None;
                        }
                    }
                    Some(ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        member.span(),
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    ))
                }
                _ => None,
            })
            .collect(),
        SemanticRule::NoUnusedBindings { .. } => ctx
            .semantic
            .scoping()
            .symbol_ids()
            .filter_map(|symbol_id| {
                let scoping = ctx.semantic.scoping();
                let flags = scoping.symbol_flags(symbol_id);
                if !should_check_symbol(flags) {
                    return None;
                }
                if has_meaningful_usage(ctx, symbol_id, flags) {
                    return None;
                }
                let name = scoping.symbol_name(symbol_id);
                if name.starts_with('_') {
                    return None;
                }
                let declaration_id = scoping.symbol_declaration(symbol_id);
                if is_exported_declaration(ctx, declaration_id) {
                    return None;
                }
                if !is_supported_unused_var_declaration(ctx, declaration_id) {
                    return None;
                }

                Some(ctx.diagnostic_with_origin(
                    compiled_rule.id.clone(),
                    compiled_rule.message.clone(),
                    scoping.symbol_span(symbol_id),
                    compiled_rule.severity.clone(),
                    RuleOrigin::English,
                ))
            })
            .collect(),
    }
}

fn expression_static_name(expression: &Expression<'_>) -> Option<String> {
    let expression = expression.without_parentheses();
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => expression
            .get_member_expr()
            .and_then(member_expression_name),
    }
}

fn member_expression_name(member: &MemberExpression<'_>) -> Option<String> {
    let object = expression_static_name(member.object())?;
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}

fn static_member_expression_name(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
) -> Option<String> {
    Some(format!(
        "{}.{}",
        expression_static_name(&member.object)?,
        member.property.name
    ))
}

fn computed_member_expression_name(
    member: &oxc_ast::ast::ComputedMemberExpression<'_>,
) -> Option<String> {
    let object = expression_static_name(&member.object)?;
    let property = member.static_property_name()?;
    Some(format!("{object}.{property}"))
}

fn member_is_call_callee(ctx: &LintContext, node_id: oxc_syntax::node::NodeId) -> bool {
    let node_span = ctx.semantic.nodes().kind(node_id).span();
    matches!(
        ctx.semantic.nodes().parent_kind(node_id),
        AstKind::CallExpression(call)
            if call
                .callee
                .get_member_expr()
                .is_some_and(|callee| callee.span() == node_span)
    )
}

fn root_identifier<'a>(member: &'a MemberExpression<'a>) -> Option<&'a IdentifierReference<'a>> {
    let object = member.object().without_parentheses();
    match object {
        Expression::Identifier(identifier) => Some(identifier),
        _ => object.get_member_expr().and_then(root_identifier),
    }
}

fn root_identifier_from_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    let expression = expression.without_parentheses();
    match expression {
        Expression::Identifier(identifier) => Some(identifier),
        _ => expression.get_member_expr().and_then(root_identifier),
    }
}

fn is_shadowed_root(ctx: &LintContext, identifier: &IdentifierReference) -> bool {
    identifier
        .reference_id
        .get()
        .and_then(|reference_id| {
            ctx.semantic
                .scoping()
                .get_reference(reference_id)
                .symbol_id()
        })
        .is_some()
}

fn is_supported_unused_var_declaration(
    ctx: &LintContext,
    declaration_id: oxc_syntax::node::NodeId,
) -> bool {
    let nodes = ctx.semantic.nodes();
    let mut current = Some(declaration_id);
    while let Some(node_id) = current {
        match nodes.kind(node_id) {
            AstKind::VariableDeclarator(_)
            | AstKind::FormalParameter(_)
            | AstKind::FormalParameterRest(_)
            | AstKind::CatchParameter(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_) => return true,
            _ => current = Some(nodes.parent_id(node_id)),
        }
    }
    false
}

fn is_exported_declaration(ctx: &LintContext, declaration_id: oxc_syntax::node::NodeId) -> bool {
    ctx.semantic
        .nodes()
        .ancestor_kinds(declaration_id)
        .any(|kind| {
            matches!(
                kind,
                AstKind::ExportNamedDeclaration(_) | AstKind::ExportDefaultDeclaration(_)
            )
        })
}

fn should_check_symbol(flags: SymbolFlags) -> bool {
    flags.is_variable() || flags.is_catch_variable() || flags.is_import()
}

fn has_meaningful_usage(
    ctx: &LintContext,
    symbol_id: oxc_syntax::symbol::SymbolId,
    flags: SymbolFlags,
) -> bool {
    ctx.semantic
        .scoping()
        .get_resolved_references(symbol_id)
        .any(|reference| reference.is_read() || (flags.is_import() && reference.is_type()))
}
