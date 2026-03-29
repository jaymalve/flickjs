pub mod no_explicit_any;
pub mod no_unused_vars;
pub mod no_console;
pub mod no_empty_catch;
pub mod prefer_const;

use miette::Result;
use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Core types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub file: PathBuf,
    pub diagnostics: Vec<LintDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintDiagnostic {
    pub rule_name: String,
    pub message: String,
    pub span: String, // "line:col"
    pub severity: Severity,
    pub fix: Option<Fix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub range: (usize, usize),
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

// ── Rule trait ──────────────────────────────────────────────

/// Every lint rule implements this trait.
pub trait LintRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, ctx: &LintContext) -> Vec<LintDiagnostic>;
}

/// Context passed to each rule.
pub struct LintContext<'a> {
    pub source: &'a str,
    pub file_path: &'a Path,
    pub source_type: SourceType,
    pub semantic: &'a Semantic<'a>,
}

impl<'a> LintContext<'a> {
    pub fn new(source: &'a str, file_path: &'a Path, semantic: &'a Semantic<'a>) -> Self {
        Self {
            source,
            file_path,
            source_type: *semantic.source_type(),
            semantic,
        }
    }

    /// Get line and column for a byte offset
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub fn diagnostic(
        &self,
        rule_name: &'static str,
        message: impl Into<String>,
        span: oxc_span::Span,
        severity: Severity,
    ) -> LintDiagnostic {
        let (line, col) = self.offset_to_line_col(span.start as usize);
        LintDiagnostic {
            rule_name: rule_name.to_string(),
            message: message.into(),
            span: format!("{line}:{col}"),
            severity,
            fix: None,
        }
    }
}

// ── Lint engine ─────────────────────────────────────────────

/// Returns all enabled built-in rules
fn builtin_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(no_explicit_any::NoExplicitAny),
        Box::new(no_console::NoConsole),
        Box::new(no_empty_catch::NoEmptyCatch),
        Box::new(prefer_const::PreferConst),
        Box::new(no_unused_vars::NoUnusedVars),
    ]
}

/// Lint a single file — parse once, then run all built-in rules
pub fn lint_file(path: &Path) -> Result<LintResult> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("Failed to read {}: {}", path.display(), e))?;

    let source_type = SourceType::from_path(path).unwrap_or_default();

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);

    let ctx = LintContext::new(&source, path, &semantic.semantic);

    let rules = builtin_rules();
    let mut diagnostics = parser_diagnostics_to_lints(&source, &parsed.errors);
    diagnostics.extend(semantic_diagnostics_to_lints(
        &source,
        &semantic.errors,
        path,
    ));
    diagnostics.extend(rules
        .iter()
        .flat_map(|rule| rule.run(&ctx))
        .collect::<Vec<_>>());

    Ok(LintResult {
        file: path.to_path_buf(),
        diagnostics,
    })
}

fn parser_diagnostics_to_lints(source: &str, errors: &[OxcDiagnostic]) -> Vec<LintDiagnostic> {
    diagnostics_to_lints(source, errors, "parse-error", Severity::Error)
}

fn semantic_diagnostics_to_lints(
    source: &str,
    errors: &[OxcDiagnostic],
    _path: &Path,
) -> Vec<LintDiagnostic> {
    diagnostics_to_lints(source, errors, "semantic-error", Severity::Error)
}

fn diagnostics_to_lints(
    source: &str,
    errors: &[OxcDiagnostic],
    rule_name: &'static str,
    severity: Severity,
) -> Vec<LintDiagnostic> {
    errors
        .iter()
        .map(|error: &OxcDiagnostic| {
            let offset = error
                .labels
                .as_ref()
                .into_iter()
                .flatten()
                .next()
                .map(|label| label.offset())
                .unwrap_or(0);
            let (line, col) = offset_to_line_col(source, offset);
            LintDiagnostic {
                rule_name: rule_name.to_string(),
                message: error.to_string(),
                span: format!("{line}:{col}"),
                severity: severity.clone(),
                fix: None,
            }
        })
        .collect()
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn apply_severity_overrides(
    mut result: LintResult,
    overrides: &HashMap<String, String>,
) -> LintResult {
    result.diagnostics = result
        .diagnostics
        .into_iter()
        .filter_map(|mut diagnostic| match overrides.get(&diagnostic.rule_name) {
            Some(level) if level.eq_ignore_ascii_case("off") => None,
            Some(level) if level.eq_ignore_ascii_case("error") => {
                diagnostic.severity = Severity::Error;
                Some(diagnostic)
            }
            Some(level) if level.eq_ignore_ascii_case("warn") => {
                diagnostic.severity = Severity::Warning;
                Some(diagnostic)
            }
            _ => Some(diagnostic),
        })
        .collect();
    result
}

// ── File hashing for cache ──────────────────────────────────

pub fn hash_file(path: &Path) -> String {
    let content = std::fs::read(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&content);
    format!("{:x}", hasher.finalize())
}

// ── Cache ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub result: LintResult,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Cache {
    pub entries: HashMap<PathBuf, CacheEntry>,
}

impl Cache {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)
                .map_err(|e| miette::miette!("Failed to read cache: {}", e))?;
            serde_json::from_str(&data)
                .map_err(|e| miette::miette!("Failed to parse cache: {}", e))
        } else {
            Ok(Self::default())
        }
    }

    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    pub fn save(&self, results: &[LintResult], path: &Path) -> Result<()> {
        let mut cache = self.entries.clone();
        for result in results {
            cache.insert(
                result.file.clone(),
                CacheEntry {
                    hash: hash_file(&result.file),
                    result: result.clone(),
                },
            );
        }
        let data = serde_json::to_string_pretty(&Cache { entries: cache })
            .map_err(|e| miette::miette!("Failed to serialize cache: {}", e))?;
        std::fs::write(path, data)
            .map_err(|e| miette::miette!("Failed to write cache: {}", e))?;
        Ok(())
    }
}
