use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "zarc",
    about = "⚡ Blazing-fast JavaScript and TypeScript linter",
    version,
    after_help = "Examples:\n  zarc check ./src\n  zarc check . --format json\n  zarc init"
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
    /// Initialize zarc config in current directory
    Init,
}

#[derive(clap::Args, Clone)]
pub struct CheckArgs {
    /// Path to lint (file or directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Path to cache file
    #[arg(long, default_value = ".zarc-cache.json")]
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub lint: LintConfig,
    #[serde(default)]
    pub files: FilesConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LintConfig {
    #[serde(default)]
    pub rules: HashMap<String, String>,
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
            lint: LintConfig::default(),
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

pub fn load_config() -> Result<Config> {
    Ok(load_config_with_fingerprint()?.config)
}

pub fn load_config_with_fingerprint() -> Result<LoadedConfig> {
    let path = Path::new("zarc.toml");
    if !path.exists() {
        return Ok(LoadedConfig {
            config: Config::default(),
            fingerprint: hash_string("__default__"),
        });
    }

    let raw = std::fs::read_to_string(path).into_diagnostic()?;
    let normalized = normalize_inline_tables(&raw);
    let config = toml::from_str(&normalized).into_diagnostic()?;

    Ok(LoadedConfig {
        config,
        fingerprint: hash_string(&normalized),
    })
}

fn normalize_inline_tables(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut brace_depth = 0usize;
    let mut in_string = false;
    let mut string_delim = '\0';
    let mut escape = false;

    for ch in raw.chars() {
        if in_string {
            normalized.push(ch);
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && string_delim == '"' {
                escape = true;
            } else if ch == string_delim {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                in_string = true;
                string_delim = ch;
                normalized.push(ch);
            }
            '{' => {
                brace_depth += 1;
                normalized.push(ch);
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                normalized.push(ch);
            }
            '\n' if brace_depth > 0 => {
                if !normalized.ends_with(' ') {
                    normalized.push(' ');
                }
            }
            _ => normalized.push(ch),
        }
    }

    normalized
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

/// Initialize a zarc.toml config file
pub fn init_config() -> Result<()> {
    use colored::*;

    let config = r#"# zarc.toml — Zarc linter configuration

[lint]
# "off" | "warn" | "error"
rules = { no-explicit-any = "warn", no-unused-vars = "error", no-console = "warn", prefer-const = "warn", no-empty-catch = "error" }

[files]
exclude = ["node_modules", "dist", "build", ".git"]
"#;

    let path = Path::new("zarc.toml");
    if path.exists() {
        eprintln!("{} zarc.toml already exists", "warning:".yellow().bold());
        return Ok(());
    }

    std::fs::write(path, config).into_diagnostic()?;
    println!("{} Created zarc.toml", "✓".green().bold());
    println!(
        "  Edit the config and run {} to start linting",
        "zarc check".cyan()
    );

    Ok(())
}
