pub mod dead_code;
pub mod no_console;
pub mod no_empty_catch;
pub mod no_explicit_any;
pub mod no_missing_return;
pub mod no_unused_vars;
pub mod no_unsafe_optional_access;
pub mod no_wrong_arg_count;
pub mod policy;
pub mod policy_ir;
pub mod prefer_const;
pub mod unreachable_code;

use miette::Result;
use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

// ── Core types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LintResult {
    pub file: PathBuf,
    pub diagnostics: Vec<LintDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LintDiagnostic {
    pub rule_name: String,
    pub message: String,
    pub span: String, // "line:col"
    pub severity: Severity,
    pub origin: RuleOrigin,
    pub fix: Option<Fix>,
    #[serde(default)]
    pub byte_start: u32,
    #[serde(default)]
    pub byte_end: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fix {
    pub range: (usize, usize),
    pub replacement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub safety: FixSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FixSafety {
    #[default]
    Safe,
    SemanticSafe,
    Risky,
    SuppressOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleOrigin {
    BuiltIn,
    Config,
    Engine,
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
        self.diagnostic_with_origin(rule_name, message, span, severity, RuleOrigin::BuiltIn)
    }

    pub fn diagnostic_with_origin(
        &self,
        rule_name: impl Into<String>,
        message: impl Into<String>,
        span: oxc_span::Span,
        severity: Severity,
        origin: RuleOrigin,
    ) -> LintDiagnostic {
        let (line, col) = self.offset_to_line_col(span.start as usize);
        LintDiagnostic {
            rule_name: rule_name.into(),
            message: message.into(),
            span: format!("{line}:{col}"),
            severity,
            origin,
            fix: None,
            byte_start: span.start,
            byte_end: span.end,
            node_kind: None,
            symbol: None,
        }
    }

    pub fn diagnostic_with_context(
        &self,
        rule_name: impl Into<String>,
        message: impl Into<String>,
        span: oxc_span::Span,
        severity: Severity,
        origin: RuleOrigin,
        node_kind: Option<String>,
        symbol: Option<String>,
    ) -> LintDiagnostic {
        let (line, col) = self.offset_to_line_col(span.start as usize);
        LintDiagnostic {
            rule_name: rule_name.into(),
            message: message.into(),
            span: format!("{line}:{col}"),
            severity,
            origin,
            fix: None,
            byte_start: span.start,
            byte_end: span.end,
            node_kind,
            symbol,
        }
    }
}

// ── Lint engine ─────────────────────────────────────────────

/// Returns all built-in rules (used when no config is present)
fn all_builtin_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(no_explicit_any::NoExplicitAny),
        Box::new(no_console::NoConsole),
        Box::new(no_empty_catch::NoEmptyCatch),
        Box::new(prefer_const::PreferConst),
        Box::new(no_unused_vars::NoUnusedVars),
        Box::new(unreachable_code::UnreachableCode),
        Box::new(no_missing_return::NoMissingReturn),
        Box::new(no_wrong_arg_count::NoWrongArgCount),
        Box::new(no_unsafe_optional_access::NoUnsafeOptionalAccess),
    ]
}

/// Returns only the built-in rules that are enabled in config
pub fn enabled_builtin_rules(config: &HashMap<String, serde_json::Value>) -> Vec<Box<dyn LintRule>> {
    let all = all_builtin_rules();
    if config.is_empty() {
        return all;
    }

    all.into_iter()
        .filter(|rule| {
            config
                .get(rule.name())
                .map(|v| crate::cli::parse_rule_severity(v).is_some())
                .unwrap_or(false)
        })
        .collect()
}

/// Get severity override for a built-in rule from config
pub fn get_severity_override(
    rule_name: &str,
    config: &HashMap<String, serde_json::Value>,
) -> Option<Severity> {
    config
        .get(rule_name)
        .and_then(|v| crate::cli::parse_rule_severity(v))
}

/// Lint a single file with config-driven rules
pub fn lint_file_with_config(
    path: &Path,
    config: &HashMap<String, serde_json::Value>,
) -> Result<LintResult> {
    let source = fs::read_to_string(path)
        .map_err(|e| miette::miette!("Failed to read {}: {}", path.display(), e))?;
    Ok(lint_source_with_config(path, &source, config))
}

pub fn lint_source_with_config(
    path: &Path,
    source: &str,
    config: &HashMap<String, serde_json::Value>,
) -> LintResult {
    let source_type = SourceType::from_path(path).unwrap_or_default();

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);

    let ctx = LintContext::new(source, path, &semantic.semantic);

    let builtin_rules = enabled_builtin_rules(config);
    let policy_rules = policy::build_all_policy_rules(config);

    let mut diagnostics = parser_diagnostics_to_lints(source, &parsed.errors);
    diagnostics.extend(semantic_diagnostics_to_lints(
        source,
        &semantic.errors,
        path,
    ));

    // Run built-in rules with severity overrides
    for rule in &builtin_rules {
        let mut rule_diagnostics = rule.run(&ctx);
        if let Some(severity) = get_severity_override(rule.name(), config) {
            for d in &mut rule_diagnostics {
                d.severity = severity.clone();
            }
        }
        diagnostics.extend(rule_diagnostics);
    }

    // Run config-driven policy rules
    diagnostics.extend(policy::run_compiled_rules(&ctx, &policy_rules));

    LintResult {
        file: path.to_path_buf(),
        diagnostics,
    }
}

// Keep backward-compatible function for existing callers
pub fn lint_file(path: &Path) -> Result<LintResult> {
    lint_file_with_config(path, &HashMap::new())
}

pub struct HashedSource {
    pub source: String,
    pub hash: String,
    pub size: u64,
}

pub fn load_source_with_hash(path: &Path) -> Result<HashedSource> {
    let bytes =
        fs::read(path).map_err(|e| miette::miette!("Failed to read {}: {}", path.display(), e))?;
    let hash = hash_bytes(&bytes);
    let size = bytes.len() as u64;
    let source = String::from_utf8(bytes)
        .map_err(|e| miette::miette!("Failed to decode {} as UTF-8: {}", path.display(), e))?;

    Ok(HashedSource { source, hash, size })
}

#[allow(dead_code)]
pub(crate) fn lint_source_at_path(path: &Path, source: &str) -> LintResult {
    lint_source_with_config(path, source, &HashMap::new())
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
                origin: RuleOrigin::Engine,
                fix: None,
                byte_start: offset as u32,
                byte_end: offset as u32,
                node_kind: None,
                symbol: None,
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

#[cfg(test)]
pub(crate) fn lint_source_for_test(path: &str, source: &str) -> LintResult {
    lint_source_at_path(Path::new(path), source)
}

// ── File hashing for cache ──────────────────────────────────

pub const CACHE_SCHEMA_VERSION: u32 = 4; // Bumped for new diagnostic fields
pub const CACHE_LOGIC_VERSION: u32 = 1;
const MAX_TIMING_SAMPLES: usize = 8;

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn hash_file(path: &Path) -> String {
    let content = fs::read(path).unwrap_or_default();
    hash_bytes(&content)
}

// ── Cache ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileFingerprint {
    pub size: u64,
    pub modified_ns: Option<u128>,
}

impl FileFingerprint {
    pub fn from_path(path: &Path) -> Result<Option<Self>> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(Some(Self::from_metadata(&metadata))),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(miette::miette!(
                "Failed to read metadata for {}: {}",
                path.display(),
                error
            )),
        }
    }

    fn from_metadata(metadata: &fs::Metadata) -> Self {
        let modified_ns = metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());

        Self {
            size: metadata.len(),
            modified_ns,
        }
    }

    pub fn matches(&self, other: &Self) -> bool {
        self.size == other.size && self.modified_ns == other.modified_ns
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RollingSamples {
    pub values: Vec<u64>,
}

impl RollingSamples {
    pub fn observe(&mut self, value: u64) {
        if value == 0 {
            return;
        }

        self.values.push(value);
        if self.values.len() > MAX_TIMING_SAMPLES {
            self.values.remove(0);
        }
    }

    pub fn median(&self) -> Option<u64> {
        if self.values.is_empty() {
            return None;
        }

        let mut sorted = self.values.clone();
        sorted.sort_unstable();
        Some(sorted[sorted.len() / 2])
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheTimings {
    pub load_ns: RollingSamples,
    pub save_ns: RollingSamples,
    pub stat_per_file_ns: RollingSamples,
    pub hash_per_byte_ns: RollingSamples,
    pub lint_per_byte_ns: RollingSamples,
}

impl CacheTimings {
    pub fn ready_for_prediction(&self) -> bool {
        self.load_ns.count() >= 3
            && self.stat_per_file_ns.count() >= 3
            && self.lint_per_byte_ns.count() >= 3
    }

    pub fn record_load(&mut self, duration_ns: u64) {
        self.load_ns.observe(duration_ns);
    }

    pub fn record_save(&mut self, duration_ns: u64) {
        self.save_ns.observe(duration_ns);
    }

    pub fn record_stat(&mut self, duration_ns: u64, files: usize) {
        if files == 0 {
            return;
        }

        self.stat_per_file_ns
            .observe(duration_ns.saturating_div(files as u64));
    }

    pub fn record_hash(&mut self, duration_ns: u64, bytes: u64) {
        if bytes == 0 {
            return;
        }

        self.hash_per_byte_ns
            .observe(duration_ns.saturating_div(bytes.max(1)));
    }

    pub fn record_lint(&mut self, duration_ns: u64, bytes: u64) {
        if bytes == 0 {
            return;
        }

        self.lint_per_byte_ns
            .observe(duration_ns.saturating_div(bytes.max(1)));
    }

    pub fn predict_load(&self) -> Option<u64> {
        self.load_ns.median()
    }

    pub fn predict_save(&self) -> Option<u64> {
        self.save_ns.median()
    }

    pub fn predict_stat(&self, files: usize) -> Option<u64> {
        self.stat_per_file_ns
            .median()
            .map(|rate| scale_cost(rate, files as u64))
    }

    pub fn predict_hash(&self, bytes: u64) -> Option<u64> {
        self.hash_per_byte_ns
            .median()
            .map(|rate| scale_cost(rate, bytes))
    }

    pub fn predict_lint(&self, bytes: u64) -> Option<u64> {
        self.lint_per_byte_ns
            .median()
            .map(|rate| scale_cost(rate, bytes))
    }
}

fn scale_cost(rate: u64, units: u64) -> u64 {
    if rate == 0 || units == 0 {
        return 0;
    }

    let total = u128::from(rate) * u128::from(units);
    total.min(u128::from(u64::MAX)) as u64
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheHeader {
    pub schema_version: u32,
    pub logic_version: u32,
    pub config_fingerprint: String,
    pub timings: CacheTimings,
}

impl CacheHeader {
    fn new(config_fingerprint: &str) -> Self {
        Self {
            schema_version: CACHE_SCHEMA_VERSION,
            logic_version: CACHE_LOGIC_VERSION,
            config_fingerprint: config_fingerprint.to_string(),
            timings: CacheTimings::default(),
        }
    }

    fn matches(&self, config_fingerprint: &str) -> bool {
        self.schema_version == CACHE_SCHEMA_VERSION
            && self.logic_version == CACHE_LOGIC_VERSION
            && self.config_fingerprint == config_fingerprint
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheEntry {
    pub fingerprint: FileFingerprint,
    pub hash: String,
    pub result: LintResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    pub header: CacheHeader,
    pub entries: HashMap<PathBuf, CacheEntry>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLoadStatus {
    Missing,
    Loaded,
    Invalidated,
}

impl Cache {
    pub fn new(config_fingerprint: &str) -> Self {
        Self {
            header: CacheHeader::new(config_fingerprint),
            entries: HashMap::new(),
        }
    }

    pub fn load(path: &Path, config_fingerprint: &str) -> Result<(Self, CacheLoadStatus)> {
        if !path.exists() {
            return Ok((Self::new(config_fingerprint), CacheLoadStatus::Missing));
        }

        let data = fs::read(path).map_err(|e| miette::miette!("Failed to read cache: {}", e))?;
        let cache = match serde_json::from_slice::<Self>(&data) {
            Ok(cache) => cache,
            Err(_) => {
                return Ok((Self::new(config_fingerprint), CacheLoadStatus::Invalidated));
            }
        };

        if cache.header.matches(config_fingerprint) {
            Ok((cache, CacheLoadStatus::Loaded))
        } else {
            Ok((Self::new(config_fingerprint), CacheLoadStatus::Invalidated))
        }
    }

    pub fn get(&self, path: &Path) -> Option<&CacheEntry> {
        self.entries.get(path)
    }

    pub fn update_fingerprint(&mut self, config_fingerprint: &str) {
        if self.header.config_fingerprint != config_fingerprint {
            self.header.config_fingerprint = config_fingerprint.to_string();
        }
    }

    pub fn upsert(
        &mut self,
        path: PathBuf,
        fingerprint: FileFingerprint,
        hash: String,
        result: LintResult,
    ) -> bool {
        let next = CacheEntry {
            fingerprint,
            hash,
            result,
        };
        let changed = self.entries.get(&path) != Some(&next);
        self.entries.insert(path, next);
        changed
    }

    pub fn prune_to(&mut self, live_paths: &HashSet<PathBuf>) -> bool {
        let before = self.entries.len();
        self.entries.retain(|path, _| live_paths.contains(path));
        before != self.entries.len()
    }

    pub fn persist(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| {
                    miette::miette!(
                        "Failed to create cache directory {}: {}",
                        parent.display(),
                        e
                    )
                })?;
            }
        }

        let data = serde_json::to_vec(self)
            .map_err(|e| miette::miette!("Failed to serialize cache: {}", e))?;
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("flint-cache.json");
        let temp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));

        fs::write(&temp_path, data).map_err(|e| miette::miette!("Failed to write cache: {}", e))?;

        if path.exists() {
            fs::remove_file(path).map_err(|e| miette::miette!("Failed to replace cache: {}", e))?;
        }

        fs::rename(&temp_path, path)
            .map_err(|e| miette::miette!("Failed to finalize cache: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cache_round_trips_for_matching_fingerprint() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cache.json");
        let file = dir.path().join("demo.ts");

        fs::write(&file, "const value: any = 1;\n").unwrap();

        let mut cache = Cache::new("fingerprint-a");
        cache.upsert(
            file.clone(),
            FileFingerprint::from_path(&file).unwrap().unwrap(),
            hash_file(&file),
            lint_source_for_test("demo.ts", "const value: any = 1;\n"),
        );
        cache.persist(&path).unwrap();

        let (loaded, status) = Cache::load(&path, "fingerprint-a").unwrap();

        assert_eq!(status, CacheLoadStatus::Loaded);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.header.config_fingerprint, "fingerprint-a");
    }

    #[test]
    fn cache_invalidates_on_fingerprint_change() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cache.json");
        let file = dir.path().join("demo.ts");

        fs::write(&file, "const value = 1;\n").unwrap();

        let mut cache = Cache::new("fingerprint-a");
        cache.upsert(
            file.clone(),
            FileFingerprint::from_path(&file).unwrap().unwrap(),
            hash_file(&file),
            lint_source_for_test("demo.ts", "const value = 1;\n"),
        );
        cache.persist(&path).unwrap();

        let (loaded, status) = Cache::load(&path, "fingerprint-b").unwrap();

        assert_eq!(status, CacheLoadStatus::Invalidated);
        assert!(loaded.entries.is_empty());
        assert_eq!(loaded.header.config_fingerprint, "fingerprint-b");
    }

    #[test]
    fn prune_to_removes_missing_paths() {
        let file_a = PathBuf::from("a.ts");
        let file_b = PathBuf::from("b.ts");
        let mut cache = Cache::new("fingerprint");

        cache.upsert(
            file_a.clone(),
            FileFingerprint {
                size: 1,
                modified_ns: Some(1),
            },
            "hash-a".to_string(),
            LintResult {
                file: file_a.clone(),
                diagnostics: Vec::new(),
            },
        );
        cache.upsert(
            file_b.clone(),
            FileFingerprint {
                size: 1,
                modified_ns: Some(1),
            },
            "hash-b".to_string(),
            LintResult {
                file: file_b.clone(),
                diagnostics: Vec::new(),
            },
        );

        let live = HashSet::from([file_a.clone()]);
        let changed = cache.prune_to(&live);

        assert!(changed);
        assert!(cache.entries.contains_key(&file_a));
        assert!(!cache.entries.contains_key(&file_b));
    }
}
