use super::policy_ir::{
    AstRule, CaseStyle, CommentRule, CompiledPolicyRule, FileRule, ForbiddenSyntaxKind, ImportRule,
    NameSelector, NamingRule, RuleIR, SemanticRule, StringMatchKind,
};
use super::{LintContext, LintDiagnostic};
use std::collections::HashMap;

mod ast;
mod comments;
mod file;
mod imports;
mod naming;
mod semantic;

pub type CompiledConfigRule = CompiledPolicyRule;

/// Build policy rules from the JSON config.
/// Maps config keys like "max-function-params", "banned-imports", etc. to CompiledPolicyRule.
pub fn build_policy_rules_from_config(
    config: &HashMap<String, serde_json::Value>,
) -> Vec<CompiledConfigRule> {
    let mut rules = Vec::new();

    for (key, value) in config {
        let severity = match crate::cli::parse_rule_severity(value) {
            Some(s) => s,
            None => continue, // disabled
        };

        if let Some(rule_ir) = config_key_to_rule_ir(key, value) {
            rules.push(CompiledPolicyRule {
                id: key.clone(),
                source_text: String::new(),
                severity,
                message: default_message(&rule_ir),
                rule: rule_ir,
            });
        }
    }

    rules
}

/// Maps a config key + value to a RuleIR. Returns None for built-in rules
/// (which are handled separately) or unrecognized keys.
fn config_key_to_rule_ir(key: &str, value: &serde_json::Value) -> Option<RuleIR> {
    match key {
        // ── AST rules ─────────────────────────────────────────
        "max-function-params" => {
            let max = value.as_u64()? as usize;
            Some(RuleIR::Ast(AstRule::MaxFunctionParams {
                scope: Default::default(),
                max,
            }))
        }
        "no-nested-ternaries" => Some(RuleIR::Ast(AstRule::ForbiddenSyntax {
            scope: Default::default(),
            syntax: ForbiddenSyntaxKind::NestedTernary,
        })),
        "no-default-export" => Some(RuleIR::Ast(AstRule::ForbiddenSyntax {
            scope: Default::default(),
            syntax: ForbiddenSyntaxKind::DefaultExport,
        })),
        "no-switch" => Some(RuleIR::Ast(AstRule::ForbiddenSyntax {
            scope: Default::default(),
            syntax: ForbiddenSyntaxKind::Switch,
        })),
        "no-debugger" => Some(RuleIR::Ast(AstRule::ForbiddenSyntax {
            scope: Default::default(),
            syntax: ForbiddenSyntaxKind::Debugger,
        })),
        "no-try-catch" => Some(RuleIR::Ast(AstRule::ForbiddenSyntax {
            scope: Default::default(),
            syntax: ForbiddenSyntaxKind::TryCatch,
        })),

        // ── Import rules ──────────────────────────────────────
        "banned-imports" => {
            // This produces multiple rules from an array value
            // Handled specially below
            None
        }
        "no-side-effect-imports" => Some(RuleIR::Import(ImportRule::NoSideEffectImport {
            scope: Default::default(),
        })),

        // ── Naming rules ──────────────────────────────────────
        "naming-functions" => {
            let style = parse_case_style(value.as_str()?)?;
            Some(RuleIR::Naming(NamingRule::Case {
                scope: Default::default(),
                selector: NameSelector::Function,
                style,
            }))
        }
        "naming-classes" => {
            let style = parse_case_style(value.as_str()?)?;
            Some(RuleIR::Naming(NamingRule::Case {
                scope: Default::default(),
                selector: NameSelector::Class,
                style,
            }))
        }
        "naming-variables" => {
            let style = parse_case_style(value.as_str()?)?;
            Some(RuleIR::Naming(NamingRule::Case {
                scope: Default::default(),
                selector: NameSelector::Variable,
                style,
            }))
        }
        "naming-constants" => {
            let style = parse_case_style(value.as_str()?)?;
            Some(RuleIR::Naming(NamingRule::Case {
                scope: Default::default(),
                selector: NameSelector::Variable,
                style,
            }))
        }

        // ── File rules ────────────────────────────────────────
        "max-file-lines" => {
            let max = value.as_u64()? as usize;
            Some(RuleIR::File(FileRule::MaxLines {
                scope: Default::default(),
                max,
            }))
        }

        // ── Comment rules ─────────────────────────────────────
        "no-comments" => Some(RuleIR::Comment(CommentRule::NoComments {
            scope: Default::default(),
        })),
        "no-todo-comments" => Some(RuleIR::Comment(CommentRule::ForbidPattern {
            scope: Default::default(),
            pattern: "TODO".to_string(),
        })),
        "no-fixme-comments" => Some(RuleIR::Comment(CommentRule::ForbidPattern {
            scope: Default::default(),
            pattern: "FIXME".to_string(),
        })),

        // ── Semantic rules ────────────────────────────────────
        "banned-calls" => {
            // Handled specially below
            None
        }

        // Built-in rules (handled by the builtin rule engine, not policy)
        "no-explicit-any" | "no-console" | "no-empty-catch" | "prefer-const"
        | "no-unused-vars" | "unreachable-code" => None,

        // Cross-file dead code rules (handled by the dead_code engine, not policy)
        "unused-exports" | "unused-files" | "unused-dependencies" => None,

        // Unknown rules - silently ignore
        _ => None,
    }
}

/// Build policy rules, including array-valued rules that expand into multiple entries
pub fn build_all_policy_rules(
    config: &HashMap<String, serde_json::Value>,
) -> Vec<CompiledConfigRule> {
    let mut rules = build_policy_rules_from_config(config);

    // Handle banned-imports (array → one rule per entry)
    if let Some(value) = config.get("banned-imports") {
        if let Some(severity) = crate::cli::parse_rule_severity(value) {
            if let Some(arr) = value.as_array() {
                for pattern in arr {
                    if let Some(pattern_str) = pattern.as_str() {
                        rules.push(CompiledPolicyRule {
                            id: format!("banned-imports/{}", pattern_str),
                            source_text: String::new(),
                            severity: severity.clone(),
                            message: format!("Import of `{pattern_str}` is banned"),
                            rule: RuleIR::Import(ImportRule::BannedModulePattern {
                                scope: Default::default(),
                                pattern: pattern_str.to_string(),
                                match_kind: StringMatchKind::Exact,
                            }),
                        });
                    }
                }
            }
        }
    }

    // Handle banned-calls (array → one rule per entry)
    if let Some(value) = config.get("banned-calls") {
        if let Some(severity) = crate::cli::parse_rule_severity(value) {
            if let Some(arr) = value.as_array() {
                for target in arr {
                    if let Some(target_str) = target.as_str() {
                        rules.push(CompiledPolicyRule {
                            id: format!("banned-calls/{}", target_str),
                            source_text: String::new(),
                            severity: severity.clone(),
                            message: format!("Calls to `{target_str}` are banned"),
                            rule: RuleIR::Semantic(SemanticRule::BannedUsage {
                                scope: Default::default(),
                                target: target_str.to_string(),
                                require_call: true,
                                require_unshadowed_root: false,
                            }),
                        });
                    }
                }
            }
        }
    }

    rules
}

fn parse_case_style(s: &str) -> Option<CaseStyle> {
    match s {
        "camelCase" => Some(CaseStyle::CamelCase),
        "PascalCase" => Some(CaseStyle::PascalCase),
        "snake_case" => Some(CaseStyle::SnakeCase),
        "kebab-case" => Some(CaseStyle::KebabCase),
        "UPPER_SNAKE_CASE" => Some(CaseStyle::UpperSnakeCase),
        _ => None,
    }
}

pub fn run_compiled_rules(
    ctx: &LintContext,
    custom_rules: &[CompiledConfigRule],
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    for rule in custom_rules {
        diagnostics.extend(run_policy_rule(ctx, rule));
    }
    diagnostics
}

fn run_policy_rule(ctx: &LintContext, rule: &CompiledPolicyRule) -> Vec<LintDiagnostic> {
    if !rule.rule.scope().matches_path(ctx.file_path) {
        return Vec::new();
    }

    match &rule.rule {
        RuleIR::Ast(ast_rule) => ast::evaluate(ctx, rule, ast_rule),
        RuleIR::Import(import_rule) => imports::evaluate(ctx, rule, import_rule),
        RuleIR::Naming(naming_rule) => naming::evaluate(ctx, rule, naming_rule),
        RuleIR::File(file_rule) => file::evaluate(ctx, rule, file_rule),
        RuleIR::Comment(comment_rule) => comments::evaluate(ctx, rule, comment_rule),
        RuleIR::Semantic(semantic_rule) => semantic::evaluate(ctx, rule, semantic_rule),
    }
}

fn default_message(rule: &RuleIR) -> String {
    match rule {
        RuleIR::Ast(AstRule::MaxFunctionParams { max, .. }) => {
            format!("Functions must not have more than {max} parameters")
        }
        RuleIR::Ast(AstRule::ForbiddenSyntax { syntax, .. }) => {
            format!("`{}` syntax is forbidden", forbidden_syntax_name(syntax))
        }
        RuleIR::Import(ImportRule::BannedModulePattern { pattern, .. }) => {
            format!("Import of `{pattern}` is banned")
        }
        RuleIR::Import(ImportRule::NoSideEffectImport { .. }) => {
            "Side-effect-only imports are forbidden".to_string()
        }
        RuleIR::Naming(NamingRule::Affix {
            selector,
            affix,
            match_kind,
            ..
        }) => match match_kind {
            super::policy_ir::AffixMatchKind::Prefix => format!(
                "{} names must start with `{affix}`",
                selector_name(selector)
            ),
            super::policy_ir::AffixMatchKind::Suffix => format!(
                "{} names must end with `{affix}`",
                selector_name(selector)
            ),
        },
        RuleIR::Naming(NamingRule::Case {
            selector, style, ..
        }) => format!(
            "{} names must use {}",
            selector_name(selector),
            case_style_name(style)
        ),
        RuleIR::File(FileRule::MaxLines { max, .. }) => {
            format!("Files must not exceed {max} lines")
        }
        RuleIR::File(FileRule::PathPattern {
            pattern,
            expectation,
            ..
        }) => match expectation {
            super::policy_ir::PathPatternExpectation::MustMatch => {
                format!("File paths must match `{pattern}`")
            }
            super::policy_ir::PathPatternExpectation::MustNotMatch => {
                format!("File paths must not match `{pattern}`")
            }
        },
        RuleIR::Comment(CommentRule::NoComments { .. }) => {
            "Comments are not allowed".to_string()
        }
        RuleIR::Comment(CommentRule::ForbidPattern { pattern, .. }) => {
            format!("Comments must not contain `{pattern}`")
        }
        RuleIR::Semantic(SemanticRule::BannedUsage {
            target,
            require_call,
            ..
        }) => {
            if *require_call {
                format!("Calls to `{target}` are banned")
            } else {
                format!("Usage of `{target}` is banned")
            }
        }
        RuleIR::Semantic(SemanticRule::NoUnusedBindings { .. }) => {
            "Bindings must be used or prefixed with `_`".to_string()
        }
    }
}

fn forbidden_syntax_name(syntax: &ForbiddenSyntaxKind) -> &'static str {
    match syntax {
        ForbiddenSyntaxKind::TryCatch => "try/catch",
        ForbiddenSyntaxKind::Switch => "switch",
        ForbiddenSyntaxKind::DefaultExport => "default export",
        ForbiddenSyntaxKind::NestedTernary => "nested ternary",
        ForbiddenSyntaxKind::Debugger => "debugger",
    }
}

fn selector_name(selector: &NameSelector) -> &'static str {
    match selector {
        NameSelector::Function => "Function",
        NameSelector::Variable => "Variable",
        NameSelector::Class => "Class",
    }
}

fn case_style_name(style: &CaseStyle) -> &'static str {
    match style {
        CaseStyle::CamelCase => "camelCase",
        CaseStyle::PascalCase => "PascalCase",
        CaseStyle::SnakeCase => "snake_case",
        CaseStyle::KebabCase => "kebab-case",
        CaseStyle::UpperSnakeCase => "UPPER_SNAKE_CASE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_max_function_params_from_config() {
        let mut config = HashMap::new();
        config.insert("max-function-params".to_string(), serde_json::json!(3));
        let rules = build_policy_rules_from_config(&config);
        assert_eq!(rules.len(), 1);
        assert!(matches!(
            rules[0].rule,
            RuleIR::Ast(AstRule::MaxFunctionParams { max: 3, .. })
        ));
    }

    #[test]
    fn builds_naming_rule_from_config() {
        let mut config = HashMap::new();
        config.insert(
            "naming-functions".to_string(),
            serde_json::json!("camelCase"),
        );
        let rules = build_policy_rules_from_config(&config);
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0].rule, RuleIR::Naming(NamingRule::Case { .. })));
    }

    #[test]
    fn builds_banned_imports_from_array() {
        let mut config = HashMap::new();
        config.insert(
            "banned-imports".to_string(),
            serde_json::json!(["lodash", "moment"]),
        );
        let rules = build_all_policy_rules(&config);
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn disabled_rules_are_skipped() {
        let mut config = HashMap::new();
        config.insert("max-function-params".to_string(), serde_json::json!(false));
        config.insert("no-debugger".to_string(), serde_json::json!("off"));
        let rules = build_policy_rules_from_config(&config);
        assert!(rules.is_empty());
    }

    #[test]
    fn builtin_rule_keys_produce_no_policy_rules() {
        let mut config = HashMap::new();
        config.insert("no-console".to_string(), serde_json::json!(true));
        let rules = build_policy_rules_from_config(&config);
        assert!(rules.is_empty());
    }
}
