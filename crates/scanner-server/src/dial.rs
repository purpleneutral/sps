use scanner_core::spec::Grade;
use std::f64::consts::PI;

const RADIUS: f64 = 54.0;
const CIRCUMFERENCE: f64 = 2.0 * PI * RADIUS;

/// Generate a circular score dial SVG for a privacy grade.
pub fn generate_dial(grade: Grade, score: u32, size: u32) -> String {
    let color = grade.dial_color_hex();
    let grade_text = grade.to_string();
    let fraction = score as f64 / 100.0;
    let offset = CIRCUMFERENCE * (1.0 - fraction);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="{size}" height="{size}" role="img" aria-label="SPS Score: {score}/100 (Grade {grade_text})">
  <title>SPS Score: {score}/100 (Grade {grade_text})</title>
  <circle cx="60" cy="60" r="60" fill="#1a1a2e"/>
  <circle cx="60" cy="60" r="{RADIUS}" fill="none" stroke="#333" stroke-width="6"/>
  <circle cx="60" cy="60" r="{RADIUS}" fill="none" stroke="{color}" stroke-width="6" stroke-linecap="round" stroke-dasharray="{CIRCUMFERENCE}" stroke-dashoffset="{offset}" transform="rotate(-90 60 60)"/>
  <text x="60" y="50" text-anchor="middle" dominant-baseline="central" font-family="system-ui,-apple-system,sans-serif" font-size="32" font-weight="700" fill="{color}">{score}</text>
  <text x="60" y="80" text-anchor="middle" font-family="system-ui,-apple-system,sans-serif" font-size="14" fill="#888">{grade_text}</text>
  <text x="60" y="108" text-anchor="middle" font-family="system-ui,-apple-system,sans-serif" font-size="8" fill="#888" opacity="0.6">SPS</text>
</svg>"##
    )
}

/// Generate a placeholder dial when no scan data is available.
pub fn generate_unknown_dial(size: u32) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="{size}" height="{size}" role="img" aria-label="SPS Score: no scan data">
  <title>SPS Score: no scan data</title>
  <circle cx="60" cy="60" r="60" fill="#1a1a2e"/>
  <circle cx="60" cy="60" r="{RADIUS}" fill="none" stroke="#333" stroke-width="6"/>
  <text x="60" y="55" text-anchor="middle" dominant-baseline="central" font-family="system-ui,-apple-system,sans-serif" font-size="14" fill="#888">no scan</text>
  <text x="60" y="108" text-anchor="middle" font-family="system-ui,-apple-system,sans-serif" font-size="8" fill="#888" opacity="0.6">SPS</text>
</svg>"##
    )
}
