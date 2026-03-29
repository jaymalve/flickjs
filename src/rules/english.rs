use crate::cli::{EnglishRuleConfig, ZarcAuthConfig};

use super::english_llm::{build_compiler, EnglishRuleCompiler};
use super::{hash_bytes, LintContext, LintDiagnostic, RuleOrigin, Severity};
use miette::Result;
use oxc_ast::ast::{
    ComputedMemberExpression, Expression, MemberExpression, StaticMemberExpression,
};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const ENGLISH_RULE_ARTIFACT_SCHEMA_VERSION: u32 = 1;
const ENGLISH_RULE_COMPILER_VERSION: u32 = 2;

const SUPPORTED_FORMS: &[&str] = &[
    "no function should have more than <number> params",
    "do not import <module>",
    "do not call <callee>",
    "do not use <member-or-callee>",
    "function names should start with <prefix>",
    "function names should end with <suffix>",
    "no file should have more than <number> lines",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledEnglishRule {
    pub id: String,
    pub source_text: String,
    pub severity: Severity,
    pub message: String,
    pub predicate: EnglishPredicate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum EnglishPredicate {
    MaxFunctionParams {
        max: usize,
    },
    BannedImport {
        module: String,
    },
    BannedUsage {
        target: String,
        require_call: bool,
    },
    FunctionNameAffix {
        affix: String,
        match_kind: NameMatchKind,
    },
    MaxFileLines {
        max: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NameMatchKind {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EnglishRulesArtifactHeader {
    schema_version: u32,
    compiler_version: u32,
    compiler_fingerprint: String,
    config_fingerprint: String,
}

impl EnglishRulesArtifactHeader {
    fn new(config_fingerprint: &str, compiler_fingerprint: &str) -> Self {
        Self {
            schema_version: ENGLISH_RULE_ARTIFACT_SCHEMA_VERSION,
            compiler_version: ENGLISH_RULE_COMPILER_VERSION,
            compiler_fingerprint: compiler_fingerprint.to_string(),
            config_fingerprint: config_fingerprint.to_string(),
        }
    }

    fn matches(&self, config_fingerprint: &str, compiler_fingerprint: &str) -> bool {
        self.schema_version == ENGLISH_RULE_ARTIFACT_SCHEMA_VERSION
            && self.compiler_version == ENGLISH_RULE_COMPILER_VERSION
            && self.compiler_fingerprint == compiler_fingerprint
            && self.config_fingerprint == config_fingerprint
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EnglishRulesArtifact {
    header: EnglishRulesArtifactHeader,
    rules: Vec<CompiledEnglishRule>,
}

pub fn compiled_rules_cache_path(cache_path: &Path) -> PathBuf {
    let parent = cache_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = cache_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(".zarc-cache.json");

    if let Some(base) = file_name.strip_suffix(".json") {
        parent.join(format!("{base}.rules.json"))
    } else {
        parent.join(format!("{file_name}.rules.json"))
    }
}

pub fn load_or_compile(
    definitions: &[EnglishRuleConfig],
    overrides: &HashMap<String, String>,
    auth: Option<&ZarcAuthConfig>,
    config_fingerprint: &str,
    cache_path: &Path,
) -> Result<Vec<CompiledEnglishRule>> {
    if definitions.is_empty() {
        return Ok(Vec::new());
    }

    let compiler = build_compiler(auth)?;
    let compiler_fingerprint = compiler_fingerprint(compiler.as_deref());
    let artifact_path = compiled_rules_cache_path(cache_path);
    if let Some(artifact) = load_artifact(
        &artifact_path,
        config_fingerprint,
        &compiler_fingerprint,
    )? {
        return Ok(artifact.rules);
    }

    let compiled = compile_rules_with_compiler(definitions, overrides, compiler.as_deref())?;
    let artifact = EnglishRulesArtifact {
        header: EnglishRulesArtifactHeader::new(config_fingerprint, &compiler_fingerprint),
        rules: compiled.clone(),
    };
    persist_artifact(&artifact_path, &artifact)?;
    Ok(compiled)
}

pub fn compile_rules(
    definitions: &[EnglishRuleConfig],
    overrides: &HashMap<String, String>,
) -> Result<Vec<CompiledEnglishRule>> {
    compile_rules_with_compiler(definitions, overrides, None)
}

fn compile_rules_with_compiler(
    definitions: &[EnglishRuleConfig],
    overrides: &HashMap<String, String>,
    compiler: Option<&dyn EnglishRuleCompiler>,
) -> Result<Vec<CompiledEnglishRule>> {
    let mut compiled = Vec::new();

    for (index, definition) in definitions.iter().enumerate() {
        if let Some(rule) = compile_definition(index, definition, overrides, compiler)? {
            compiled.push(rule);
        }
    }

    Ok(compiled)
}

fn compile_definition(
    index: usize,
    definition: &EnglishRuleConfig,
    overrides: &HashMap<String, String>,
    compiler: Option<&dyn EnglishRuleCompiler>,
) -> Result<Option<CompiledEnglishRule>> {
    let default_severity = parse_severity(&definition.severity)?;
    let predicate = compile_predicate(index, &definition.text, compiler)?;
    let normalized = normalize_rule_text(&definition.text);
    let id = rule_id(&predicate, &normalized);

    let override_entry = overrides
        .get(&id)
        .map(|value| parse_severity(value))
        .transpose()?;
    let severity = match override_entry {
        Some(Some(level)) => Some(level),
        Some(None) => None,
        None => default_severity,
    };

    if let Some(severity) = severity {
        let message = default_message(&predicate);
        Ok(Some(CompiledEnglishRule {
            id,
            source_text: definition.text.trim().to_string(),
            severity,
            message,
            predicate,
        }))
    } else {
        Ok(None)
    }
}

fn compile_predicate(
    index: usize,
    text: &str,
    compiler: Option<&dyn EnglishRuleCompiler>,
) -> Result<EnglishPredicate> {
    if let Some(predicate) = parse_rule(text) {
        return Ok(predicate);
    }

    if let Some(compiler) = compiler {
        return compiler
            .compile_rule(text)?
            .ok_or_else(|| {
                miette::miette!(
                    "English rule compiler could not deterministically map lint.english_rules[{}]: {:?} into a supported native predicate",
                    index,
                    text
                )
            });
    }

    Err(miette::miette!(
        "Unsupported english rule at lint.english_rules[{}]: {:?}. Supported native forms: {}. Add `api_key = \"...\"` to `.zarcrc` to enable hosted natural-language compilation.",
        index,
        text,
        SUPPORTED_FORMS.join("; ")
    ))
}

pub fn run_compiled_rules(
    ctx: &LintContext,
    custom_rules: &[CompiledEnglishRule],
) -> Vec<LintDiagnostic> {
    let mut diagnostics = Vec::new();
    for rule in custom_rules {
        diagnostics.extend(run_rule(ctx, rule));
    }
    diagnostics
}

fn run_rule(ctx: &LintContext, rule: &CompiledEnglishRule) -> Vec<LintDiagnostic> {
    match &rule.predicate {
        EnglishPredicate::MaxFunctionParams { max } => {
            run_max_function_params_rule(ctx, rule, *max)
        }
        EnglishPredicate::BannedImport { module } => run_banned_import_rule(ctx, rule, module),
        EnglishPredicate::BannedUsage {
            target,
            require_call,
        } => run_banned_usage_rule(ctx, rule, target, *require_call),
        EnglishPredicate::FunctionNameAffix { affix, match_kind } => {
            run_function_name_rule(ctx, rule, affix, match_kind)
        }
        EnglishPredicate::MaxFileLines { max } => run_max_file_lines_rule(ctx, rule, *max),
    }
}

fn run_max_function_params_rule(
    ctx: &LintContext,
    rule: &CompiledEnglishRule,
    max: usize,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::Function(function) if function.body.is_some() => {
                let count = parameter_count(&function.params);
                (count > max).then(|| {
                    let span = function
                        .id
                        .as_ref()
                        .map(|identifier| identifier.span)
                        .unwrap_or_else(|| function.span());
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        format!("Function has {count} parameters; maximum allowed is {max}"),
                        span,
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            AstKind::ArrowFunctionExpression(function) => {
                let count = parameter_count(&function.params);
                (count > max).then(|| {
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        format!("Function has {count} parameters; maximum allowed is {max}"),
                        function.span(),
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            _ => None,
        })
        .collect()
}

fn run_banned_import_rule(
    ctx: &LintContext,
    rule: &CompiledEnglishRule,
    module: &str,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::ImportDeclaration(declaration)
                if declaration.source.value.as_str() == module =>
            {
                Some(ctx.diagnostic_with_origin(
                    rule.id.clone(),
                    rule.message.clone(),
                    declaration.span(),
                    rule.severity.clone(),
                    RuleOrigin::English,
                ))
            }
            _ => None,
        })
        .collect()
}

fn run_banned_usage_rule(
    ctx: &LintContext,
    rule: &CompiledEnglishRule,
    target: &str,
    require_call: bool,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::CallExpression(call) => {
                let callee = expression_static_name(&call.callee)?;
                (callee == target).then(|| {
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        rule.message.clone(),
                        call.span,
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            AstKind::StaticMemberExpression(member) if !require_call => {
                if member_is_call_callee(ctx, node.id()) {
                    return None;
                }
                let name = static_member_expression_name(member)?;
                (name == target).then(|| {
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        rule.message.clone(),
                        member.span(),
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            AstKind::ComputedMemberExpression(member) if !require_call => {
                if member_is_call_callee(ctx, node.id()) {
                    return None;
                }
                let name = computed_member_expression_name(member)?;
                (name == target).then(|| {
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        rule.message.clone(),
                        member.span(),
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            _ => None,
        })
        .collect()
}

fn run_function_name_rule(
    ctx: &LintContext,
    rule: &CompiledEnglishRule,
    affix: &str,
    match_kind: &NameMatchKind,
) -> Vec<LintDiagnostic> {
    ctx.semantic
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            AstKind::Function(function) => {
                let identifier = function.id.as_ref()?;
                let matches = match match_kind {
                    NameMatchKind::Prefix => identifier.name.as_str().starts_with(affix),
                    NameMatchKind::Suffix => identifier.name.as_str().ends_with(affix),
                };

                (!matches).then(|| {
                    ctx.diagnostic_with_origin(
                        rule.id.clone(),
                        rule.message.clone(),
                        identifier.span,
                        rule.severity.clone(),
                        RuleOrigin::English,
                    )
                })
            }
            _ => None,
        })
        .collect()
}

fn run_max_file_lines_rule(
    ctx: &LintContext,
    rule: &CompiledEnglishRule,
    max: usize,
) -> Vec<LintDiagnostic> {
    let line_count = ctx.source.lines().count();
    if line_count <= max {
        return Vec::new();
    }

    vec![ctx.diagnostic_with_origin(
        rule.id.clone(),
        format!("File has {line_count} lines; maximum allowed is {max}"),
        Span::empty(0),
        rule.severity.clone(),
        RuleOrigin::English,
    )]
}

fn parameter_count(params: &oxc_ast::ast::FormalParameters<'_>) -> usize {
    params.items.len() + usize::from(params.rest.is_some())
}

fn parse_rule(text: &str) -> Option<EnglishPredicate> {
    let normalized = normalize_rule_text(text);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    parse_max_function_params(&tokens)
        .or_else(|| parse_banned_import(&tokens))
        .or_else(|| parse_banned_usage(&tokens))
        .or_else(|| parse_function_name_rule(&tokens))
        .or_else(|| parse_max_file_lines(&tokens))
}

fn parse_max_function_params(tokens: &[&str]) -> Option<EnglishPredicate> {
    (tokens.len() == 8
        && tokens[0] == "no"
        && matches!(tokens[1], "function" | "functions")
        && matches!(tokens[2], "should" | "must" | "shall")
        && tokens[3] == "have"
        && tokens[4] == "more"
        && tokens[5] == "than"
        && matches!(tokens[7], "param" | "params" | "parameter" | "parameters"))
    .then(|| tokens[6].parse::<usize>().ok())
    .flatten()
    .map(|max| EnglishPredicate::MaxFunctionParams { max })
}

fn parse_banned_import(tokens: &[&str]) -> Option<EnglishPredicate> {
    if tokens.len() >= 4 && tokens[0] == "do" && tokens[1] == "not" && tokens[2] == "import" {
        return Some(EnglishPredicate::BannedImport {
            module: strip_wrapping_quotes(&tokens[3..].join(" ")),
        });
    }

    if tokens.len() >= 4
        && tokens[0] == "no"
        && matches!(tokens[1], "imports" | "import")
        && tokens[2] == "from"
    {
        return Some(EnglishPredicate::BannedImport {
            module: strip_wrapping_quotes(&tokens[3..].join(" ")),
        });
    }

    None
}

fn parse_banned_usage(tokens: &[&str]) -> Option<EnglishPredicate> {
    if tokens.len() >= 4 && tokens[0] == "do" && tokens[1] == "not" {
        if tokens[2] == "call" {
            return Some(EnglishPredicate::BannedUsage {
                target: strip_wrapping_quotes(&tokens[3..].join(" ")),
                require_call: true,
            });
        }

        if tokens[2] == "use" {
            return Some(EnglishPredicate::BannedUsage {
                target: strip_wrapping_quotes(&tokens[3..].join(" ")),
                require_call: false,
            });
        }
    }

    if tokens.len() >= 4
        && tokens[0] == "no"
        && matches!(tokens[1], "calls" | "call")
        && tokens[2] == "to"
    {
        return Some(EnglishPredicate::BannedUsage {
            target: strip_wrapping_quotes(&tokens[3..].join(" ")),
            require_call: true,
        });
    }

    None
}

fn parse_function_name_rule(tokens: &[&str]) -> Option<EnglishPredicate> {
    if tokens.len() >= 6
        && tokens[0] == "function"
        && matches!(tokens[1], "name" | "names")
        && matches!(tokens[2], "should" | "must" | "shall")
        && tokens[4] == "with"
    {
        let affix = strip_wrapping_quotes(&tokens[5..].join(" "));
        let match_kind = match tokens[3] {
            "start" => NameMatchKind::Prefix,
            "end" => NameMatchKind::Suffix,
            _ => return None,
        };

        return Some(EnglishPredicate::FunctionNameAffix { affix, match_kind });
    }

    None
}

fn parse_max_file_lines(tokens: &[&str]) -> Option<EnglishPredicate> {
    (tokens.len() == 8
        && tokens[0] == "no"
        && tokens[1] == "file"
        && matches!(tokens[2], "should" | "must" | "shall")
        && tokens[3] == "have"
        && tokens[4] == "more"
        && tokens[5] == "than"
        && matches!(tokens[7], "line" | "lines"))
    .then(|| tokens[6].parse::<usize>().ok())
    .flatten()
    .map(|max| EnglishPredicate::MaxFileLines { max })
}

fn default_message(predicate: &EnglishPredicate) -> String {
    match predicate {
        EnglishPredicate::MaxFunctionParams { max } => {
            format!("Functions must not have more than {max} parameters")
        }
        EnglishPredicate::BannedImport { module } => {
            format!("Imports from `{module}` are forbidden")
        }
        EnglishPredicate::BannedUsage {
            target,
            require_call,
        } => {
            if *require_call {
                format!("Calls to `{target}` are forbidden")
            } else {
                format!("Usage of `{target}` is forbidden")
            }
        }
        EnglishPredicate::FunctionNameAffix { affix, match_kind } => match match_kind {
            NameMatchKind::Prefix => format!("Function names must start with `{affix}`"),
            NameMatchKind::Suffix => format!("Function names must end with `{affix}`"),
        },
        EnglishPredicate::MaxFileLines { max } => {
            format!("Files must not exceed {max} lines")
        }
    }
}

fn rule_id(predicate: &EnglishPredicate, normalized_text: &str) -> String {
    let kind = predicate_kind_name(predicate);
    let digest = hash_bytes(format!("{kind}:{normalized_text}").as_bytes());
    format!("english/{kind}/{}", &digest[..10])
}

fn predicate_kind_name(predicate: &EnglishPredicate) -> &'static str {
    match predicate {
        EnglishPredicate::MaxFunctionParams { .. } => "max-function-params",
        EnglishPredicate::BannedImport { .. } => "banned-import",
        EnglishPredicate::BannedUsage { .. } => "banned-usage",
        EnglishPredicate::FunctionNameAffix { .. } => "function-name-affix",
        EnglishPredicate::MaxFileLines { .. } => "max-file-lines",
    }
}

fn parse_severity(level: &str) -> Result<Option<Severity>> {
    if level.eq_ignore_ascii_case("off") {
        return Ok(None);
    }

    if level.eq_ignore_ascii_case("warn") {
        return Ok(Some(Severity::Warning));
    }

    if level.eq_ignore_ascii_case("error") {
        return Ok(Some(Severity::Error));
    }

    Err(miette::miette!(
        "Unsupported severity {:?} for english rule. Expected one of: off, warn, error",
        level
    ))
}

fn normalize_rule_text(text: &str) -> String {
    let trimmed = text.trim().trim_end_matches(['.', ';', '!', '?']);
    let mut normalized = String::with_capacity(trimmed.len());
    let mut in_whitespace = false;

    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !in_whitespace && !normalized.is_empty() {
                normalized.push(' ');
            }
            in_whitespace = true;
        } else {
            normalized.push(ch.to_ascii_lowercase());
            in_whitespace = false;
        }
    }

    normalized
}

fn strip_wrapping_quotes(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
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

fn static_member_expression_name(member: &StaticMemberExpression<'_>) -> Option<String> {
    Some(format!(
        "{}.{}",
        expression_static_name(&member.object)?,
        member.property.name
    ))
}

fn computed_member_expression_name(member: &ComputedMemberExpression<'_>) -> Option<String> {
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

fn compiler_fingerprint(compiler: Option<&dyn EnglishRuleCompiler>) -> String {
    let compiler_mode = compiler
        .map(|compiler| compiler.fingerprint_material())
        .unwrap_or_else(|| "manual-only".to_string());
    hash_bytes(
        format!(
            "{ENGLISH_RULE_ARTIFACT_SCHEMA_VERSION}:{ENGLISH_RULE_COMPILER_VERSION}:{}:{compiler_mode}",
            SUPPORTED_FORMS.join("|")
        )
        .as_bytes(),
    )
}

fn load_artifact(
    path: &Path,
    config_fingerprint: &str,
    compiler_fingerprint: &str,
) -> Result<Option<EnglishRulesArtifact>> {
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read(path)
        .map_err(|error| miette::miette!("Failed to read english-rule artifact: {}", error))?;
    let artifact = match serde_json::from_slice::<EnglishRulesArtifact>(&data) {
        Ok(artifact) => artifact,
        Err(_) => return Ok(None),
    };

    if artifact
        .header
        .matches(config_fingerprint, compiler_fingerprint)
    {
        Ok(Some(artifact))
    } else {
        Ok(None)
    }
}

fn persist_artifact(path: &Path, artifact: &EnglishRulesArtifact) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                miette::miette!(
                    "Failed to create english-rule cache directory {}: {}",
                    parent.display(),
                    error
                )
            })?;
        }
    }

    let data = serde_json::to_vec(artifact)
        .map_err(|error| miette::miette!("Failed to serialize english-rule artifact: {}", error))?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("zarc-cache.rules.json");
    let temp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));

    fs::write(&temp_path, data)
        .map_err(|error| miette::miette!("Failed to write english-rule artifact: {}", error))?;

    if path.exists() {
        fs::remove_file(path).map_err(|error| {
            miette::miette!("Failed to replace english-rule artifact: {}", error)
        })?;
    }

    fs::rename(&temp_path, path)
        .map_err(|error| miette::miette!("Failed to finalize english-rule artifact: {}", error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::english_llm::EnglishRuleCompiler;
    use crate::rules::lint_source_for_test_with_english_rules;
    use std::collections::HashMap;
    use tempfile::tempdir;

    struct MockEnglishRuleCompiler {
        predicate: Option<EnglishPredicate>,
        error: Option<String>,
        fingerprint: &'static str,
    }

    impl EnglishRuleCompiler for MockEnglishRuleCompiler {
        fn compile_rule(&self, _rule_text: &str) -> Result<Option<EnglishPredicate>> {
            if let Some(error) = &self.error {
                return Err(miette::miette!("{}", error));
            }

            Ok(self.predicate.clone())
        }

        fn fingerprint_material(&self) -> String {
            self.fingerprint.to_string()
        }
    }

    fn english_rule(text: &str, severity: &str) -> EnglishRuleConfig {
        EnglishRuleConfig {
            text: text.to_string(),
            severity: severity.to_string(),
        }
    }

    fn english_overrides(map: Vec<(&str, &str)>) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        for (key, value) in map {
            overrides.insert(key.to_string(), value.to_string());
        }
        overrides
    }

    fn english_diagnostics<'a>(result: &'a crate::rules::LintResult) -> Vec<&'a LintDiagnostic> {
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.origin == RuleOrigin::English)
            .collect()
    }

    #[test]
    fn compiles_max_function_params_rule() {
        let rules = compile_rules(
            &[english_rule(
                "no function should have more than 3 params",
                "error",
            )],
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].predicate,
            EnglishPredicate::MaxFunctionParams { max: 3 }
        );
        assert_eq!(rules[0].severity, Severity::Error);
    }

    #[test]
    fn rejects_unsupported_english_rule() {
        let error = compile_rules(
            &[english_rule(
                "all reducers should stay tiny and elegant",
                "warn",
            )],
            &HashMap::new(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("Unsupported english rule"));
    }

    #[test]
    fn unsupported_english_rule_points_to_zarcrc_auth_flow() {
        let error = compile_rules(
            &[english_rule(
                "all reducers should stay tiny and elegant",
                "warn",
            )],
            &HashMap::new(),
        )
        .unwrap_err();

        assert!(error.to_string().contains(".zarcrc"));
    }

    #[test]
    fn compiled_artifact_round_trips() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("cache.json");
        let definitions = vec![english_rule(
            "no function should have more than 2 params",
            "warn",
        )];

        let first = load_or_compile(
            &definitions,
            &HashMap::new(),
            None,
            "fingerprint-a",
            &cache_path,
        )
        .unwrap();
        let artifact_path = compiled_rules_cache_path(&cache_path);
        let loaded = load_artifact(
            &artifact_path,
            "fingerprint-a",
            &compiler_fingerprint(None),
        )
            .unwrap()
            .expect("artifact should be readable");

        assert!(artifact_path.exists());
        assert_eq!(first, loaded.rules);
    }

    #[test]
    fn llm_compiles_unsupported_phrasing_into_supported_predicate() {
        let compiler = MockEnglishRuleCompiler {
            predicate: Some(EnglishPredicate::MaxFunctionParams { max: 2 }),
            error: None,
            fingerprint: "mock-v1",
        };
        let rules = compile_rules_with_compiler(
            &[english_rule("functions must accept at most 2 arguments", "warn")],
            &HashMap::new(),
            Some(&compiler),
        )
        .unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].predicate,
            EnglishPredicate::MaxFunctionParams { max: 2 }
        );
    }

    #[test]
    fn llm_errors_fail_closed() {
        let compiler = MockEnglishRuleCompiler {
            predicate: None,
            error: Some("English rule compiler returned invalid JSON".to_string()),
            fingerprint: "mock-v1",
        };
        let error = compile_rules_with_compiler(
            &[english_rule("functions must accept at most 2 arguments", "warn")],
            &HashMap::new(),
            Some(&compiler),
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("English rule compiler returned invalid JSON"));
    }

    #[test]
    fn compiled_artifact_invalidates_when_compiler_fingerprint_changes() {
        let dir = tempdir().unwrap();
        let artifact_path = dir.path().join("cache.rules.json");
        let artifact = EnglishRulesArtifact {
            header: EnglishRulesArtifactHeader::new("fingerprint-a", "compiler-a"),
            rules: vec![CompiledEnglishRule {
                id: "english/max-function-params/mock".to_string(),
                source_text: "no function should have more than 2 params".to_string(),
                severity: Severity::Warning,
                message: "Functions must not have more than 2 parameters".to_string(),
                predicate: EnglishPredicate::MaxFunctionParams { max: 2 },
            }],
        };

        persist_artifact(&artifact_path, &artifact).unwrap();

        let loaded = load_artifact(&artifact_path, "fingerprint-a", "compiler-b").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn max_function_params_rule_reports_diagnostics() {
        let rules = compile_rules(
            &[english_rule(
                "no function should have more than 2 params",
                "error",
            )],
            &HashMap::new(),
        )
        .unwrap();
        let result = lint_source_for_test_with_english_rules(
            "test.js",
            "function sum(a, b, c) { return a + b + c; }\n",
            &rules,
        );
        let diagnostics = english_diagnostics(&result);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("maximum allowed is 2"));
    }

    #[test]
    fn banned_import_rule_reports_diagnostics() {
        let rules = compile_rules(
            &[english_rule("do not import lodash", "warn")],
            &HashMap::new(),
        )
        .unwrap();
        let result = lint_source_for_test_with_english_rules(
            "test.js",
            "import thing from 'lodash';\n",
            &rules,
        );
        let diagnostics = english_diagnostics(&result);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "Imports from `lodash` are forbidden"
        );
    }

    #[test]
    fn function_name_prefix_rule_reports_diagnostics() {
        let rules = compile_rules(
            &[english_rule("function names should start with use", "warn")],
            &HashMap::new(),
        )
        .unwrap();
        let result =
            lint_source_for_test_with_english_rules("test.js", "function readData() {}\n", &rules);
        let diagnostics = english_diagnostics(&result);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].message,
            "Function names must start with `use`"
        );
    }

    #[test]
    fn lint_rules_map_toggles_english_rule() {
        let definitions = vec![english_rule(
            "no function should have more than 3 params",
            "error",
        )];
        let base = compile_rules(&definitions, &HashMap::new()).unwrap();
        let id = base[0].id.clone();

        let overrides_off = english_overrides(vec![(id.as_str(), "off")]);
        assert!(compile_rules(&definitions, &overrides_off)
            .unwrap()
            .is_empty());
        let overrides_warn = english_overrides(vec![(id.as_str(), "warn")]);
        let rules = compile_rules(&definitions, &overrides_warn).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].severity, Severity::Warning);
    }
}
