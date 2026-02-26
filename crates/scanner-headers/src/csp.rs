use reqwest::header::HeaderMap;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::SecurityHeaders;

/// Check Content-Security-Policy: present, no unsafe-inline, no unsafe-eval.
pub fn check_csp(headers: &HeaderMap) -> Vec<CheckResult> {
    let mut checks = Vec::new();

    let csp_value = headers
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok());

    match csp_value {
        Some(csp) => {
            checks.push(CheckResult::pass(
                CAT,
                "csp_present",
                "Content-Security-Policy present",
                6,
                Some(truncate_for_display(csp, 120)),
            ));

            // Check for unsafe-inline in script-src
            let script_src = extract_directive(csp, "script-src")
                .or_else(|| extract_directive(csp, "default-src"));

            if let Some(directive) = &script_src {
                if directive.contains("'unsafe-inline'") {
                    checks.push(CheckResult::fail(
                        CAT,
                        "csp_no_unsafe_inline",
                        "CSP blocks unsafe-inline",
                        3,
                        Some("script-src contains 'unsafe-inline'".into()),
                    ));
                } else {
                    checks.push(CheckResult::pass(
                        CAT,
                        "csp_no_unsafe_inline",
                        "CSP blocks unsafe-inline",
                        3,
                        None,
                    ));
                }

                if directive.contains("'unsafe-eval'") {
                    checks.push(CheckResult::fail(
                        CAT,
                        "csp_no_unsafe_eval",
                        "CSP blocks unsafe-eval",
                        3,
                        Some("script-src contains 'unsafe-eval'".into()),
                    ));
                } else {
                    checks.push(CheckResult::pass(
                        CAT,
                        "csp_no_unsafe_eval",
                        "CSP blocks unsafe-eval",
                        3,
                        None,
                    ));
                }
            } else {
                // No script-src or default-src — the browser default is permissive
                checks.push(CheckResult::fail(
                    CAT,
                    "csp_no_unsafe_inline",
                    "CSP blocks unsafe-inline",
                    3,
                    Some("No script-src or default-src directive found".into()),
                ));
                checks.push(CheckResult::fail(
                    CAT,
                    "csp_no_unsafe_eval",
                    "CSP blocks unsafe-eval",
                    3,
                    Some("No script-src or default-src directive found".into()),
                ));
            }
        }
        None => {
            checks.push(CheckResult::fail(
                CAT,
                "csp_present",
                "Content-Security-Policy present",
                6,
                Some("Content-Security-Policy header not present".into()),
            ));
            checks.push(CheckResult::fail(
                CAT,
                "csp_no_unsafe_inline",
                "CSP blocks unsafe-inline",
                3,
                None,
            ));
            checks.push(CheckResult::fail(
                CAT,
                "csp_no_unsafe_eval",
                "CSP blocks unsafe-eval",
                3,
                None,
            ));
        }
    }

    checks
}

/// Extract a CSP directive value by name.
fn extract_directive<'a>(csp: &'a str, directive: &str) -> Option<&'a str> {
    for part in csp.split(';') {
        let part = part.trim();
        if part.starts_with(directive) {
            return Some(part);
        }
    }
    None
}

fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}
