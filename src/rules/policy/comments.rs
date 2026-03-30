use crate::rules::policy_ir::{CommentRule, CompiledPolicyRule};
use crate::rules::{LintContext, LintDiagnostic, RuleOrigin};
use oxc_span::Span;

pub fn evaluate(
    ctx: &LintContext,
    compiled_rule: &CompiledPolicyRule,
    rule: &CommentRule,
) -> Vec<LintDiagnostic> {
    let comments = extract_comments(ctx.source);
    match rule {
        CommentRule::NoComments { .. } => comments
            .into_iter()
            .map(|comment| {
                ctx.diagnostic_with_origin(
                    compiled_rule.id.clone(),
                    compiled_rule.message.clone(),
                    Span::empty(comment.start as u32),
                    compiled_rule.severity.clone(),
                    RuleOrigin::English,
                )
            })
            .collect(),
        CommentRule::ForbidPattern { pattern, .. } => {
            let normalized_pattern = pattern.to_ascii_lowercase();
            comments
                .into_iter()
                .filter(|comment| comment.text.to_ascii_lowercase().contains(&normalized_pattern))
                .map(|comment| {
                    ctx.diagnostic_with_origin(
                        compiled_rule.id.clone(),
                        compiled_rule.message.clone(),
                        Span::empty(comment.start as u32),
                        compiled_rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
                .collect()
        }
    }
}

#[derive(Debug, Clone)]
struct SourceComment {
    start: usize,
    text: String,
}

fn extract_comments(source: &str) -> Vec<SourceComment> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut index = 0;

    while index + 1 < bytes.len() {
        match (bytes[index], bytes[index + 1]) {
            (b'/', b'/') => {
                let start = index;
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                comments.push(SourceComment {
                    start,
                    text: source[start + 2..index].trim().to_string(),
                });
            }
            (b'/', b'*') => {
                let start = index;
                index += 2;
                let content_start = index;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                    index += 1;
                }
                let content_end = index.min(bytes.len());
                if index + 1 < bytes.len() {
                    index += 2;
                }
                comments.push(SourceComment {
                    start,
                    text: source[content_start..content_end].trim().to_string(),
                });
            }
            _ => index += 1,
        }
    }

    comments
}
