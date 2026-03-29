pub mod cli;
pub mod rules;

use clap::Parser;
use colored::*;
use miette::Result;
use std::time::Instant;

pub fn run() -> Result<i32> {
    let cli = cli::Cli::parse();
    let start = Instant::now();

    let exit_code = match &cli.command {
        cli::Command::Check(args) => run_check(args, start)?,
        cli::Command::Init => {
            cli::init_config()?;
            0
        }
    };

    if cli.timing {
        let elapsed = start.elapsed();
        eprintln!("\n⚡ Zarc finished in {:.2?}", elapsed);
    }

    Ok(exit_code)
}

fn run_check(args: &cli::CheckArgs, start: Instant) -> Result<i32> {
    use rayon::prelude::*;

    let config = cli::load_config()?;
    let files = cli::discover_files(&args.path, &config.files.exclude, &args.ignore)?;
    let cache = if args.no_cache {
        rules::Cache::default()
    } else {
        rules::Cache::load(&args.cache_path)?
    };

    let results: Vec<rules::LintResult> = files
        .par_iter()
        .filter_map(|file| {
            if !args.no_cache {
                if let Some(cached) = cache.get(file) {
                    if cached.hash == rules::hash_file(file) {
                        return Some(rules::apply_severity_overrides(
                            cached.result.clone(),
                            &config.lint.rules,
                        ));
                    }
                }
            }

            match rules::lint_file(file) {
                Ok(result) => Some(rules::apply_severity_overrides(result, &config.lint.rules)),
                Err(e) => {
                    eprintln!("Error linting {}: {}", file.display(), e);
                    None
                }
            }
        })
        .collect();

    if !args.no_cache {
        cache.save(&results, &args.cache_path)?;
    }

    let summary = print_results(&results, &args.format, start.elapsed());
    Ok(if summary.errors > 0 { 1 } else { 0 })
}

struct Summary {
    errors: usize,
}

fn print_results(
    results: &[rules::LintResult],
    format: &cli::OutputFormat,
    elapsed: std::time::Duration,
) -> Summary {
    match format {
        cli::OutputFormat::Json => print_json(results),
        cli::OutputFormat::Compact => print_compact(results),
        cli::OutputFormat::Pretty => print_pretty(results, elapsed),
    }
}

fn print_pretty(results: &[rules::LintResult], elapsed: std::time::Duration) -> Summary {
    let mut total_errors = 0;
    let mut total_warnings = 0;

    for result in results {
        for diagnostic in &result.diagnostics {
            match diagnostic.severity {
                rules::Severity::Error => {
                    total_errors += 1;
                    println!(
                        "  {} {}:{} {}",
                        "error".red().bold(),
                        result.file.display(),
                        diagnostic.span,
                        diagnostic.message
                    );
                }
                rules::Severity::Warning => {
                    total_warnings += 1;
                    println!(
                        "  {} {}:{} {}",
                        "warn".yellow().bold(),
                        result.file.display(),
                        diagnostic.span,
                        diagnostic.message
                    );
                }
            }
            println!("    {} {}", "rule".dimmed(), diagnostic.rule_name);
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
