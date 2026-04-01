use super::Severity;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledPolicyRule {
    pub id: String,
    pub source_text: String,
    pub severity: Severity,
    pub message: String,
    pub rule: RuleIR,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "category", content = "rule", rename_all = "kebab-case")]
pub enum RuleIR {
    Ast(AstRule),
    Import(ImportRule),
    Naming(NamingRule),
    File(FileRule),
    Comment(CommentRule),
    Semantic(SemanticRule),
}

impl RuleIR {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            RuleIR::Ast(rule) => rule.kind_slug(),
            RuleIR::Import(rule) => rule.kind_slug(),
            RuleIR::Naming(rule) => rule.kind_slug(),
            RuleIR::File(rule) => rule.kind_slug(),
            RuleIR::Comment(rule) => rule.kind_slug(),
            RuleIR::Semantic(rule) => rule.kind_slug(),
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            RuleIR::Ast(rule) => rule.scope(),
            RuleIR::Import(rule) => rule.scope(),
            RuleIR::Naming(rule) => rule.scope(),
            RuleIR::File(rule) => rule.scope(),
            RuleIR::Comment(rule) => rule.scope(),
            RuleIR::Semantic(rule) => rule.scope(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleScope {
    #[serde(default)]
    pub include_paths: Vec<String>,
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
}

impl RuleScope {
    pub fn matches_path(&self, file_path: &Path) -> bool {
        let normalized_path = normalize_path(file_path);
        let includes_match = self.include_paths.is_empty()
            || self
                .include_paths
                .iter()
                .any(|pattern| path_pattern_matches(pattern, &normalized_path));
        if !includes_match {
            return false;
        }

        if self
            .exclude_paths
            .iter()
            .any(|pattern| path_pattern_matches(pattern, &normalized_path))
        {
            return false;
        }

        if self.extensions.is_empty() {
            return true;
        }

        file_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                self.extensions.iter().any(|candidate| {
                    candidate.trim_start_matches('.').eq_ignore_ascii_case(extension)
                })
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AstRule {
    MaxFunctionParams {
        #[serde(default)]
        scope: RuleScope,
        max: usize,
    },
    ForbiddenSyntax {
        #[serde(default)]
        scope: RuleScope,
        syntax: ForbiddenSyntaxKind,
    },
}

impl AstRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            AstRule::MaxFunctionParams { .. } => "ast/max-function-params",
            AstRule::ForbiddenSyntax { .. } => "ast/forbidden-syntax",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            AstRule::MaxFunctionParams { scope, .. } | AstRule::ForbiddenSyntax { scope, .. } => {
                scope
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ForbiddenSyntaxKind {
    TryCatch,
    Switch,
    DefaultExport,
    NestedTernary,
    Debugger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ImportRule {
    BannedModulePattern {
        #[serde(default)]
        scope: RuleScope,
        pattern: String,
        #[serde(default)]
        match_kind: StringMatchKind,
    },
    NoSideEffectImport {
        #[serde(default)]
        scope: RuleScope,
    },
}

impl ImportRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            ImportRule::BannedModulePattern { .. } => "import/banned-module-pattern",
            ImportRule::NoSideEffectImport { .. } => "import/no-side-effect-import",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            ImportRule::BannedModulePattern { scope, .. }
            | ImportRule::NoSideEffectImport { scope } => scope,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum StringMatchKind {
    #[default]
    Exact,
    Prefix,
    Suffix,
    Contains,
    Glob,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum NamingRule {
    Affix {
        #[serde(default)]
        scope: RuleScope,
        selector: NameSelector,
        affix: String,
        match_kind: AffixMatchKind,
    },
    Case {
        #[serde(default)]
        scope: RuleScope,
        selector: NameSelector,
        style: CaseStyle,
    },
}

impl NamingRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            NamingRule::Affix { .. } => "naming/affix",
            NamingRule::Case { .. } => "naming/case",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            NamingRule::Affix { scope, .. } | NamingRule::Case { scope, .. } => scope,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NameSelector {
    Function,
    Variable,
    Class,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AffixMatchKind {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CaseStyle {
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    UpperSnakeCase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FileRule {
    MaxLines {
        #[serde(default)]
        scope: RuleScope,
        max: usize,
    },
    PathPattern {
        #[serde(default)]
        scope: RuleScope,
        pattern: String,
        expectation: PathPatternExpectation,
    },
}

impl FileRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            FileRule::MaxLines { .. } => "file/max-lines",
            FileRule::PathPattern { .. } => "file/path-pattern",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            FileRule::MaxLines { scope, .. } | FileRule::PathPattern { scope, .. } => scope,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PathPatternExpectation {
    MustMatch,
    MustNotMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CommentRule {
    NoComments {
        #[serde(default)]
        scope: RuleScope,
    },
    ForbidPattern {
        #[serde(default)]
        scope: RuleScope,
        pattern: String,
    },
}

impl CommentRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            CommentRule::NoComments { .. } => "comment/no-comments",
            CommentRule::ForbidPattern { .. } => "comment/forbid-pattern",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            CommentRule::NoComments { scope } | CommentRule::ForbidPattern { scope, .. } => scope,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SemanticRule {
    BannedUsage {
        #[serde(default)]
        scope: RuleScope,
        target: String,
        require_call: bool,
        #[serde(default)]
        require_unshadowed_root: bool,
    },
    NoUnusedBindings {
        #[serde(default)]
        scope: RuleScope,
    },
}

impl SemanticRule {
    pub fn kind_slug(&self) -> &'static str {
        match self {
            SemanticRule::BannedUsage { .. } => "semantic/banned-usage",
            SemanticRule::NoUnusedBindings { .. } => "semantic/no-unused-bindings",
        }
    }

    pub fn scope(&self) -> &RuleScope {
        match self {
            SemanticRule::BannedUsage { scope, .. } | SemanticRule::NoUnusedBindings { scope } => {
                scope
            }
        }
    }
}

fn normalize_path(file_path: &Path) -> String {
    file_path.to_string_lossy().replace('\\', "/")
}

fn path_pattern_matches(pattern: &str, normalized_path: &str) -> bool {
    let normalized_pattern = pattern.replace('\\', "/");
    glob_match(&normalized_pattern, normalized_path)
        || glob_match(&format!("**/{}", normalized_pattern.trim_start_matches('/')), normalized_path)
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
