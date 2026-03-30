use crate::rules::policy_ir::{
    CompiledPolicyRule, FileRule, PathPatternExpectation, RuleScope,
};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_span::Span;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &FileRule,
) -> Vec<LintDiagnostic> {
    match rule {
        FileRule::MaxLines { max, .. } => {
            let line_count = ctx.source.lines().count();
            if line_count <= *max {
                return Vec::new();
            }

            vec![ctx.diagnostic_with_origin(
                compiled_rule.id.clone(),
                format!("File has {line_count} lines; maximum allowed is {max}"),
                Span::empty(0),
                compiled_rule.severity.clone(),
                RuleOrigin::Config,
            )]
        }
        FileRule::PathPattern {
            pattern,
            expectation,
            ..
        } => {
            let normalized_path = ctx.file_path.to_string_lossy().replace('\\', "/");
            let matches = path_pattern_matches(pattern, &normalized_path);
            let violated = match expectation {
                PathPatternExpectation::MustMatch => !matches,
                PathPatternExpectation::MustNotMatch => matches,
            };
            if !violated {
                return Vec::new();
            }

            vec![ctx.diagnostic_with_origin(
                compiled_rule.id.clone(),
                compiled_rule.message.clone(),
                Span::empty(0),
                compiled_rule.severity.clone(),
                RuleOrigin::Config,
            )]
        }
    }
}

fn path_pattern_matches(pattern: &str, normalized_path: &str) -> bool {
    let scope = RuleScope {
        include_paths: vec![pattern.to_string()],
        exclude_paths: Vec::new(),
        extensions: Vec::new(),
    };
    scope.matches_path(std::path::Path::new(normalized_path))
}
