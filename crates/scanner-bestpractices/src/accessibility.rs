use scanner_core::browser_types::BrowserData;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::BestPractices;

/// Check if the page returns meaningful content without JavaScript.
///
/// Heuristic: the initial HTML response (without JS execution) should contain
/// meaningful text content — not just a loading spinner or empty div.
/// When browser data is available, uses rendered HTML to confirm SPA detection.
pub fn check_js_free(html: &str, browser_data: Option<&BrowserData>) -> CheckResult {
    let static_meaningful = has_meaningful_content(html);

    if static_meaningful {
        return CheckResult::pass(
            CAT,
            "accessible_without_js",
            "Accessible without JavaScript",
            1,
            None,
        );
    }

    // Static HTML lacks content — check browser data for confirmation
    let detail = match browser_data {
        Some(bd) if has_meaningful_content(&bd.rendered_html) => {
            "Page requires JavaScript: static HTML has no content, \
             but rendered page does (confirmed SPA)"
                .into()
        }
        _ => "Page appears to require JavaScript for content (likely a SPA)".into(),
    };

    CheckResult::fail(
        CAT,
        "accessible_without_js",
        "Accessible without JavaScript",
        1,
        Some(detail),
    )
}

/// Determine if HTML contains meaningful content without JS execution.
fn has_meaningful_content(html: &str) -> bool {
    let lower = html.to_lowercase();

    // Common SPA indicators: the body is nearly empty or contains only a root div
    let spa_indicators = [
        r#"<div id="root"></div>"#,
        r#"<div id="app"></div>"#,
        r#"<div id="__next"></div>"#,
        r#"<div id="__nuxt"></div>"#,
        r#"<div id="root">"#,
        "loading...",
        "please enable javascript",
        "you need to enable javascript",
        "this app requires javascript",
    ];

    for indicator in &spa_indicators {
        if lower.contains(indicator) {
            // Check if there's also substantial text content alongside the indicator
            let text_content = strip_tags_approximate(&lower);
            let word_count = text_content.split_whitespace().count();
            // If there are very few words besides the SPA indicator, it's JS-dependent
            if word_count < 50 {
                return false;
            }
        }
    }

    // Check if the body has at least some meaningful text content
    let text = strip_tags_approximate(&lower);
    let word_count = text.split_whitespace().count();

    // A page with fewer than 20 words (excluding markup) is likely JS-dependent
    word_count >= 20
}

/// Very rough tag stripping for content analysis. Not a full HTML parser.
fn strip_tags_approximate(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if !in_tag && chars[i] == '<' {
            in_tag = true;
            // Check for script/style opening tags
            let remaining: String = chars[i..].iter().take(20).collect();
            if remaining.starts_with("<script") {
                in_script = true;
            } else if remaining.starts_with("<style") {
                in_style = true;
            } else if remaining.starts_with("</script") {
                in_script = false;
            } else if remaining.starts_with("</style") {
                in_style = false;
            }
        } else if in_tag && chars[i] == '>' {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            result.push(chars[i]);
        }
        i += 1;
    }

    result
}
