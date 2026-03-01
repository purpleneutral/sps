use reqwest::header::HeaderMap;
use scanner_core::check::CheckResult;
use scanner_core::spec::Category;

const CAT: Category = Category::SecurityHeaders;

/// Check X-Content-Type-Options and X-Frame-Options.
pub fn check_misc_headers(headers: &HeaderMap) -> Vec<CheckResult> {
    vec![check_xcto(headers), check_xfo(headers)]
}

fn check_xcto(headers: &HeaderMap) -> CheckResult {
    let value = headers
        .get("x-content-type-options")
        .and_then(|v| v.to_str().ok());

    match value {
        Some(v) if v.eq_ignore_ascii_case("nosniff") => CheckResult::pass(
            CAT,
            "x_content_type_options",
            "X-Content-Type-Options",
            1,
            Some("nosniff".into()),
        ),
        Some(v) => CheckResult::fail(
            CAT,
            "x_content_type_options",
            "X-Content-Type-Options",
            1,
            Some(format!("Value is \"{v}\" (expected \"nosniff\")")),
        ),
        None => CheckResult::fail(
            CAT,
            "x_content_type_options",
            "X-Content-Type-Options",
            1,
            Some("Header not present".into()),
        ),
    }
}

fn check_xfo(headers: &HeaderMap) -> CheckResult {
    let value = headers.get("x-frame-options").and_then(|v| v.to_str().ok());

    match value {
        Some(v) if v.eq_ignore_ascii_case("deny") || v.eq_ignore_ascii_case("sameorigin") => {
            CheckResult::pass(
                CAT,
                "x_frame_options",
                "X-Frame-Options",
                1,
                Some(v.to_string()),
            )
        }
        Some(v) => CheckResult::fail(
            CAT,
            "x_frame_options",
            "X-Frame-Options",
            1,
            Some(format!("Value is \"{v}\" (expected DENY or SAMEORIGIN)")),
        ),
        None => CheckResult::fail(
            CAT,
            "x_frame_options",
            "X-Frame-Options",
            1,
            Some("Header not present".into()),
        ),
    }
}
