use scanner_core::spec::Grade;

/// Generate a shields.io-style SVG badge for a privacy grade.
pub fn generate_badge(grade: Grade, score: u32) -> String {
    let color = grade.badge_color_hex();
    let grade_text = grade.to_string();
    let label = "SPS";
    let value = format!("{grade_text} ({score}/100)");

    // Calculate widths (approximate: 6.5px per char + padding)
    let label_width: f32 = label.len() as f32 * 6.5 + 12.0;
    let value_width: f32 = value.len() as f32 * 6.5 + 12.0;
    let total_width = label_width + value_width;

    let label_x = label_width / 2.0;
    let value_x = label_width + value_width / 2.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="20" role="img" aria-label="{label}: {value}">
  <title>{label}: {value}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{total_width}" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_width}" height="20" fill="#555"/>
    <rect x="{label_width}" width="{value_width}" height="20" fill="{color}"/>
    <rect width="{total_width}" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="11">
    <text aria-hidden="true" x="{label_x}" y="15" fill="#010101" fill-opacity=".3">{label}</text>
    <text x="{label_x}" y="14">{label}</text>
    <text aria-hidden="true" x="{value_x}" y="15" fill="#010101" fill-opacity=".3">{value}</text>
    <text x="{value_x}" y="14">{value}</text>
  </g>
</svg>"##
    )
}

/// Generate a placeholder badge when no scan data is available.
pub fn generate_unknown_badge() -> String {
    let label = "SPS";
    let value = "unknown";
    let color = "#9f9f9f";

    let label_width: f32 = label.len() as f32 * 6.5 + 12.0;
    let value_width: f32 = value.len() as f32 * 6.5 + 12.0;
    let total_width = label_width + value_width;

    let label_x = label_width / 2.0;
    let value_x = label_width + value_width / 2.0;

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{total_width}" height="20" role="img" aria-label="{label}: {value}">
  <title>{label}: {value}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{total_width}" height="20" rx="3" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{label_width}" height="20" fill="#555"/>
    <rect x="{label_width}" width="{value_width}" height="20" fill="{color}"/>
    <rect width="{total_width}" height="20" fill="url(#s)"/>
  </g>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="11">
    <text aria-hidden="true" x="{label_x}" y="15" fill="#010101" fill-opacity=".3">{label}</text>
    <text x="{label_x}" y="14">{label}</text>
    <text aria-hidden="true" x="{value_x}" y="15" fill="#010101" fill-opacity=".3">{value}</text>
    <text x="{value_x}" y="14">{value}</text>
  </g>
</svg>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_is_valid_svg() {
        let svg = generate_badge(Grade::A, 92);
        assert!(svg.starts_with("<svg xmlns="));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn badge_contains_score_and_grade() {
        let svg = generate_badge(Grade::B, 80);
        assert!(svg.contains("B (80/100)"));
    }

    #[test]
    fn badge_uses_correct_colors() {
        let svg_a_plus = generate_badge(Grade::APlus, 97);
        assert!(svg_a_plus.contains("#4c1"));

        let svg_a = generate_badge(Grade::A, 92);
        assert!(svg_a.contains("#97ca00"));

        let svg_b = generate_badge(Grade::B, 80);
        assert!(svg_b.contains("#007ec6"));

        let svg_c = generate_badge(Grade::C, 65);
        assert!(svg_c.contains("#dfb317"));

        let svg_d = generate_badge(Grade::D, 45);
        assert!(svg_d.contains("#fe7d37"));

        let svg_f = generate_badge(Grade::F, 20);
        assert!(svg_f.contains("#e05d44"));
    }

    #[test]
    fn badge_has_accessibility_attributes() {
        let svg = generate_badge(Grade::B, 80);
        assert!(svg.contains(r#"role="img""#));
        assert!(svg.contains(r#"aria-label="SPS: B (80/100)""#));
        assert!(svg.contains("<title>SPS: B (80/100)</title>"));
    }

    #[test]
    fn badge_a_plus_displays_correctly() {
        let svg = generate_badge(Grade::APlus, 97);
        assert!(svg.contains("A+ (97/100)"));
    }

    #[test]
    fn badge_has_fixed_height() {
        let svg = generate_badge(Grade::A, 90);
        assert!(svg.contains(r#"height="20""#));
    }

    #[test]
    fn badge_label_is_sps() {
        let svg = generate_badge(Grade::A, 90);
        assert!(svg.contains(">SPS</text>"));
    }

    #[test]
    fn unknown_badge_is_valid_svg() {
        let svg = generate_unknown_badge();
        assert!(svg.starts_with("<svg xmlns="));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn unknown_badge_shows_unknown() {
        let svg = generate_unknown_badge();
        assert!(svg.contains("unknown"));
    }

    #[test]
    fn unknown_badge_uses_grey() {
        let svg = generate_unknown_badge();
        assert!(svg.contains("#9f9f9f"));
    }

    #[test]
    fn unknown_badge_has_accessibility() {
        let svg = generate_unknown_badge();
        assert!(svg.contains(r#"role="img""#));
        assert!(svg.contains(r#"aria-label="SPS: unknown""#));
        assert!(svg.contains("<title>SPS: unknown</title>"));
    }
}
