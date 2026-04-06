pub mod cli;
pub mod project;
pub mod rule_catalog;
pub mod rules;
pub mod results_tui;
pub mod rules_tui;
pub mod tui_common;

use clap::Parser;
use colored::*;
use miette::Result;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const AUTO_SMALL_FILE_THRESHOLD: usize = 32;
const AUTO_SMALL_BYTE_THRESHOLD: u64 = 256 * 1024;
const AUTO_MARGIN_PERCENT: u64 = 10;

pub fn run() -> Result<i32> {
    let cli = cli::Cli::parse();
    let process_start = Instant::now();

    let (exit_code, metrics) = match &cli.command {
        cli::Command::Check(args) => {
            let execution = execute_check(args)?;
            let summary = print_results(
                &execution.results,
                &args.format,
                execution.metrics.total_runtime,
                &args.path,
            );
            (
                if summary.errors > 0 { 1 } else { 0 },
                Some(execution.metrics),
            )
        }
        cli::Command::Init => {
            cli::init_config()?;
            (0, None)
        }
        cli::Command::Rules(args) => {
            rules_tui::run(args)?;
            (0, None)
        }
    };

    if cli.timing {
        match metrics {
            Some(metrics) => print_timing(&metrics),
            None => eprintln!("\n⚡ Flint finished in {:.2?}", process_start.elapsed()),
        }
    }

    Ok(exit_code)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheMode {
    Auto,
    Off,
}

impl CacheMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheDecision {
    Off,
    Used,
    Bypassed,
}

impl CacheDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Used => "used",
            Self::Bypassed => "bypassed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheReason {
    Disabled,
    CacheMissing,
    CacheInvalidated,
    BootstrapSmall,
    PredictedWin,
    PredictedColdFaster,
    InsufficientSamples,
}

impl CacheReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::CacheMissing => "cache-missing",
            Self::CacheInvalidated => "cache-invalidated",
            Self::BootstrapSmall => "bootstrap-small",
            Self::PredictedWin => "predicted-win",
            Self::PredictedColdFaster => "predicted-cold-faster",
            Self::InsufficientSamples => "insufficient-samples",
        }
    }
}

#[derive(Debug, Clone)]
struct RunMetrics {
    total_runtime: Duration,
    cache_mode: CacheMode,
    cache_decision: CacheDecision,
    cache_reason: CacheReason,
    files_discovered: usize,
    total_bytes: u64,
    cache_hits: usize,
    hash_hits: usize,
    cache_misses: usize,
    changed_files: usize,
    load_time: Duration,
    stat_time: Duration,
    hash_time: Duration,
    lint_time: Duration,
    save_time: Duration,
    hash_bytes: u64,
    lint_bytes: u64,
    primed_cache: bool,
}

impl Default for RunMetrics {
    fn default() -> Self {
        Self {
            total_runtime: Duration::ZERO,
            cache_mode: CacheMode::Auto,
            cache_decision: CacheDecision::Bypassed,
            cache_reason: CacheReason::CacheMissing,
            files_discovered: 0,
            total_bytes: 0,
            cache_hits: 0,
            hash_hits: 0,
            cache_misses: 0,
            changed_files: 0,
            load_time: Duration::ZERO,
            stat_time: Duration::ZERO,
            hash_time: Duration::ZERO,
            lint_time: Duration::ZERO,
            save_time: Duration::ZERO,
            hash_bytes: 0,
            lint_bytes: 0,
            primed_cache: false,
        }
    }
}

struct CheckExecution {
    results: Vec<rules::LintResult>,
    metrics: RunMetrics,
}

#[derive(Clone)]
struct FileSnapshot {
    path: PathBuf,
    fingerprint: rules::FileFingerprint,
}

struct CacheUpdate {
    path: PathBuf,
    fingerprint: rules::FileFingerprint,
    hash: String,
    result: rules::LintResult,
}

enum FileStatus {
    MetadataHit,
    HashHit,
    Miss,
}

struct FileExecution {
    result: rules::LintResult,
    update: Option<CacheUpdate>,
    status: FileStatus,
    hash_time: Duration,
    hash_bytes: u64,
    lint_time: Duration,
    lint_bytes: u64,
}

enum ExecutionStrategy {
    Bypass {
        reason: CacheReason,
        prime_cache: bool,
    },
    UseCache {
        reason: CacheReason,
    },
}

fn execute_check(args: &cli::CheckArgs) -> Result<CheckExecution> {
    let total_start = Instant::now();
    let loaded_config = cli::load_config_with_fingerprint()?;
    let rule_config = &loaded_config.config.rules;
    let project = project::ProjectInfo::detect(&args.path);
    let config_fingerprint = rules::hash_bytes(
        format!("{}:{}", loaded_config.fingerprint, project.fingerprint()).as_bytes(),
    );
    let files = cli::discover_files(
        &args.path,
        &loaded_config.config.files.exclude,
        &args.ignore,
    )?;

    let mut metrics = RunMetrics {
        cache_mode: if args.no_cache {
            CacheMode::Off
        } else {
            CacheMode::Auto
        },
        cache_decision: if args.no_cache {
            CacheDecision::Off
        } else {
            CacheDecision::Bypassed
        },
        cache_reason: if args.no_cache {
            CacheReason::Disabled
        } else {
            CacheReason::CacheMissing
        },
        files_discovered: files.len(),
        ..RunMetrics::default()
    };

    if args.no_cache {
        let snapshots = collect_file_snapshots(&files, &mut metrics);
        let (mut results, _) = execute_without_cache(
            &snapshots,
            rule_config,
            loaded_config.config.detect,
            &project,
            false,
            &mut metrics,
        );
        run_cross_file_analysis(&mut results, &files, rule_config, &args.path);
        metrics.total_runtime = total_start.elapsed();
        return Ok(CheckExecution { results, metrics });
    }

    let load_start = Instant::now();
    let (mut cache, load_status) = rules::Cache::load(&args.cache_path, &config_fingerprint)?;
    metrics.load_time = load_start.elapsed();

    let snapshots = collect_file_snapshots(&files, &mut metrics);
    let strategy = select_strategy(&cache, load_status, &snapshots, &metrics);

    let (mut results, updates, dirty_entries) = match strategy {
        ExecutionStrategy::Bypass {
            reason,
            prime_cache,
        } => {
            metrics.cache_decision = CacheDecision::Bypassed;
            metrics.cache_reason = reason;
            metrics.primed_cache = prime_cache;

            let (results, updates) = execute_without_cache(
                &snapshots,
                rule_config,
                loaded_config.config.detect,
                &project,
                prime_cache,
                &mut metrics,
            );
            (results, updates, prime_cache)
        }
        ExecutionStrategy::UseCache { reason } => {
            metrics.cache_decision = CacheDecision::Used;
            metrics.cache_reason = reason;
            execute_with_cache(
                &snapshots,
                &cache,
                rule_config,
                loaded_config.config.detect,
                &project,
                &mut metrics,
            )
        }
    };

    let live_paths: HashSet<PathBuf> = snapshots
        .iter()
        .map(|snapshot| snapshot.path.clone())
        .collect();
    let has_deleted_entries = cache.entries.keys().any(|path| !live_paths.contains(path));

    if dirty_entries || has_deleted_entries {
        cache.update_fingerprint(&config_fingerprint);
        let mut should_persist = cache.prune_to(&live_paths);

        for update in updates {
            should_persist |=
                cache.upsert(update.path, update.fingerprint, update.hash, update.result);
        }

        cache
            .header
            .timings
            .record_load(duration_ns(metrics.load_time));
        cache
            .header
            .timings
            .record_stat(duration_ns(metrics.stat_time), snapshots.len());
        cache
            .header
            .timings
            .record_hash(duration_ns(metrics.hash_time), metrics.hash_bytes);
        cache
            .header
            .timings
            .record_lint(duration_ns(metrics.lint_time), metrics.lint_bytes);

        if should_persist {
            let save_start = Instant::now();
            cache.persist(&args.cache_path)?;
            metrics.save_time = save_start.elapsed();
        }
    }

    run_cross_file_analysis(&mut results, &files, rule_config, &args.path);
    metrics.total_runtime = total_start.elapsed();
    Ok(CheckExecution { results, metrics })
}

/// Run cross-file dead code analysis (unused exports, unused files, unused dependencies).
/// Appends diagnostics to the existing per-file results.
fn run_cross_file_analysis(
    results: &mut Vec<rules::LintResult>,
    files: &[PathBuf],
    rule_config: &HashMap<String, serde_json::Value>,
    project_root: &Path,
) {
    let wants_unused_exports = rule_config
        .get("unused-exports")
        .and_then(|v| cli::parse_rule_severity(v))
        .is_some();
    let wants_unused_files = rule_config
        .get("unused-files")
        .and_then(|v| cli::parse_rule_severity(v))
        .is_some();
    let wants_unused_deps = rule_config
        .get("unused-dependencies")
        .and_then(|v| cli::parse_rule_severity(v))
        .is_some();

    if !wants_unused_exports && !wants_unused_files && !wants_unused_deps {
        return;
    }

    // Read all file sources for import graph analysis
    let file_sources: Vec<(PathBuf, String)> = files
        .par_iter()
        .filter_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .map(|source| (path.clone(), source))
        })
        .collect();

    let resolver_config = project::load_module_resolution_config(project_root);
    let graph = rules::dead_code::build_import_graph(&file_sources, files, resolver_config);

    // Apply severity overrides
    let export_severity = rule_config
        .get("unused-exports")
        .and_then(|v| cli::parse_rule_severity(v));
    let file_severity = rule_config
        .get("unused-files")
        .and_then(|v| cli::parse_rule_severity(v));
    let dep_severity = rule_config
        .get("unused-dependencies")
        .and_then(|v| cli::parse_rule_severity(v));

    if wants_unused_exports {
        let mut diagnostics = rules::dead_code::find_unused_exports(&graph);
        if let Some(ref severity) = export_severity {
            for (_, d) in &mut diagnostics {
                d.severity = severity.clone();
            }
        }
        append_paired_diagnostics(results, diagnostics);
    }

    if wants_unused_files {
        let mut diagnostics = rules::dead_code::find_unused_files(&graph);
        if let Some(ref severity) = file_severity {
            for (_, d) in &mut diagnostics {
                d.severity = severity.clone();
            }
        }
        append_paired_diagnostics(results, diagnostics);
    }

    if wants_unused_deps {
        let package_json = project_root.join("package.json");
        if package_json.exists() {
            let mut diagnostics = rules::dead_code::find_unused_dependencies(&graph, &package_json);
            if let Some(ref severity) = dep_severity {
                for d in &mut diagnostics {
                    d.severity = severity.clone();
                }
            }
            if !diagnostics.is_empty() {
                results.push(rules::LintResult {
                    file: package_json,
                    diagnostics,
                });
            }
        }
    }
}

/// Append (file_path, diagnostic) pairs to matching file results, or create new entries.
fn append_paired_diagnostics(
    results: &mut Vec<rules::LintResult>,
    diagnostics: Vec<(PathBuf, rules::LintDiagnostic)>,
) {
    for (file_path, diagnostic) in diagnostics {
        if let Some(existing) = results.iter_mut().find(|r| r.file == file_path) {
            existing.diagnostics.push(diagnostic);
        } else {
            results.push(rules::LintResult {
                file: file_path,
                diagnostics: vec![diagnostic],
            });
        }
    }
}

fn collect_file_snapshots(files: &[PathBuf], metrics: &mut RunMetrics) -> Vec<FileSnapshot> {
    let stat_start = Instant::now();
    let snapshots: Vec<FileSnapshot> = files
        .par_iter()
        .filter_map(|file| match rules::FileFingerprint::from_path(file) {
            Ok(Some(fingerprint)) => Some(FileSnapshot {
                path: file.clone(),
                fingerprint,
            }),
            Ok(None) => None,
            Err(error) => {
                eprintln!("Error reading metadata for {}: {}", file.display(), error);
                None
            }
        })
        .collect();
    metrics.stat_time = stat_start.elapsed();
    metrics.files_discovered = snapshots.len();
    metrics.total_bytes = snapshots
        .iter()
        .map(|snapshot| snapshot.fingerprint.size)
        .sum();
    snapshots
}

fn select_strategy(
    cache: &rules::Cache,
    load_status: rules::CacheLoadStatus,
    snapshots: &[FileSnapshot],
    metrics: &RunMetrics,
) -> ExecutionStrategy {
    let is_small = snapshots.len() <= AUTO_SMALL_FILE_THRESHOLD
        || metrics.total_bytes <= AUTO_SMALL_BYTE_THRESHOLD;

    match load_status {
        rules::CacheLoadStatus::Missing => {
            if is_small {
                ExecutionStrategy::Bypass {
                    reason: CacheReason::BootstrapSmall,
                    prime_cache: false,
                }
            } else {
                ExecutionStrategy::Bypass {
                    reason: CacheReason::CacheMissing,
                    prime_cache: true,
                }
            }
        }
        rules::CacheLoadStatus::Invalidated => {
            if is_small {
                ExecutionStrategy::Bypass {
                    reason: CacheReason::BootstrapSmall,
                    prime_cache: false,
                }
            } else {
                ExecutionStrategy::Bypass {
                    reason: CacheReason::CacheInvalidated,
                    prime_cache: true,
                }
            }
        }
        rules::CacheLoadStatus::Loaded => {
            let timings = &cache.header.timings;
            if !timings.ready_for_prediction() && is_small {
                return ExecutionStrategy::Bypass {
                    reason: CacheReason::BootstrapSmall,
                    prime_cache: false,
                };
            }

            if !timings.ready_for_prediction() {
                return ExecutionStrategy::UseCache {
                    reason: CacheReason::InsufficientSamples,
                };
            }

            if should_use_cache(cache, snapshots, metrics.total_bytes) {
                ExecutionStrategy::UseCache {
                    reason: CacheReason::PredictedWin,
                }
            } else {
                ExecutionStrategy::Bypass {
                    reason: CacheReason::PredictedColdFaster,
                    prime_cache: false,
                }
            }
        }
    }
}

fn should_use_cache(cache: &rules::Cache, snapshots: &[FileSnapshot], total_bytes: u64) -> bool {
    let inspect_bytes = snapshots
        .iter()
        .filter(|snapshot| {
            cache
                .get(&snapshot.path)
                .map(|entry| !entry.fingerprint.matches(&snapshot.fingerprint))
                .unwrap_or(true)
        })
        .map(|snapshot| snapshot.fingerprint.size)
        .sum::<u64>();

    let timings = &cache.header.timings;
    let predicted_cold = timings.predict_lint(total_bytes).unwrap_or(u64::MAX);
    let predicted_warm = timings.predict_load().unwrap_or(0)
        + timings.predict_stat(snapshots.len()).unwrap_or(0)
        + if inspect_bytes > 0 {
            timings
                .predict_hash(inspect_bytes)
                .or_else(|| timings.predict_lint(inspect_bytes))
                .unwrap_or(0)
                + timings.predict_lint(inspect_bytes).unwrap_or(0)
                + timings
                    .predict_save()
                    .or_else(|| timings.predict_load())
                    .unwrap_or(0)
        } else {
            0
        };

    predicted_warm.saturating_mul(100) <= predicted_cold.saturating_mul(100 - AUTO_MARGIN_PERCENT)
}

fn execute_without_cache(
    snapshots: &[FileSnapshot],
    rule_config: &HashMap<String, serde_json::Value>,
    detect: bool,
    project: &project::ProjectInfo,
    prepare_cache_entries: bool,
    metrics: &mut RunMetrics,
) -> (Vec<rules::LintResult>, Vec<CacheUpdate>) {
    let executions = snapshots
        .par_iter()
        .filter_map(|snapshot| {
            if prepare_cache_entries {
                let lint_start = Instant::now();
                match rules::load_source_with_hash(&snapshot.path) {
                    Ok(loaded) => {
                        let result = rules::lint_source_with_config(
                            &snapshot.path,
                            &loaded.source,
                            rule_config,
                            detect,
                            project,
                        );
                        let lint_time = lint_start.elapsed();
                        Some(FileExecution {
                            update: Some(CacheUpdate {
                                path: snapshot.path.clone(),
                                fingerprint: snapshot.fingerprint.clone(),
                                hash: loaded.hash,
                                result: result.clone(),
                            }),
                            result,
                            status: FileStatus::Miss,
                            hash_time: Duration::ZERO,
                            hash_bytes: 0,
                            lint_time,
                            lint_bytes: loaded.size,
                        })
                    }
                    Err(error) => {
                        eprintln!("Error linting {}: {}", snapshot.path.display(), error);
                        None
                    }
                }
            } else {
                let lint_start = Instant::now();
                match rules::lint_file_with_config(&snapshot.path, rule_config, detect, project) {
                    Ok(result) => Some(FileExecution {
                        result,
                        update: None,
                        status: FileStatus::Miss,
                        hash_time: Duration::ZERO,
                        hash_bytes: 0,
                        lint_time: lint_start.elapsed(),
                        lint_bytes: snapshot.fingerprint.size,
                    }),
                    Err(error) => {
                        eprintln!("Error linting {}: {}", snapshot.path.display(), error);
                        None
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let mut results = Vec::with_capacity(executions.len());
    let mut updates = Vec::new();

    for execution in executions {
        metrics.cache_misses += 1;
        metrics.changed_files += 1;
        metrics.hash_time += execution.hash_time;
        metrics.hash_bytes += execution.hash_bytes;
        metrics.lint_time += execution.lint_time;
        metrics.lint_bytes += execution.lint_bytes;
        if let Some(update) = execution.update {
            updates.push(update);
        }
        results.push(execution.result);
    }

    (results, updates)
}

fn execute_with_cache(
    snapshots: &[FileSnapshot],
    cache: &rules::Cache,
    rule_config: &HashMap<String, serde_json::Value>,
    detect: bool,
    project: &project::ProjectInfo,
    metrics: &mut RunMetrics,
) -> (Vec<rules::LintResult>, Vec<CacheUpdate>, bool) {
    let executions: Vec<FileExecution> = snapshots
        .par_iter()
        .filter_map(|snapshot| {
            if let Some(cached) = cache.get(&snapshot.path) {
                if cached.fingerprint.matches(&snapshot.fingerprint) {
                    return Some(FileExecution {
                        result: cached.result.clone(),
                        update: None,
                        status: FileStatus::MetadataHit,
                        hash_time: Duration::ZERO,
                        hash_bytes: 0,
                        lint_time: Duration::ZERO,
                        lint_bytes: 0,
                    });
                }

                let hash_start = Instant::now();
                let loaded = match rules::load_source_with_hash(&snapshot.path) {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        eprintln!("Error linting {}: {}", snapshot.path.display(), error);
                        return None;
                    }
                };
                let hash_time = hash_start.elapsed();

                if loaded.hash == cached.hash {
                    return Some(FileExecution {
                        result: cached.result.clone(),
                        update: Some(CacheUpdate {
                            path: snapshot.path.clone(),
                            fingerprint: snapshot.fingerprint.clone(),
                            hash: loaded.hash,
                            result: cached.result.clone(),
                        }),
                        status: FileStatus::HashHit,
                        hash_time,
                        hash_bytes: loaded.size,
                        lint_time: Duration::ZERO,
                        lint_bytes: 0,
                    });
                }

                let lint_start = Instant::now();
                let result = rules::lint_source_with_config(
                    &snapshot.path,
                    &loaded.source,
                    rule_config,
                    detect,
                    project,
                );
                let lint_time = lint_start.elapsed();

                return Some(FileExecution {
                    update: Some(CacheUpdate {
                        path: snapshot.path.clone(),
                        fingerprint: snapshot.fingerprint.clone(),
                        hash: loaded.hash,
                        result: result.clone(),
                    }),
                    result,
                    status: FileStatus::Miss,
                    hash_time,
                    hash_bytes: loaded.size,
                    lint_time,
                    lint_bytes: loaded.size,
                });
            }

            let hash_start = Instant::now();
            let loaded = match rules::load_source_with_hash(&snapshot.path) {
                Ok(loaded) => loaded,
                Err(error) => {
                    eprintln!("Error linting {}: {}", snapshot.path.display(), error);
                    return None;
                }
            };
            let hash_time = hash_start.elapsed();

            let lint_start = Instant::now();
            let result = rules::lint_source_with_config(
                &snapshot.path,
                &loaded.source,
                rule_config,
                detect,
                project,
            );
            let lint_time = lint_start.elapsed();

            Some(FileExecution {
                update: Some(CacheUpdate {
                    path: snapshot.path.clone(),
                    fingerprint: snapshot.fingerprint.clone(),
                    hash: loaded.hash,
                    result: result.clone(),
                }),
                result,
                status: FileStatus::Miss,
                hash_time,
                hash_bytes: loaded.size,
                lint_time,
                lint_bytes: loaded.size,
            })
        })
        .collect();

    let mut results = Vec::with_capacity(executions.len());
    let mut updates = Vec::new();
    let mut dirty_entries = false;

    for execution in executions {
        match execution.status {
            FileStatus::MetadataHit => {
                metrics.cache_hits += 1;
            }
            FileStatus::HashHit => {
                metrics.cache_hits += 1;
                metrics.hash_hits += 1;
                dirty_entries = true;
            }
            FileStatus::Miss => {
                metrics.cache_misses += 1;
                metrics.changed_files += 1;
                dirty_entries = true;
            }
        }

        metrics.hash_time += execution.hash_time;
        metrics.hash_bytes += execution.hash_bytes;
        metrics.lint_time += execution.lint_time;
        metrics.lint_bytes += execution.lint_bytes;

        if let Some(update) = execution.update {
            updates.push(update);
        }
        results.push(execution.result);
    }

    (results, updates, dirty_entries)
}

fn duration_ns(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

fn print_timing(metrics: &RunMetrics) {
    eprintln!("\n⚡ Flint finished in {:.2?}", metrics.total_runtime);
    eprintln!(
        "  cache mode={} decision={} reason={} hits={} hash_hits={} misses={} changed={} primed={}",
        metrics.cache_mode.as_str(),
        metrics.cache_decision.as_str(),
        metrics.cache_reason.as_str(),
        metrics.cache_hits,
        metrics.hash_hits,
        metrics.cache_misses,
        metrics.changed_files,
        if metrics.primed_cache { "yes" } else { "no" },
    );
    eprintln!(
        "  timing load={:.2?} stat={:.2?} hash={:.2?} lint={:.2?} save={:.2?}",
        metrics.load_time,
        metrics.stat_time,
        metrics.hash_time,
        metrics.lint_time,
        metrics.save_time,
    );
}

pub(crate) struct Summary {
    pub(crate) errors: usize,
}

pub(crate) struct PrettyEntry<'a> {
    pub(crate) file: &'a Path,
    pub(crate) diagnostic: &'a rules::LintDiagnostic,
}

pub(crate) fn diagnostic_counts(results: &[rules::LintResult]) -> (usize, usize) {
    let mut errors = 0;
    let mut warnings = 0;
    for result in results {
        for diagnostic in &result.diagnostics {
            match diagnostic.severity {
                rules::Severity::Error => errors += 1,
                rules::Severity::Warning => warnings += 1,
            }
        }
    }
    (errors, warnings)
}

/// Categories sorted by `category_order`, entries sorted like pretty output.
pub(crate) fn group_results_for_display<'a>(
    results: &'a [rules::LintResult],
) -> Vec<(String, Vec<PrettyEntry<'a>>)> {
    let mut grouped: HashMap<String, Vec<PrettyEntry<'a>>> = HashMap::new();

    for result in results {
        for diagnostic in &result.diagnostics {
            grouped
                .entry(diagnostic_category_key(diagnostic))
                .or_default()
                .push(PrettyEntry {
                    file: &result.file,
                    diagnostic,
                });
        }
    }

    let mut categories: Vec<(String, Vec<PrettyEntry<'a>>)> = grouped.into_iter().collect();
    categories.sort_by(|(left, _), (right, _)| {
        category_order(left)
            .cmp(&category_order(right))
            .then_with(|| left.cmp(right))
    });

    for (_, entries) in &mut categories {
        entries.sort_by(|left, right| {
            crate::tui_common::severity_rank(&left.diagnostic.severity)
                .cmp(&crate::tui_common::severity_rank(&right.diagnostic.severity))
                .then_with(|| left.file.cmp(right.file))
                .then_with(|| left.diagnostic.byte_start.cmp(&right.diagnostic.byte_start))
                .then_with(|| left.diagnostic.rule_name.cmp(&right.diagnostic.rule_name))
        });
    }

    categories
}

fn print_results(
    results: &[rules::LintResult],
    format: &cli::OutputFormat,
    elapsed: std::time::Duration,
    scan_root: &Path,
) -> Summary {
    match format {
        cli::OutputFormat::Json => print_json(results),
        cli::OutputFormat::Compact => print_compact(results),
        cli::OutputFormat::Pretty => print_pretty(results, elapsed),
        cli::OutputFormat::Tui => results_tui::print_or_fallback(results, elapsed, scan_root),
        cli::OutputFormat::AgentJson => print_agent_json(results),
    }
}

pub(crate) fn print_pretty(results: &[rules::LintResult], elapsed: std::time::Duration) -> Summary {
    let (total_errors, total_warnings) = diagnostic_counts(results);
    let categories = group_results_for_display(results);

    for (index, (category, entries)) in categories.iter().enumerate() {
        if index > 0 {
            println!();
        }

        println!(
            "  {} {}",
            category_display_name(&category).bold(),
            format!("({})", entries.len()).dimmed()
        );

        for entry in entries {
            let diagnostic = entry.diagnostic;
            match diagnostic.severity {
                rules::Severity::Error => println!(
                    "    {} {}:{} {}",
                    "error".red().bold(),
                    entry.file.display(),
                    diagnostic.span,
                    diagnostic.message
                ),
                rules::Severity::Warning => println!(
                    "    {} {}:{} {}",
                    "warn".yellow().bold(),
                    entry.file.display(),
                    diagnostic.span,
                    diagnostic.message
                ),
            }
            println!("      {} {}", "rule".dimmed(), diagnostic.rule_name);
            if let Some(help) = diagnostic_help_text(diagnostic) {
                println!("      {} {}", "help".dimmed(), help);
            }
        }
    }

    println!();
    if total_errors == 0 && total_warnings == 0 {
        println!(
            "  {} No issues found in {:.2?}",
            "✓".green().bold(),
            elapsed
        );
    } else {
        println!(
            "  {} {} errors, {} warnings in {:.2?}",
            "✗".red().bold(),
            total_errors,
            total_warnings,
            elapsed
        );
    }

    Summary {
        errors: total_errors,
    }
}

pub(crate) fn diagnostic_category_key(diagnostic: &rules::LintDiagnostic) -> String {
    match diagnostic.rule_name.as_str() {
        "parse-error" => "parse".to_string(),
        "semantic-error" => "semantic".to_string(),
        _ if diagnostic.origin == rules::RuleOrigin::Config
            && !diagnostic.rule_name.contains('/') =>
        {
            "policy".to_string()
        }
        _ => diagnostic
            .rule_name
            .split_once('/')
            .map(|(prefix, _)| prefix.to_string())
            .unwrap_or_else(|| "core".to_string()),
    }
}

pub(crate) fn category_order(category: &str) -> usize {
    match category {
        "parse" => 0,
        "semantic" => 1,
        "core" => 2,
        "react" => 3,
        "nextjs" => 4,
        "react-native" => 5,
        "server" => 6,
        "policy" => 7,
        _ => 8,
    }
}

pub(crate) fn category_display_name(category: &str) -> &'static str {
    match category {
        "parse" => "Parse",
        "semantic" => "Semantic",
        "core" => "Core",
        "react" => "React",
        "nextjs" => "Next.js",
        "react-native" => "React Native",
        "server" => "Server",
        "policy" => "Policy",
        _ => "Other",
    }
}

pub(crate) fn diagnostic_help_text(diagnostic: &rules::LintDiagnostic) -> Option<String> {
    match diagnostic.rule_name.as_str() {
        "parse-error" => Some("Fix syntax issues before Flint can run rule checks.".to_string()),
        "semantic-error" => {
            Some("Fix semantic issues before Flint can analyze this file fully.".to_string())
        }
        _ if matches!(
            diagnostic.origin,
            rules::RuleOrigin::BuiltIn | rules::RuleOrigin::Config
        ) =>
        {
            Some(format!(
                "configure in flint.json -> rules.\"{}\": \"off\"",
                diagnostic.rule_name
            ))
        }
        _ => None,
    }
}

fn print_compact(results: &[rules::LintResult]) -> Summary {
    let mut total_errors = 0;

    for result in results {
        for diagnostic in &result.diagnostics {
            if diagnostic.severity == rules::Severity::Error {
                total_errors += 1;
            }
            println!(
                "{}:{}: {} [{}]",
                result.file.display(),
                diagnostic.span,
                diagnostic.message,
                diagnostic.rule_name
            );
        }
    }

    Summary {
        errors: total_errors,
    }
}

fn print_json(results: &[rules::LintResult]) -> Summary {
    let total_errors = results
        .iter()
        .flat_map(|result| &result.diagnostics)
        .filter(|diagnostic| diagnostic.severity == rules::Severity::Error)
        .count();

    println!(
        "{}",
        serde_json::to_string_pretty(results).unwrap_or_else(|_| "[]".to_string())
    );

    Summary {
        errors: total_errors,
    }
}

// ── Agent JSON output ──────────────────────────────────────

#[derive(Serialize)]
struct AgentDiagnostic {
    rule: String,
    severity: String,
    message: String,
    file: String,
    span: AgentSpan,
    context: AgentContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    fix: Option<AgentFix>,
}

#[derive(Serialize)]
struct AgentSpan {
    start: AgentPosition,
    end: AgentPosition,
    byte_start: u32,
    byte_end: u32,
}

#[derive(Serialize)]
struct AgentPosition {
    line: usize,
    col: usize,
}

#[derive(Serialize)]
struct AgentContext {
    source_line: String,
    surrounding_lines: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
}

#[derive(Serialize)]
struct AgentFix {
    description: String,
    edits: Vec<AgentEdit>,
    safety: String,
}

#[derive(Serialize)]
struct AgentEdit {
    start_byte: usize,
    end_byte: usize,
    replacement: String,
}

#[derive(Serialize)]
struct AgentFileResult {
    file: String,
    diagnostics: Vec<AgentDiagnostic>,
}

fn print_agent_json(results: &[rules::LintResult]) -> Summary {
    let mut total_errors = 0;
    let mut agent_results = Vec::new();

    for result in results {
        let source = std::fs::read_to_string(&result.file).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        let mut agent_diagnostics = Vec::new();

        for diagnostic in &result.diagnostics {
            if diagnostic.severity == rules::Severity::Error {
                total_errors += 1;
            }

            // Parse "line:col" span
            let (start_line, start_col) = parse_span(&diagnostic.span);
            let (end_line, end_col) = if diagnostic.byte_end > diagnostic.byte_start {
                offset_to_line_col_from_source(&source, diagnostic.byte_end as usize)
            } else {
                (start_line, start_col)
            };

            // Get source context
            let source_line = lines
                .get(start_line.saturating_sub(1))
                .unwrap_or(&"")
                .to_string();

            let surrounding_start = start_line.saturating_sub(2); // 1 line before
            let surrounding_end = (start_line + 1).min(lines.len()); // 1 line after
            let surrounding_lines: Vec<String> = lines
                .get(surrounding_start..surrounding_end)
                .unwrap_or(&[])
                .iter()
                .map(|l| l.to_string())
                .collect();

            let agent_fix = diagnostic.fix.as_ref().map(|fix| AgentFix {
                description: fix
                    .description
                    .clone()
                    .unwrap_or_else(|| "Apply fix".to_string()),
                edits: vec![AgentEdit {
                    start_byte: fix.range.0,
                    end_byte: fix.range.1,
                    replacement: fix.replacement.clone(),
                }],
                safety: format!("{:?}", fix.safety).to_lowercase(),
            });

            agent_diagnostics.push(AgentDiagnostic {
                rule: diagnostic.rule_name.clone(),
                severity: match diagnostic.severity {
                    rules::Severity::Error => "error".to_string(),
                    rules::Severity::Warning => "warning".to_string(),
                },
                message: diagnostic.message.clone(),
                file: result.file.display().to_string(),
                span: AgentSpan {
                    start: AgentPosition {
                        line: start_line,
                        col: start_col,
                    },
                    end: AgentPosition {
                        line: end_line,
                        col: end_col,
                    },
                    byte_start: diagnostic.byte_start,
                    byte_end: diagnostic.byte_end,
                },
                context: AgentContext {
                    source_line,
                    surrounding_lines,
                    node_kind: diagnostic.node_kind.clone(),
                    symbol: diagnostic.symbol.clone(),
                },
                fix: agent_fix,
            });
        }

        if !agent_diagnostics.is_empty() {
            agent_results.push(AgentFileResult {
                file: result.file.display().to_string(),
                diagnostics: agent_diagnostics,
            });
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&agent_results).unwrap_or_else(|_| "[]".to_string())
    );

    Summary {
        errors: total_errors,
    }
}

fn parse_span(span: &str) -> (usize, usize) {
    let parts: Vec<&str> = span.split(':').collect();
    let line = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let col = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    (line, col)
}

fn offset_to_line_col_from_source(source: &str, offset: usize) -> (usize, usize) {
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
mod tests {
    use super::*;
    use std::fs;
    use std::thread::sleep;
    use tempfile::tempdir;

    const PERF_WARMUPS: usize = 2;
    const PERF_SAMPLES: usize = 7;

    #[derive(Clone, Copy)]
    struct FixtureSpec {
        files: usize,
        repeats: usize,
    }

    #[test]
    fn builtin_rules_default_to_core_category() {
        let diagnostic = rules::LintDiagnostic {
            rule_name: "no-console".to_string(),
            message: "Unexpected console statement".to_string(),
            span: "1:1".to_string(),
            severity: rules::Severity::Warning,
            origin: rules::RuleOrigin::BuiltIn,
            fix: None,
            byte_start: 0,
            byte_end: 0,
            node_kind: None,
            symbol: None,
        };

        assert_eq!(diagnostic_category_key(&diagnostic), "core");
        assert_eq!(category_display_name("core"), "Core");
    }

    #[test]
    fn config_rules_without_namespace_group_as_policy() {
        let diagnostic = rules::LintDiagnostic {
            rule_name: "custom-policy".to_string(),
            message: "Nope".to_string(),
            span: "1:1".to_string(),
            severity: rules::Severity::Warning,
            origin: rules::RuleOrigin::Config,
            fix: None,
            byte_start: 0,
            byte_end: 0,
            node_kind: None,
            symbol: None,
        };

        assert_eq!(diagnostic_category_key(&diagnostic), "policy");
    }

    #[test]
    fn builtin_diagnostics_include_disable_hint() {
        let diagnostic = rules::LintDiagnostic {
            rule_name: "react/no-fetch-in-effect".to_string(),
            message: "Move data fetching out of `useEffect` when possible".to_string(),
            span: "1:1".to_string(),
            severity: rules::Severity::Warning,
            origin: rules::RuleOrigin::BuiltIn,
            fix: None,
            byte_start: 0,
            byte_end: 0,
            node_kind: None,
            symbol: None,
        };

        assert_eq!(
            diagnostic_help_text(&diagnostic),
            Some(
                "configure in flint.json -> rules.\"react/no-fetch-in-effect\": \"off\""
                    .to_string()
            )
        );
    }

    #[test]
    fn no_cache_ignores_broken_cache_files() {
        let dir = tempdir().unwrap();
        write_fixture(
            dir.path(),
            FixtureSpec {
                files: 1,
                repeats: 4,
            },
        );
        let cache_path = dir.path().join("cache.json");
        fs::write(&cache_path, "{not-json").unwrap();

        let execution = execute_check(&check_args(dir.path(), &cache_path, true)).unwrap();

        assert_eq!(execution.metrics.cache_mode, CacheMode::Off);
        assert_eq!(execution.metrics.cache_decision, CacheDecision::Off);
        assert_eq!(fs::read_to_string(&cache_path).unwrap(), "{not-json");
    }

    #[test]
    fn auto_bypasses_small_unseeded_workloads() {
        let dir = tempdir().unwrap();
        write_fixture(
            dir.path(),
            FixtureSpec {
                files: 2,
                repeats: 4,
            },
        );
        let cache_path = dir.path().join("cache.json");

        let execution = execute_check(&check_args(dir.path(), &cache_path, false)).unwrap();

        assert_eq!(execution.metrics.cache_decision, CacheDecision::Bypassed);
        assert_eq!(execution.metrics.cache_reason, CacheReason::BootstrapSmall);
        assert!(!cache_path.exists());
    }

    #[test]
    fn auto_primes_and_then_uses_cache_on_large_workloads() {
        let dir = tempdir().unwrap();
        write_fixture(
            dir.path(),
            FixtureSpec {
                files: 48,
                repeats: 300,
            },
        );
        let cache_path = dir.path().join("cache.json");
        let args = check_args(dir.path(), &cache_path, false);

        let first = execute_check(&args).unwrap();
        assert_eq!(first.metrics.cache_decision, CacheDecision::Bypassed);
        assert_eq!(first.metrics.cache_reason, CacheReason::CacheMissing);
        assert!(first.metrics.primed_cache);
        assert!(cache_path.exists());

        let second = execute_check(&args).unwrap();
        assert_eq!(second.metrics.cache_decision, CacheDecision::Used);
        assert_eq!(second.metrics.cache_hits, second.metrics.files_discovered);
        assert_eq!(second.metrics.cache_misses, 0);
    }

    #[test]
    fn cache_prunes_deleted_files() {
        let dir = tempdir().unwrap();
        write_fixture(
            dir.path(),
            FixtureSpec {
                files: 40,
                repeats: 300,
            },
        );
        let cache_path = dir.path().join("cache.json");
        let deleted = dir.path().join("file_000.ts");
        let args = check_args(dir.path(), &cache_path, false);

        execute_check(&args).unwrap();
        fs::remove_file(&deleted).unwrap();
        let execution = execute_check(&args).unwrap();

        assert_eq!(execution.metrics.cache_decision, CacheDecision::Used);
        let (cache, status) = rules::Cache::load(&cache_path, &default_fingerprint()).unwrap();
        assert_eq!(status, rules::CacheLoadStatus::Loaded);
        assert!(!cache.entries.contains_key(&deleted));
        assert_eq!(cache.entries.len(), 39);
    }

    #[test]
    fn metadata_only_changes_reuse_cached_results_via_hash_fallback() {
        let dir = tempdir().unwrap();
        write_fixture(
            dir.path(),
            FixtureSpec {
                files: 40,
                repeats: 300,
            },
        );
        let cache_path = dir.path().join("cache.json");
        let args = check_args(dir.path(), &cache_path, false);
        let target = dir.path().join("file_005.ts");
        let original = fs::read_to_string(&target).unwrap();

        execute_check(&args).unwrap();
        sleep(Duration::from_millis(1200));
        fs::write(&target, &original).unwrap();

        let execution = execute_check(&args).unwrap();

        assert_eq!(execution.metrics.cache_decision, CacheDecision::Used);
        assert_eq!(execution.metrics.hash_hits, 1);
        assert_eq!(execution.metrics.cache_misses, 0);
    }

    #[test]
    #[ignore]
    fn cache_perf_gate() {
        assert_perf_ratio(
            "tiny unchanged",
            FixtureSpec {
                files: 16,
                repeats: 128,
            },
            Scenario::Unchanged,
            1.05,
        );
        assert_perf_ratio(
            "medium unchanged",
            FixtureSpec {
                files: 192,
                repeats: 500,
            },
            Scenario::Unchanged,
            0.80,
        );
        assert_perf_ratio(
            "large unchanged",
            FixtureSpec {
                files: 320,
                repeats: 600,
            },
            Scenario::Unchanged,
            0.70,
        );
        assert_perf_ratio(
            "medium one-file-changed",
            FixtureSpec {
                files: 192,
                repeats: 500,
            },
            Scenario::OneFileChanged,
            0.90,
        );
    }

    #[derive(Clone, Copy)]
    enum Scenario {
        Unchanged,
        OneFileChanged,
    }

    fn assert_perf_ratio(name: &str, spec: FixtureSpec, scenario: Scenario, threshold: f64) {
        let auto = measure_samples(spec, scenario, false);
        let cold = measure_samples(spec, scenario, true);
        let auto_median = median_duration(&auto);
        let cold_median = median_duration(&cold);
        let ratio = auto_median.as_secs_f64() / cold_median.as_secs_f64();

        assert!(
            ratio <= threshold,
            "{name} ratio {:.3} exceeded threshold {:.3} (auto={:.3}ms cold={:.3}ms)",
            ratio,
            threshold,
            auto_median.as_secs_f64() * 1000.0,
            cold_median.as_secs_f64() * 1000.0,
        );
    }

    fn measure_samples(spec: FixtureSpec, scenario: Scenario, no_cache: bool) -> Vec<Duration> {
        let mut samples = Vec::with_capacity(PERF_SAMPLES);

        for index in 0..(PERF_WARMUPS + PERF_SAMPLES) {
            let dir = tempdir().unwrap();
            write_fixture(dir.path(), spec);
            let cache_path = dir.path().join("cache.json");
            let args = check_args(dir.path(), &cache_path, no_cache);

            if no_cache {
                if matches!(scenario, Scenario::OneFileChanged) {
                    mutate_one_file(dir.path());
                }
                let execution = execute_check(&args).unwrap();
                if index >= PERF_WARMUPS {
                    samples.push(execution.metrics.total_runtime);
                }
                continue;
            }

            execute_check(&args).unwrap();
            if matches!(scenario, Scenario::OneFileChanged) {
                mutate_one_file(dir.path());
            }
            let execution = execute_check(&args).unwrap();
            if index >= PERF_WARMUPS {
                samples.push(execution.metrics.total_runtime);
            }
        }

        samples
    }

    fn median_duration(samples: &[Duration]) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }

    fn check_args(
        path: &std::path::Path,
        cache_path: &std::path::Path,
        no_cache: bool,
    ) -> cli::CheckArgs {
        cli::CheckArgs {
            path: path.to_path_buf(),
            cache_path: cache_path.to_path_buf(),
            ignore: Vec::new(),
            no_cache,
            format: cli::OutputFormat::Compact,
        }
    }

    fn write_fixture(root: &std::path::Path, spec: FixtureSpec) {
        for index in 0..spec.files {
            let mut body = String::new();
            for line in 0..spec.repeats {
                if (index + line) % 11 == 0 {
                    body.push_str(&format!("const value_{index}_{line}: any = {line};\n"));
                } else {
                    body.push_str(&format!("const value_{index}_{line} = {line};\n"));
                }
            }

            fs::write(root.join(format!("file_{index:03}.ts")), body).unwrap();
        }
    }

    fn mutate_one_file(root: &std::path::Path) {
        let path = root.join("file_000.ts");
        let content = fs::read_to_string(&path).unwrap();
        fs::write(&path, format!("{content}\nconsole.log('changed');\n")).unwrap();
    }

    fn default_fingerprint() -> String {
        let config_fingerprint = cli::load_config_with_fingerprint().unwrap().fingerprint;
        rules::hash_bytes(
            format!(
                "{}:{}",
                config_fingerprint,
                crate::project::ProjectInfo::default().fingerprint()
            )
            .as_bytes(),
        )
    }
}
