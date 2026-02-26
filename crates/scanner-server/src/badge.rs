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
