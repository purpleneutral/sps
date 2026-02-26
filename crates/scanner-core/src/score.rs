use crate::check::CategoryResult;
use crate::spec::{Grade, SpecVersion};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete scan result for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// The domain that was scanned.
    pub domain: String,
    /// Specification version used for this scan.
    pub spec_version: SpecVersion,
    /// When the scan was performed.
    pub scanned_at: DateTime<Utc>,
    /// Results per category.
    pub categories: Vec<CategoryResult>,
    /// Total score (0-100).
    pub total_score: u32,
    /// Computed grade.
    pub grade: Grade,
    /// Actionable recommendations.
    pub recommendations: Vec<String>,
}

impl ScanResult {
    /// Build a ScanResult from category results. Computes total score, grade, and recommendations.
    pub fn from_categories(
        domain: String,
        categories: Vec<CategoryResult>,
        recommendations: Vec<String>,
    ) -> Self {
        let total_score: u32 = categories.iter().map(|c| c.points_awarded()).sum();
        let grade = Grade::from_score(total_score);

        Self {
            domain,
            spec_version: SpecVersion::current(),
            scanned_at: Utc::now(),
            categories,
            total_score,
            grade,
            recommendations,
        }
    }
}
