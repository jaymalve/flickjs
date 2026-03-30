use crate::rules::policy_ir::{
    AffixMatchKind, CaseStyle, CompiledPolicyRule, NameSelector, NamingRule,
};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_ast::ast::Expression;
use oxc_ast::AstKind;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &NamingRule,
) -> Vec<LintDiagnostic> {
    match rule {
        NamingRule::Affix {
            selector,
            affix,
            match_kind,
            ..
        } => collect_names(ctx, selector)
            .into_iter()
            .filter_map(|(name, span)| {
                let matches = match match_kind {
                    AffixMatchKind::Prefix => name.starts_with(affix),
                    AffixMatchKind::Suffix => name.ends_with(affix),
                };
                (!matches).then(|| {
                    ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        span,
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            })
            .collect(),
        NamingRule::Case {
            selector, style, ..
        } => collect_names(ctx, selector)
            .into_iter()
            .filter_map(|(name, span)| {
                (!matches_case(&name, style)).then(|| {
                    ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        span,
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            })
            .collect(),
    }
}

fn collect_names(ctx: &LintContext, selector: &NameSelector) -> Vec<(String, oxc_span::Span)> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match (selector, node.kind()) {
            (NameSelector::Function, AstKind::Function(function)) => function
                .id
                .as_ref()
                .map(|identifier| (identifier.name.to_string(), identifier.span)),
            (NameSelector::Function, AstKind::VariableDeclarator(declarator)) => {
                let init = declarator.init.as_ref()?;
                let is_function_like = matches!(
                    init.without_parentheses(),
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                );
                is_function_like.then(|| {
                    declarator
                        .id
                        .get_binding_identifiers()
                        .into_iter()
                        .next()
                        .map(|binding| (binding.name.to_string(), binding.span))
                })?
            }
            (NameSelector::Variable, AstKind::VariableDeclarator(declarator)) => declarator
                .id
                .get_binding_identifiers()
                .into_iter()
                .next()
                .map(|binding| (binding.name.to_string(), binding.span)),
            (NameSelector::Class, AstKind::Class(class)) => class
                .id
                .as_ref()
                .map(|identifier| (identifier.name.to_string(), identifier.span)),
            _ => None,
        })
        .collect()
}

fn matches_case(name: &str, style: &CaseStyle) -> bool {
    match style {
        CaseStyle::CamelCase => is_camel_case(name),
        CaseStyle::PascalCase => is_pascal_case(name),
        CaseStyle::SnakeCase => is_snake_case(name),
        CaseStyle::KebabCase => is_kebab_case(name),
        CaseStyle::UpperSnakeCase => is_upper_snake_case(name),
    }
}

fn is_camel_case(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric())
        && !value.contains('_')
        && !value.contains('-')
}

fn is_pascal_case(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|first| first.is_ascii_uppercase())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric())
        && !value.contains('_')
        && !value.contains('-')
}

fn is_snake_case(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        && !value.starts_with('_')
        && !value.ends_with('_')
        && !value.contains("__")
}

fn is_kebab_case(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn is_upper_snake_case(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && !value.starts_with('_')
        && !value.ends_with('_')
        && !value.contains("__")
}
