use serde::{Deserialize, Serialize};
use std::fmt;

/// The Seglamater Privacy Specification version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecVersion {
    pub major: u32,
    pub minor: u32,
}

impl SpecVersion {
    pub const V1_0: Self = Self { major: 1, minor: 0 };

    pub fn current() -> Self {
        Self::V1_0
    }
}

impl fmt::Display for SpecVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

/// Scoring categories defined by the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    TransportSecurity,
    SecurityHeaders,
    TrackingThirdParties,
    CookieBehavior,
    EmailDnsSecurity,
    BestPractices,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::TransportSecurity,
        Category::SecurityHeaders,
        Category::TrackingThirdParties,
        Category::CookieBehavior,
        Category::EmailDnsSecurity,
        Category::BestPractices,
    ];

    /// Maximum points available in this category (SPS v1.0).
    pub fn max_points(self) -> u32 {
        match self {
            Category::TransportSecurity => 20,
            Category::SecurityHeaders => 20,
            Category::TrackingThirdParties => 30,
            Category::CookieBehavior => 15,
            Category::EmailDnsSecurity => 10,
            Category::BestPractices => 5,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Category::TransportSecurity => "TRANSPORT SECURITY",
            Category::SecurityHeaders => "SECURITY HEADERS",
            Category::TrackingThirdParties => "TRACKING & THIRD PARTIES",
            Category::CookieBehavior => "COOKIE BEHAVIOR",
            Category::EmailDnsSecurity => "EMAIL & DNS SECURITY",
            Category::BestPractices => "BEST PRACTICES",
        }
    }
}

/// Grade thresholds per SPS v1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    F,
    D,
    C,
    B,
    A,
    APlus,
}

impl Grade {
    pub fn from_score(score: u32) -> Self {
        match score {
            95..=100 => Grade::APlus,
            90..=94 => Grade::A,
            75..=89 => Grade::B,
            60..=74 => Grade::C,
            40..=59 => Grade::D,
            _ => Grade::F,
        }
    }

    pub fn color_name(self) -> &'static str {
        match self {
            Grade::APlus => "bright green",
            Grade::A => "green",
            Grade::B => "blue",
            Grade::C => "yellow",
            Grade::D => "orange",
            Grade::F => "red",
        }
    }

    pub fn is_verified(self) -> bool {
        matches!(self, Grade::APlus | Grade::A)
    }

    /// Hex color for badge SVG rendering (shields.io style).
    pub fn badge_color_hex(self) -> &'static str {
        match self {
            Grade::APlus => "#4c1",
            Grade::A => "#97ca00",
            Grade::B => "#007ec6",
            Grade::C => "#dfb317",
            Grade::D => "#fe7d37",
            Grade::F => "#e05d44",
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}
