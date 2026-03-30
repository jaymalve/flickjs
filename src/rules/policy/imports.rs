use crate::rules::policy_ir::{CompiledPolicyRule, ImportRule, StringMatchKind};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_ast::AstKind;
use oxc_span::GetSpan;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &ImportRule,
) -> Vec<LintDiagnostic> {
    match rule {
        ImportRule::BannedModulePattern {
            pattern,
            match_kind,
            ..
        } => ctx
            .semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::ImportDeclaration(declaration)
                    if matches_string(
                        declaration.source.value.as_str(),
                        pattern,
                        match_kind,
                    ) =>
                {
                    Some(ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        declaration.span(),
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    ))
                }
                _ => None,
            })
            .collect(),
        ImportRule::NoSideEffectImport { .. } => ctx
            .semantic
            .nodes()
            .iter()
            .filter_map(|node| match node.kind() {
                AstKind::ImportDeclaration(declaration) if declaration.specifiers.is_none() => {
                    Some(ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        declaration.span(),
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    ))
                }
                _ => None,
            })
            .collect(),
    }
}

fn matches_string(value: &str, pattern: &str, match_kind: &StringMatchKind) -> bool {
    match match_kind {
        StringMatchKind::Exact => value == pattern,
        StringMatchKind::Prefix => value.starts_with(pattern),
        StringMatchKind::Suffix => value.ends_with(pattern),
        StringMatchKind::Contains => value.contains(pattern),
        StringMatchKind::Glob => glob_match(pattern, value),
    }
}

fn glob_match(pattern: &str, value: &str) -> bool {
    let pattern_chars = pattern.chars().collect::<Vec<_>>();
    let value_chars = value.chars().collect::<Vec<_>>();
    let mut memo = vec![vec![None; value_chars.len() + 1]; pattern_chars.len() + 1];
    glob_match_inner(&pattern_chars, &value_chars, 0, 0, &mut memo)
}

fn glob_match_inner(
    pattern: &[char],
    value: &[char],
    pattern_index: usize,
    value_index: usize,
    memo: &mut [Vec<Option<bool>>],
) -> bool {
    if let Some(cached) = memo[pattern_index][value_index] {
        return cached;
    }

    let result = if pattern_index == pattern.len() {
        value_index == value.len()
    } else {
        match pattern[pattern_index] {
            '*' => {
                glob_match_inner(pattern, value, pattern_index + 1, value_index, memo)
                    || (value_index < value.len()
                        && glob_match_inner(pattern, value, pattern_index, value_index + 1, memo))
            }
            '?' => {
                value_index < value.len()
                    && glob_match_inner(pattern, value, pattern_index + 1, value_index + 1, memo)
            }
            current => {
                value_index < value.len()
                    && current == value[value_index]
                    && glob_match_inner(pattern, value, pattern_index + 1, value_index + 1, memo)
            }
        }
    };

    memo[pattern_index][value_index] = Some(result);
    result
}
