use crate::cli::{EnglishRuleConfig, ZarcAuthConfig};

use super::english_llm::{build_compiler, EnglishRuleCompiler};
use super::policy_ir::{
    AffixMatchKind, AstRule, CommentRule, CompiledPolicyRule, FileRule, ImportRule, NameSelector,
    NamingRule, PathPatternExpectation, RuleIR, SemanticRule, StringMatchKind,
};
use super::{hash_bytes, LintContext, LintDiagnostic, Severity};
use miette::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

mod ast;
mod comments;
mod file;
mod imports;
mod naming;
mod semantic;

const POLICY_RULE_ARTIFACT_SCHEMA_VERSION: u32 = 2;
const POLICY_RULE_COMPILER_VERSION: u32 = 3;

pub type CompiledEnglishRule = CompiledPolicyRule;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PolicyRulesArtifactHeader {
    schema_version: u32,
    compiler_version: u32,
    compiler_fingerprint: String,
    config_fingerprint: String,
}

impl PolicyRulesArtifactHeader {
    fn new(config_fingerprint: &str, compiler_fingerprint: &str) -> Self {
        Self {
            schema_version: POLICY_RULE_ARTIFACT_SCHEMA_VERSION,
            compiler_version: POLICY_RULE_COMPILER_VERSION,
            compiler_fingerprint: compiler_fingerprint.to_string(),
            config_fingerprint: config_fingerprint.to_string(),
        }
    }

    fn matches(&self, config_fingerprint: &str, compiler_fingerprint: &str) -> bool {
        self.schema_version == POLICY_RULE_ARTIFACT_SCHEMA_VERSION
            && self.compiler_version == POLICY_RULE_COMPILER_VERSION
            && self.compiler_fingerprint == compiler_fingerprint
            && self.config_fingerprint == config_fingerprint
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PolicyRulesArtifact {
    header: PolicyRulesArtifactHeader,
    rules: Vec<CompiledPolicyRule>,
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
    let artifact = PolicyRulesArtifact {
        header: PolicyRulesArtifactHeader::new(config_fingerprint, &compiler_fingerprint),
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
) -> Result<Vec<CompiledPolicyRule>> {
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
) -> Result<Option<CompiledPolicyRule>> {
    let default_severity = parse_severity(&definition.severity)?;
    let rule = compile_rule_ir(index, &definition.text, compiler)?;
    let normalized = normalize_rule_text(&definition.text);
    let id = rule_id(&rule, &normalized);

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
        Ok(Some(CompiledPolicyRule {
            id,
            source_text: definition.text.trim().to_string(),
            severity,
            message: default_message(&rule),
            rule,
        }))
    } else {
        Ok(None)
    }
}

fn compile_rule_ir(
    index: usize,
    text: &str,
    compiler: Option<&dyn EnglishRuleCompiler>,
) -> Result<RuleIR> {
    if let Some(rule) = parse_rule(text) {
        return Ok(rule);
    }

    if let Some(compiler) = compiler {
        return compiler.compile_rule(text)?.ok_or_else(|| {
            miette::miette!(
                "English rule compiler could not deterministically map lint.english_rules[{}]: {:?} into a supported native policy IR",
                index,
                text
            )
        });
    }

    Err(miette::miette!(
        "Unsupported english rule at lint.english_rules[{}]: {:?}. Add `api_key = \"...\"` to `.zarcrc` to enable hosted natural-language compilation into native policy IR.",
        index,
        text
    ))
}

pub fn run_compiled_rules(
    ctx: &LintContext,
    custom_rules: &[CompiledEnglishRule],
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

fn parse_rule(text: &str) -> Option<RuleIR> {
    let normalized = normalize_rule_text(text);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    parse_max_function_params(&tokens)
        .or_else(|| parse_banned_import(&tokens))
        .or_else(|| parse_banned_usage(&tokens))
        .or_else(|| parse_function_name_rule(&tokens))
        .or_else(|| parse_max_file_lines(&tokens))
        .or_else(|| parse_no_comments(&tokens))
        .or_else(|| parse_forbidden_comment_pattern(&tokens))
}

fn parse_max_function_params(tokens: &[&str]) -> Option<RuleIR> {
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
    .map(|max| RuleIR::Ast(AstRule::MaxFunctionParams {
        scope: Default::default(),
        max,
    }))
}

fn parse_banned_import(tokens: &[&str]) -> Option<RuleIR> {
    if tokens.len() >= 4 && tokens[0] == "do" && tokens[1] == "not" && tokens[2] == "import" {
        return Some(RuleIR::Import(ImportRule::BannedModulePattern {
            scope: Default::default(),
            pattern: strip_wrapping_quotes(&tokens[3..].join(" ")),
            match_kind: StringMatchKind::Exact,
        }));
    }

    if tokens.len() >= 4
        && tokens[0] == "no"
        && matches!(tokens[1], "imports" | "import")
        && tokens[2] == "from"
    {
        return Some(RuleIR::Import(ImportRule::BannedModulePattern {
            scope: Default::default(),
            pattern: strip_wrapping_quotes(&tokens[3..].join(" ")),
            match_kind: StringMatchKind::Exact,
        }));
    }

    None
}

fn parse_banned_usage(tokens: &[&str]) -> Option<RuleIR> {
    if tokens.len() >= 4 && tokens[0] == "do" && tokens[1] == "not" {
        if tokens[2] == "call" {
            return Some(RuleIR::Semantic(SemanticRule::BannedUsage {
                scope: Default::default(),
                target: strip_wrapping_quotes(&tokens[3..].join(" ")),
                require_call: true,
                require_unshadowed_root: false,
            }));
        }

        if tokens[2] == "use" {
            return Some(RuleIR::Semantic(SemanticRule::BannedUsage {
                scope: Default::default(),
                target: strip_wrapping_quotes(&tokens[3..].join(" ")),
                require_call: false,
                require_unshadowed_root: false,
            }));
        }
    }

    if tokens.len() >= 4
        && tokens[0] == "no"
        && matches!(tokens[1], "calls" | "call")
        && tokens[2] == "to"
    {
        return Some(RuleIR::Semantic(SemanticRule::BannedUsage {
            scope: Default::default(),
            target: strip_wrapping_quotes(&tokens[3..].join(" ")),
            require_call: true,
            require_unshadowed_root: false,
        }));
    }

    None
}

fn parse_function_name_rule(tokens: &[&str]) -> Option<RuleIR> {
    if tokens.len() >= 6
        && tokens[0] == "function"
        && matches!(tokens[1], "name" | "names")
        && matches!(tokens[2], "should" | "must" | "shall")
        && tokens[4] == "with"
    {
        let affix = strip_wrapping_quotes(&tokens[5..].join(" "));
        let match_kind = match tokens[3] {
            "start" => AffixMatchKind::Prefix,
            "end" => AffixMatchKind::Suffix,
            _ => return None,
        };

        return Some(RuleIR::Naming(NamingRule::Affix {
            scope: Default::default(),
            selector: NameSelector::Function,
            affix,
            match_kind,
        }));
    }

    None
}

fn parse_max_file_lines(tokens: &[&str]) -> Option<RuleIR> {
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
    .map(|max| RuleIR::File(FileRule::MaxLines {
        scope: Default::default(),
        max,
    }))
}

fn parse_no_comments(tokens: &[&str]) -> Option<RuleIR> {
    (tokens == ["no", "comments", "in", "files"]
        || tokens == ["do", "not", "allow", "comments"]
        || tokens == ["do", "not", "use", "comments"])
        .then(|| RuleIR::Comment(CommentRule::NoComments {
            scope: Default::default(),
        }))
}

fn parse_forbidden_comment_pattern(tokens: &[&str]) -> Option<RuleIR> {
    if tokens.len() >= 6
        && tokens[0] == "do"
        && tokens[1] == "not"
        && tokens[2] == "use"
        && tokens[4] == "in"
        && tokens[5] == "comments"
    {
        return Some(RuleIR::Comment(CommentRule::ForbidPattern {
            scope: Default::default(),
            pattern: strip_wrapping_quotes(tokens[3]),
        }));
    }

    None
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
            format!("Imports matching `{pattern}` are forbidden")
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
            AffixMatchKind::Prefix => format!(
                "{} names must start with `{affix}`",
                selector_name(selector)
            ),
            AffixMatchKind::Suffix => format!(
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
            PathPatternExpectation::MustMatch => {
                format!("File paths must match `{pattern}`")
            }
            PathPatternExpectation::MustNotMatch => {
                format!("File paths must not match `{pattern}`")
            }
        },
        RuleIR::Comment(CommentRule::NoComments { .. }) => {
            "Comments are not allowed in this file".to_string()
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
                format!("Calls to `{target}` are forbidden")
            } else {
                format!("Usage of `{target}` is forbidden")
            }
        }
        RuleIR::Semantic(SemanticRule::NoUnusedBindings { .. }) => {
            "Bindings must be used or prefixed with `_`".to_string()
        }
    }
}

fn forbidden_syntax_name(syntax: &super::policy_ir::ForbiddenSyntaxKind) -> &'static str {
    match syntax {
        super::policy_ir::ForbiddenSyntaxKind::TryCatch => "try/catch",
        super::policy_ir::ForbiddenSyntaxKind::Switch => "switch",
        super::policy_ir::ForbiddenSyntaxKind::DefaultExport => "default export",
        super::policy_ir::ForbiddenSyntaxKind::NestedTernary => "nested ternary",
        super::policy_ir::ForbiddenSyntaxKind::Debugger => "debugger",
    }
}

fn selector_name(selector: &NameSelector) -> &'static str {
    match selector {
        NameSelector::Function => "Function",
        NameSelector::Variable => "Variable",
        NameSelector::Class => "Class",
    }
}

fn case_style_name(style: &super::policy_ir::CaseStyle) -> &'static str {
    match style {
        super::policy_ir::CaseStyle::CamelCase => "camelCase",
        super::policy_ir::CaseStyle::PascalCase => "PascalCase",
        super::policy_ir::CaseStyle::SnakeCase => "snake_case",
        super::policy_ir::CaseStyle::KebabCase => "kebab-case",
        super::policy_ir::CaseStyle::UpperSnakeCase => "UPPER_SNAKE_CASE",
    }
}

fn rule_id(rule: &RuleIR, normalized_text: &str) -> String {
    let kind = rule.kind_slug();
    let digest = hash_bytes(format!("{kind}:{normalized_text}").as_bytes());
    format!("policy/{kind}/{}", &digest[..10])
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

fn compiler_fingerprint(compiler: Option<&dyn EnglishRuleCompiler>) -> String {
    let compiler_mode = compiler
        .map(|compiler| compiler.fingerprint_material())
        .unwrap_or_else(|| "manual-only".to_string());
    hash_bytes(
        format!(
            "{POLICY_RULE_ARTIFACT_SCHEMA_VERSION}:{POLICY_RULE_COMPILER_VERSION}:{compiler_mode}"
        )
        .as_bytes(),
    )
}

fn load_artifact(
    path: &Path,
    config_fingerprint: &str,
    compiler_fingerprint: &str,
) -> Result<Option<PolicyRulesArtifact>> {
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read(path)
        .map_err(|error| miette::miette!("Failed to read policy-rule artifact: {}", error))?;
    let artifact = match serde_json::from_slice::<PolicyRulesArtifact>(&data) {
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

fn persist_artifact(path: &Path, artifact: &PolicyRulesArtifact) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                miette::miette!(
                    "Failed to create policy-rule cache directory {}: {}",
                    parent.display(),
                    error
                )
            })?;
        }
    }

    let data = serde_json::to_vec(artifact)
        .map_err(|error| miette::miette!("Failed to serialize policy-rule artifact: {}", error))?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("zarc-cache.rules.json");
    let temp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));

    fs::write(&temp_path, data)
        .map_err(|error| miette::miette!("Failed to write policy-rule artifact: {}", error))?;

    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| miette::miette!("Failed to replace policy-rule artifact: {}", error))?;
    }

    fs::rename(&temp_path, path)
        .map_err(|error| miette::miette!("Failed to finalize policy-rule artifact: {}", error))?;
    Ok(())
}
