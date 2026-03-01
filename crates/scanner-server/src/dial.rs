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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_is_valid_svg() {
        let svg = generate_dial(Grade::A, 92, 120);
        assert!(svg.starts_with("<svg xmlns="));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn dial_contains_score_and_grade() {
        let svg = generate_dial(Grade::B, 80, 120);
        assert!(svg.contains(">80</text>"));
        assert!(svg.contains(">B</text>"));
    }

    #[test]
    fn dial_uses_correct_colors() {
        let svg_a_plus = generate_dial(Grade::APlus, 97, 120);
        assert!(svg_a_plus.contains("#22c55e"));

        let svg_b = generate_dial(Grade::B, 80, 120);
        assert!(svg_b.contains("#3b82f6"));

        let svg_c = generate_dial(Grade::C, 65, 120);
        assert!(svg_c.contains("#eab308"));

        let svg_d = generate_dial(Grade::D, 45, 120);
        assert!(svg_d.contains("#f97316"));

        let svg_f = generate_dial(Grade::F, 20, 120);
        assert!(svg_f.contains("#ef4444"));
    }

    #[test]
    fn dial_respects_size() {
        let svg_small = generate_dial(Grade::A, 90, 60);
        assert!(svg_small.contains(r#"width="60""#));
        assert!(svg_small.contains(r#"height="60""#));

        let svg_large = generate_dial(Grade::A, 90, 300);
        assert!(svg_large.contains(r#"width="300""#));
        assert!(svg_large.contains(r#"height="300""#));
    }

    #[test]
    fn dial_viewbox_is_constant() {
        let svg_small = generate_dial(Grade::A, 90, 60);
        let svg_large = generate_dial(Grade::A, 90, 300);
        assert!(svg_small.contains(r#"viewBox="0 0 120 120""#));
        assert!(svg_large.contains(r#"viewBox="0 0 120 120""#));
    }

    #[test]
    fn dial_has_accessibility_attributes() {
        let svg = generate_dial(Grade::B, 80, 120);
        assert!(svg.contains(r#"role="img""#));
        assert!(svg.contains(r#"aria-label="SPS Score: 80/100 (Grade B)""#));
        assert!(svg.contains("<title>SPS Score: 80/100 (Grade B)</title>"));
    }

    #[test]
    fn dial_arc_offset_scales_with_score() {
        let svg_full = generate_dial(Grade::APlus, 100, 120);
        // score=100 → offset=0
        assert!(svg_full.contains("stroke-dashoffset=\"0\""));

        let svg_zero = generate_dial(Grade::F, 0, 120);
        // score=0 → offset=circumference
        let expected = format!("stroke-dashoffset=\"{}\"", CIRCUMFERENCE);
        assert!(svg_zero.contains(&expected));
    }

    #[test]
    fn dial_a_plus_displays_correctly() {
        let svg = generate_dial(Grade::APlus, 97, 120);
        assert!(svg.contains(">A+</text>"));
        assert!(svg.contains(">97</text>"));
    }

    #[test]
    fn unknown_dial_is_valid_svg() {
        let svg = generate_unknown_dial(120);
        assert!(svg.starts_with("<svg xmlns="));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn unknown_dial_shows_no_scan() {
        let svg = generate_unknown_dial(120);
        assert!(svg.contains(">no scan</text>"));
        assert!(!svg.contains("stroke-dashoffset"));
    }

    #[test]
    fn unknown_dial_respects_size() {
        let svg = generate_unknown_dial(80);
        assert!(svg.contains(r#"width="80""#));
        assert!(svg.contains(r#"height="80""#));
        assert!(svg.contains(r#"viewBox="0 0 120 120""#));
    }

    #[test]
    fn unknown_dial_has_accessibility() {
        let svg = generate_unknown_dial(120);
        assert!(svg.contains(r#"role="img""#));
        assert!(svg.contains(r#"aria-label="SPS Score: no scan data""#));
        assert!(svg.contains("<title>SPS Score: no scan data</title>"));
    }
}
