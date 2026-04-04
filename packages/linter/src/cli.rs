use crate::project::ProjectInfo;
use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
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
    /// Browse Flint's built-in rules in a terminal UI
    Rules(RulesArgs),
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

#[derive(clap::Args, Clone, Debug, Default)]
pub struct RulesArgs {
    /// Open the rule browser focused on a specific group
    #[arg(long)]
    pub group: Option<String>,

    /// Start the rule browser with a search query applied
    #[arg(long)]
    pub search: Option<String>,
}

// ── Config types ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_detect")]
    pub detect: bool,
    #[serde(default)]
    pub rules: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_excludes")]
    pub exclude: Vec<String>,
}

/// Separate deserialization type for flint.json — rules default to empty
/// but detection may still enable built-ins when `detect` is true.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileConfig {
    #[serde(default = "default_detect")]
    pub detect: bool,
    #[serde(default)]
    pub rules: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub files: FilesConfig,
}

pub struct LoadedConfig {
    pub config: Config,
    pub fingerprint: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            detect: default_detect(),
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

fn default_detect() -> bool {
    true
}

fn default_rules() -> HashMap<String, serde_json::Value> {
    let mut rules = HashMap::new();
    rules.insert(
        "no-explicit-any".into(),
        serde_json::Value::String("warn".into()),
    );
    rules.insert(
        "no-unused-vars".into(),
        serde_json::Value::String("error".into()),
    );
    rules.insert(
        "no-console".into(),
        serde_json::Value::String("warn".into()),
    );
    rules.insert(
        "prefer-const".into(),
        serde_json::Value::String("warn".into()),
    );
    rules.insert(
        "no-empty-catch".into(),
        serde_json::Value::String("error".into()),
    );
    rules
}

pub fn load_config() -> Result<Config> {
    Ok(load_config_with_fingerprint()?.config)
}

pub fn load_config_with_fingerprint() -> Result<LoadedConfig> {
    let path = Path::new("flint.json");
    if !path.exists() {
        let config = Config::default();
        return Ok(LoadedConfig {
            fingerprint: hash_string("__default_detect_v1__"),
            config,
        });
    }

    let raw = std::fs::read_to_string(path).into_diagnostic()?;
    let file_config: FileConfig = serde_json::from_str(&raw)
        .map_err(|e| miette::miette!("Failed to parse flint.json: {}", e))?;
    let config = Config {
        detect: file_config.detect,
        rules: file_config.rules,
        files: file_config.files,
    };

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

    let project = ProjectInfo::detect(Path::new("."));
    let config = build_init_config(&project);

    let path = Path::new("flint.json");
    if path.exists() {
        eprintln!("{} flint.json already exists", "warning:".yellow().bold());
        return Ok(());
    }

    let raw = serde_json::to_string_pretty(&config).into_diagnostic()? + "\n";
    std::fs::write(path, raw).into_diagnostic()?;
    println!("{} Created flint.json", "✓".green().bold());
    let detected = detected_frameworks(&project);
    if detected.is_empty() {
        println!("  Detected frameworks: none");
    } else {
        println!("  Detected frameworks: {}", detected.join(", ").cyan());
    }
    println!(
        "  {}",
        "`detect: true` will auto-enable matching built-in rule categories.".dimmed()
    );
    println!(
        "  Edit the config and run {} to start linting",
        "flint check".cyan()
    );

    Ok(())
}

fn build_init_config(project: &ProjectInfo) -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://flickjs.dev/lint/schema.json",
        "detect": true,
        "rules": starter_rules_for_project(project),
        "files": {
            "exclude": default_excludes(),
        }
    })
}

fn starter_rules_for_project(project: &ProjectInfo) -> serde_json::Map<String, serde_json::Value> {
    let mut rules = serde_json::Map::new();

    for (rule, severity) in [
        ("no-explicit-any", "warn"),
        ("no-unused-vars", "error"),
        ("no-console", "warn"),
        ("prefer-const", "warn"),
        ("no-empty-catch", "error"),
        ("unreachable-code", "error"),
        ("no-missing-return", "error"),
        ("no-wrong-arg-count", "error"),
        ("no-unsafe-optional-access", "error"),
        ("no-eval", "error"),
        ("no-hardcoded-secrets", "warn"),
    ] {
        rules.insert(rule.into(), serde_json::json!(severity));
    }

    if project.has_react {
        for (rule, severity) in [
            ("react/no-fetch-in-effect", "warn"),
            ("react/functional-set-state", "warn"),
            ("react/no-array-index-key", "warn"),
            ("react/no-usememo-simple-expr", "warn"),
            ("react/no-hydration-flicker", "warn"),
        ] {
            rules.insert(rule.into(), serde_json::json!(severity));
        }
    }

    if project.has_next {
        for (rule, severity) in [
            ("nextjs/no-img-element", "warn"),
            ("nextjs/prefer-next-link", "warn"),
            ("nextjs/missing-metadata", "warn"),
            ("nextjs/no-async-client-component", "warn"),
            ("react/server-auth-actions", "warn"),
        ] {
            rules.insert(rule.into(), serde_json::json!(severity));
        }
    }

    if project.has_server_framework() {
        for (rule, severity) in [
            ("server/no-sql-injection", "error"),
            ("server/no-shell-injection", "error"),
            ("server/require-input-validation", "warn"),
            ("server/no-unhandled-async-route", "warn"),
            ("server/no-n-plus-one", "warn"),
        ] {
            rules.insert(rule.into(), serde_json::json!(severity));
        }
    }

    if project.has_react_native || project.has_expo {
        for (rule, severity) in [
            ("react-native/no-inline-styles", "warn"),
            ("react-native/no-anonymous-list-render", "warn"),
            ("react-native/require-key-extractor", "warn"),
        ] {
            rules.insert(rule.into(), serde_json::json!(severity));
        }
    }

    rules
}

fn detected_frameworks(project: &ProjectInfo) -> Vec<&'static str> {
    let mut frameworks = Vec::new();

    if project.has_next {
        frameworks.push("nextjs");
    } else if project.has_react {
        frameworks.push("react");
    }
    if project.has_server_framework() {
        frameworks.push("server");
    }
    if project.has_react_native {
        frameworks.push("react-native");
    }
    if project.has_expo {
        frameworks.push("expo");
    }

    frameworks
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
    use clap::CommandFactory;

    #[test]
    fn default_config_has_five_rules() {
        let config = Config::default();
        assert!(config.detect);
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

    #[test]
    fn init_template_includes_nextjs_rules() {
        let project = ProjectInfo {
            has_react: true,
            has_next: true,
            ..ProjectInfo::default()
        };
        let rules = starter_rules_for_project(&project);
        assert!(rules.contains_key("nextjs/no-img-element"));
        assert!(rules.contains_key("nextjs/missing-metadata"));
        assert!(rules.contains_key("react/server-auth-actions"));
    }

    #[test]
    fn init_template_includes_server_rules() {
        let project = ProjectInfo {
            has_express: true,
            ..ProjectInfo::default()
        };
        let rules = starter_rules_for_project(&project);
        assert!(rules.contains_key("server/no-sql-injection"));
        assert!(rules.contains_key("server/require-input-validation"));
    }

    #[test]
    fn detected_frameworks_prefers_nextjs_label_for_next_projects() {
        let project = ProjectInfo {
            has_react: true,
            has_next: true,
            has_express: true,
            ..ProjectInfo::default()
        };
        assert_eq!(detected_frameworks(&project), vec!["nextjs", "server"]);
    }

    #[test]
    fn init_template_sets_detect_true() {
        let config = build_init_config(&ProjectInfo::test_all());
        assert_eq!(config.get("detect"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn parses_rules_command_flags() {
        let cli = Cli::parse_from([
            "flint",
            "rules",
            "--group",
            "react-hooks",
            "--search",
            "fetch",
        ]);

        match cli.command {
            Command::Rules(args) => {
                assert_eq!(args.group.as_deref(), Some("react-hooks"));
                assert_eq!(args.search.as_deref(), Some("fetch"));
            }
            _ => panic!("expected rules command"),
        }
    }

    #[test]
    fn help_mentions_rules_command() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("rules"));
        assert!(help.contains("Browse Flint's built-in rules"));
    }
}
