use crate::spec::Category;
use serde::{Deserialize, Serialize};

/// The result of a single check within a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Which category this check belongs to.
    pub category: Category,
    /// Unique identifier for this check (e.g., "tls_1_3_supported").
    pub id: String,
    /// Human-readable description of what was checked.
    pub description: String,
    /// Maximum points this check is worth.
    pub max_points: u32,
    /// Points awarded (0 if failed, max_points if passed, partial for some checks).
    pub points_awarded: u32,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional detail message (e.g., the actual header value found, or why it failed).
    pub detail: Option<String>,
}

impl CheckResult {
    pub fn pass(
        category: Category,
        id: impl Into<String>,
        description: impl Into<String>,
        points: u32,
        detail: Option<String>,
    ) -> Self {
        Self {
            category,
            id: id.into(),
            description: description.into(),
            max_points: points,
            points_awarded: points,
            passed: true,
            detail,
        }
    }

    pub fn fail(
        category: Category,
        id: impl Into<String>,
        description: impl Into<String>,
        max_points: u32,
        detail: Option<String>,
    ) -> Self {
        Self {
            category,
            id: id.into(),
            description: description.into(),
            max_points,
            points_awarded: 0,
            passed: false,
            detail,
        }
    }
}

/// Results for one entire category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: Category,
    pub checks: Vec<CheckResult>,
}

impl CategoryResult {
    pub fn new(category: Category, checks: Vec<CheckResult>) -> Self {
        Self { category, checks }
    }

    pub fn points_awarded(&self) -> u32 {
        self.checks.iter().map(|c| c.points_awarded).sum()
    }

    pub fn max_points(&self) -> u32 {
        self.category.max_points()
    }
}
