use crate::rules::LintResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthScore {
    pub score: u32,
    pub errors: usize,
    pub warnings: usize,
    pub files_scanned: usize,
    pub density: f64,
}

impl HealthScore {
    pub fn compute(results: &[LintResult], files_scanned: usize) -> Self {
        let (errors, warnings) = crate::diagnostic_counts(results);
        let weighted = errors * 5 + warnings;
        let files = files_scanned.max(1) as f64;
        let density = weighted as f64 / files;
        let deduction = ((density * 10.0 + 1.0).ln() * 15.0).min(100.0);
        let score = (100.0 - deduction).max(0.0).round() as u32;

        Self {
            score,
            errors,
            warnings,
            files_scanned,
            density,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.score {
            90..=100 => "healthy",
            70..=89 => "needs attention",
            _ => "needs work",
        }
    }

    pub fn ascii_face(&self) -> &'static str {
        match self.score {
            90..=100 => "(^_^)",
            70..=89 => "(o_o)",
            _ => "(>_<)",
        }
    }

    pub fn progress_bar(&self) -> String {
        let filled = (self.score as usize) / 10;
        let empty = 10 - filled;
        format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{LintDiagnostic, RuleOrigin, Severity};
    use std::path::PathBuf;

    fn make_results(errors: usize, warnings: usize) -> Vec<LintResult> {
        let mut diagnostics = Vec::new();
        for _ in 0..errors {
            diagnostics.push(LintDiagnostic {
                rule_name: "test-error".to_string(),
                message: "err".to_string(),
                span: "1:1".to_string(),
                severity: Severity::Error,
                origin: RuleOrigin::BuiltIn,
                fix: None,
                byte_start: 0,
                byte_end: 0,
                node_kind: None,
                symbol: None,
            });
        }
        for _ in 0..warnings {
            diagnostics.push(LintDiagnostic {
                rule_name: "test-warn".to_string(),
                message: "warn".to_string(),
                span: "1:1".to_string(),
                severity: Severity::Warning,
                origin: RuleOrigin::BuiltIn,
                fix: None,
                byte_start: 0,
                byte_end: 0,
                node_kind: None,
                symbol: None,
            });
        }
        vec![LintResult {
            file: PathBuf::from("test.ts"),
            diagnostics,
        }]
    }

    #[test]
    fn zero_issues_scores_100() {
        let score = HealthScore::compute(&[], 10);
        assert_eq!(score.score, 100);
        assert_eq!(score.label(), "healthy");
    }

    #[test]
    fn zero_files_scores_100() {
        let score = HealthScore::compute(&[], 0);
        assert_eq!(score.score, 100);
    }

    #[test]
    fn few_warnings_in_large_project_stays_high() {
        let results = make_results(0, 3);
        let score = HealthScore::compute(&results, 100);
        assert!(score.score >= 90, "score was {}", score.score);
    }

    #[test]
    fn many_errors_in_small_project_is_low() {
        let results = make_results(20, 10);
        let score = HealthScore::compute(&results, 5);
        assert!(score.score < 70, "score was {}", score.score);
    }

    #[test]
    fn score_never_exceeds_100() {
        let score = HealthScore::compute(&[], 1000);
        assert!(score.score <= 100);
    }

    #[test]
    fn score_never_below_0() {
        let results = make_results(1000, 1000);
        let score = HealthScore::compute(&results, 1);
        assert_eq!(score.score, 0);
    }

    #[test]
    fn label_ranges() {
        assert_eq!(HealthScore::compute(&[], 10).label(), "healthy");

        let mid = make_results(2, 5);
        let score = HealthScore::compute(&mid, 10);
        // Just verify it returns a valid label
        assert!(!score.label().is_empty());
    }

    #[test]
    fn ascii_face_tiers() {
        let healthy = HealthScore::compute(&[], 10);
        assert_eq!(healthy.ascii_face(), "(^_^)");

        let mid = make_results(0, 3);
        let attention = HealthScore::compute(&mid, 5);
        assert!(
            attention.score >= 70 && attention.score < 90,
            "expected 70-89, got {}",
            attention.score
        );
        assert_eq!(attention.ascii_face(), "(o_o)");

        let bad = make_results(20, 10);
        let work = HealthScore::compute(&bad, 5);
        assert!(work.score < 70);
        assert_eq!(work.ascii_face(), "(>_<)");
    }

    #[test]
    fn progress_bar_rendering() {
        let full = HealthScore::compute(&[], 10);
        assert_eq!(full.progress_bar(), "[##########]");

        let empty = make_results(1000, 1000);
        let zero = HealthScore::compute(&empty, 1);
        assert_eq!(zero.progress_bar(), "[----------]");
    }
}
