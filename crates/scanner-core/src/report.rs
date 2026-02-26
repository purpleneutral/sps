use crate::score::ScanResult;
use colored::Colorize;
use std::fmt::Write;

/// Format a scan result as human-readable colored text.
pub fn format_text(result: &ScanResult) -> String {
    let mut out = String::new();

    writeln!(out, "Seglamater Privacy Scan — {}", result.domain).unwrap();
    writeln!(out, "Specification: SPS {}", result.spec_version).unwrap();
    writeln!(
        out,
        "Scanned: {}",
        result.scanned_at.format("%Y-%m-%dT%H:%M:%SZ")
    )
    .unwrap();
    writeln!(out).unwrap();

    let grade_str = format!("Score: {}/100 (Grade: {})", result.total_score, result.grade);
    let grade_str = match result.grade {
        crate::spec::Grade::APlus | crate::spec::Grade::A => grade_str.green().bold().to_string(),
        crate::spec::Grade::B => grade_str.blue().bold().to_string(),
        crate::spec::Grade::C => grade_str.yellow().bold().to_string(),
        crate::spec::Grade::D => grade_str.bright_red().bold().to_string(),
        crate::spec::Grade::F => grade_str.red().bold().to_string(),
    };
    writeln!(out, "{grade_str}").unwrap();
    writeln!(out).unwrap();

    for cat in &result.categories {
        let header = format!(
            "{:<44} {}/{}",
            cat.category.display_name(),
            cat.points_awarded(),
            cat.max_points()
        );
        writeln!(out, "{}", header.bold()).unwrap();

        for check in &cat.checks {
            let status = if check.passed {
                "PASS".green().to_string()
            } else {
                "FAIL".red().to_string()
            };
            let points = format!("[{}]", check.points_awarded);

            let detail = check
                .detail
                .as_ref()
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();

            writeln!(
                out,
                "  {status}  {points:>4} {}{}",
                check.description, detail
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    if !result.recommendations.is_empty() {
        writeln!(out, "{}", "RECOMMENDATIONS:".bold()).unwrap();
        for (i, rec) in result.recommendations.iter().enumerate() {
            writeln!(out, "  {}. {rec}", i + 1).unwrap();
        }
    }

    out
}

/// Format a scan result as JSON.
pub fn format_json(result: &ScanResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}
