use scanner_core::check::CategoryResult;

/// Generate actionable recommendations based on scan results.
pub fn generate(categories: &[CategoryResult]) -> Vec<String> {
    let mut recs = Vec::new();

    for cat in categories {
        for check in &cat.checks {
            if check.passed {
                continue;
            }

            let rec = match check.id.as_str() {
                // Transport
                "tls_1_3_supported" => Some(
                    "Enable TLS 1.3 on your web server for improved security and performance"
                        .to_string(),
                ),
                "tls_legacy_disabled" => Some(
                    "Disable TLS 1.0 and TLS 1.1 — they have known vulnerabilities".to_string(),
                ),
                "hsts_enabled" => Some(
                    "Add the Strict-Transport-Security header to enforce HTTPS connections"
                        .to_string(),
                ),
                "hsts_max_age" => Some(
                    "Set HSTS max-age to at least 31536000 (1 year)".to_string(),
                ),
                "hsts_subdomains" => {
                    Some("Add includeSubDomains to your HSTS header".to_string())
                }
                "hsts_preload" => Some(
                    "Add the preload directive to your HSTS header and submit to hstspreload.org"
                        .to_string(),
                ),

                // Headers
                "csp_present" => Some(
                    "Add a Content-Security-Policy header to control which resources can load"
                        .to_string(),
                ),
                "csp_no_unsafe_inline" => Some(
                    "Remove 'unsafe-inline' from CSP script-src — use nonces or hashes instead"
                        .to_string(),
                ),
                "csp_no_unsafe_eval" => Some(
                    "Remove 'unsafe-eval' from CSP script-src — refactor code to avoid eval()"
                        .to_string(),
                ),
                "referrer_policy" => Some(
                    "Set Referrer-Policy to 'strict-origin-when-cross-origin' or 'no-referrer'"
                        .to_string(),
                ),
                "permissions_policy" => Some(
                    "Set Permissions-Policy to restrict sensitive browser APIs (camera, microphone, geolocation)"
                        .to_string(),
                ),
                "x_content_type_options" => {
                    Some("Set X-Content-Type-Options: nosniff".to_string())
                }
                "x_frame_options" => {
                    Some("Set X-Frame-Options to DENY or SAMEORIGIN".to_string())
                }

                // Tracking
                "no_analytics" => Some(
                    "Remove third-party analytics or replace with privacy-respecting alternatives (Plausible, Umami, self-hosted Matomo)"
                        .to_string(),
                ),
                "no_trackers" => Some(
                    "Remove third-party advertising and tracking scripts".to_string(),
                ),
                "no_fingerprinting" => Some(
                    "Remove fingerprinting scripts and avoid browser fingerprinting techniques"
                        .to_string(),
                ),
                "no_third_party_cdns" => Some(
                    "Self-host fonts and JavaScript libraries instead of using third-party CDNs"
                        .to_string(),
                ),
                "all_https" => Some(
                    "Ensure all external resources are loaded over HTTPS (no mixed content)"
                        .to_string(),
                ),

                // Cookies
                "no_third_party_cookies" => {
                    Some("Remove third-party cookies".to_string())
                }
                "cookies_secure" => {
                    Some("Add the Secure flag to all cookies".to_string())
                }
                "cookies_httponly" => {
                    Some("Add the HttpOnly flag to all cookies that don't need JavaScript access"
                        .to_string())
                }
                "cookies_samesite" => Some(
                    "Add SameSite=Lax or SameSite=Strict to all cookies".to_string(),
                ),
                "cookies_expiration" => Some(
                    "Reduce cookie expiration to 1 year or less".to_string(),
                ),

                // DNS
                "spf_present" => Some(
                    "Set a strict SPF record ending with -all (hard fail)".to_string(),
                ),
                "dkim_present" => Some(
                    "Configure DKIM signing for your email domain".to_string(),
                ),
                "dmarc_policy" => Some(
                    "Set DMARC policy to 'quarantine' or 'reject' instead of 'none'".to_string(),
                ),
                "dnssec_enabled" => {
                    Some("Enable DNSSEC for your domain".to_string())
                }
                "caa_present" => Some(
                    "Add a CAA record to restrict which CAs can issue certificates for your domain"
                        .to_string(),
                ),

                // Best practices
                "security_txt" => Some(
                    "Add a security.txt file at /.well-known/security.txt per RFC 9116".to_string(),
                ),
                "privacy_json" => Some(
                    "Add a privacy.json file at /.well-known/privacy.json (Seglamater Privacy Specification)"
                        .to_string(),
                ),
                "accessible_without_js" => Some(
                    "Ensure your page renders meaningful content without JavaScript (server-side rendering)"
                        .to_string(),
                ),

                _ => None,
            };

            if let Some(r) = rec {
                recs.push(r);
            }
        }
    }

    recs
}
