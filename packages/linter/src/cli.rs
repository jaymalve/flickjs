use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "flint",
    about = "⚡ Flint — The JavaScript Static Analysis Engine",
    version,
    after_help = "Examples:\n  flint check ./src\n  flint check . --format agent-json\n  flint init"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Show timing information
    #[arg(long, global = true)]
    pub timing: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Lint JavaScript and TypeScript files
    Check(CheckArgs),
    /// Initialize flint config in current directory
    Init,
}

#[derive(clap::Args, Clone)]
pub struct CheckArgs {
    /// Path to lint (file or directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Path to cache file
    #[arg(long, default_value = ".flint-cache.json")]
    pub cache_path: PathBuf,

    /// Additional patterns to ignore
    #[arg(long)]
    pub ignore: Vec<String>,

    /// Disable the cache for this run
    #[arg(long)]
    pub no_cache: bool,

    /// Output format
    #[arg(long, default_value = "pretty")]
    pub format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Pretty,
    Json,
    Compact,
    AgentJson,
}

// ── Config types ───────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rules: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_excludes")]
    pub exclude: Vec<String>,
}

pub struct LoadedConfig {
    pub config: Config,
    pub fingerprint: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rules: default_rules(),
            files: FilesConfig::default(),
        }
    }
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            exclude: default_excludes(),
        }
    }
}

fn default_excludes() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".git".to_string(),
    ]
}

fn default_rules() -> HashMap<String, serde_json::Value> {
    let mut rules = HashMap::new();
    rules.insert("no-explicit-any".into(), serde_json::Value::String("warn".into()));
    rules.insert("no-unused-vars".into(), serde_json::Value::String("error".into()));
    rules.insert("no-console".into(), serde_json::Value::String("warn".into()));
    rules.insert("prefer-const".into(), serde_json::Value::String("warn".into()));
    rules.insert("no-empty-catch".into(), serde_json::Value::String("error".into()));
    rules
}

pub fn load_config() -> Result<Config> {
    Ok(load_config_with_fingerprint()?.config)
}

pub fn load_config_with_fingerprint() -> Result<LoadedConfig> {
    let path = Path::new("flint.json");
    if !path.exists() {
        return Ok(LoadedConfig {
            config: Config::default(),
            fingerprint: hash_string("__default__"),
        });
    }

    let raw = std::fs::read_to_string(path).into_diagnostic()?;
    let config: Config = serde_json::from_str(&raw)
        .map_err(|e| miette::miette!("Failed to parse flint.json: {}", e))?;

    Ok(LoadedConfig {
        config,
        fingerprint: hash_string(&raw),
    })
}

fn hash_string(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Discover all JS/TS files in a path, respecting .gitignore
pub fn discover_files(
    path: &Path,
    excludes: &[String],
    extra_ignores: &[String],
) -> Result<Vec<PathBuf>> {
    use ignore::WalkBuilder;

    let mut builder = WalkBuilder::new(path);
    builder.hidden(true).git_ignore(true).git_global(true);
    let ignore_patterns: Vec<&str> = excludes
        .iter()
        .chain(extra_ignores.iter())
        .map(String::as_str)
        .collect();

    let files: Vec<PathBuf> = builder
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .filter(|entry| !is_ignored_path(entry.path(), &ignore_patterns))
        .filter(|entry| {
            let path = entry.path();
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts")
            )
        })
        .map(|entry| entry.into_path())
        .collect();

    Ok(files)
}

fn is_ignored_path(path: &Path, patterns: &[&str]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| {
        path.components()
            .any(|component| component.as_os_str() == *pattern)
            || path_str.contains(pattern)
    })
}

/// Initialize a flint.json config file
pub fn init_config() -> Result<()> {
    use colored::*;

    let config = r#"{
  "$schema": "https://flickjs.dev/lint/schema.json",
  "rules": {
    "no-explicit-any": "warn",
    "no-unused-vars": "error",
    "no-console": "warn",
    "prefer-const": "warn",
    "no-empty-catch": "error",
    "no-debugger": true,
    "no-var": true,
    "no-nested-ternaries": true,
    "no-default-export": true,
    "no-switch": false,
    "no-type-assertion": false,
    "no-await-in-loops": true,
    "max-function-params": 4,
    "max-file-lines": 500,
    "naming-functions": "camelCase",
    "naming-classes": "PascalCase",
    "banned-imports": [],
    "banned-calls": [],
    "unreachable-code": "error",
    "unused-exports": "warn",
    "unused-files": "warn",
    "unused-dependencies": "warn",
    "no-missing-return": "error",
    "no-wrong-arg-count": "error",
    "no-unsafe-optional-access": "error"
  },
  "files": {
    "exclude": ["node_modules", "dist", "build", ".git"]
  }
}
"#;

    let path = Path::new("flint.json");
    if path.exists() {
        eprintln!("{} flint.json already exists", "warning:".yellow().bold());
        return Ok(());
    }

    std::fs::write(path, config).into_diagnostic()?;
    println!("{} Created flint.json", "✓".green().bold());
    println!(
        "  Edit the config and run {} to start linting",
        "flint check".cyan()
    );

    Ok(())
}

/// Parse a rule config value into an optional severity.
/// Returns Ok(None) if the rule is disabled.
pub fn parse_rule_severity(value: &serde_json::Value) -> Option<super::rules::Severity> {
    match value {
        serde_json::Value::Bool(false) => None,
        serde_json::Value::Bool(true) => Some(super::rules::Severity::Warning),
        serde_json::Value::String(s) if s.eq_ignore_ascii_case("off") => None,
        serde_json::Value::String(s) if s.eq_ignore_ascii_case("warn") => {
            Some(super::rules::Severity::Warning)
        }
        serde_json::Value::String(s) if s.eq_ignore_ascii_case("error") => {
            Some(super::rules::Severity::Error)
        }
        // Numbers, arrays, and other strings mean enabled (warn by default)
        serde_json::Value::Number(_) => Some(super::rules::Severity::Warning),
        serde_json::Value::Array(_) => Some(super::rules::Severity::Warning),
        serde_json::Value::String(_) => Some(super::rules::Severity::Warning),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_five_rules() {
        let config = Config::default();
        assert_eq!(config.rules.len(), 5);
        assert!(config.rules.contains_key("no-console"));
    }

    #[test]
    fn parse_severity_from_bool() {
        assert!(parse_rule_severity(&serde_json::json!(true)).is_some());
        assert!(parse_rule_severity(&serde_json::json!(false)).is_none());
    }

    #[test]
    fn parse_severity_from_string() {
        assert_eq!(
            parse_rule_severity(&serde_json::json!("error")),
            Some(super::super::rules::Severity::Error)
        );
        assert_eq!(
            parse_rule_severity(&serde_json::json!("warn")),
            Some(super::super::rules::Severity::Warning)
        );
        assert!(parse_rule_severity(&serde_json::json!("off")).is_none());
    }

    #[test]
    fn parse_severity_from_number() {
        assert!(parse_rule_severity(&serde_json::json!(3)).is_some());
    }
}
