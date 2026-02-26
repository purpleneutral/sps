/// Known analytics domains (built-in list, supplements external blocklists).
pub const ANALYTICS_DOMAINS: &[&str] = &[
    "google-analytics.com",
    "analytics.google.com",
    "googletagmanager.com",
    "hotjar.com",
    "static.hotjar.com",
    "plausible.io",
    "analytics.umami.is",
    "matomo.cloud",
    "mixpanel.com",
    "cdn.mxpnl.com",
    "heapanalytics.com",
    "amplitude.com",
    "cdn.amplitude.com",
    "segment.com",
    "cdn.segment.com",
    "fullstory.com",
    "rs.fullstory.com",
    "mouseflow.com",
    "cdn.mouseflow.com",
    "clarity.ms",
    "newrelic.com",
    "js-agent.newrelic.com",
    "nr-data.net",
    "stats.wp.com",
    "pixel.wp.com",
    "luckyorange.com",
    "cdn.luckyorange.com",
    "clicky.com",
    "static.getclicky.com",
    "kissmetrics.com",
    "pendo.io",
    "cdn.pendo.io",
    "logrocket.com",
    "cdn.logrocket.io",
];

/// Known advertising and tracking domains.
pub const TRACKER_DOMAINS: &[&str] = &[
    // Google Ads / DoubleClick
    "googlesyndication.com",
    "doubleclick.net",
    "googleadservices.com",
    "google.com/pagead",
    "adservice.google.com",
    // Facebook / Meta
    "connect.facebook.net",
    "facebook.com/tr",
    "pixel.facebook.com",
    "fbevents.js",
    // Twitter / X
    "static.ads-twitter.com",
    "analytics.twitter.com",
    "t.co",
    // LinkedIn
    "snap.licdn.com",
    "linkedin.com/px",
    // TikTok
    "analytics.tiktok.com",
    // Pinterest
    "ct.pinterest.com",
    // Criteo
    "static.criteo.net",
    "dis.criteo.com",
    // Taboola
    "cdn.taboola.com",
    // Outbrain
    "widgets.outbrain.com",
    // Ad networks
    "amazon-adsystem.com",
    "adsrvr.org",
    "adnxs.com",
    "rubiconproject.com",
    "pubmatic.com",
    "openx.net",
    "casalemedia.com",
    "contextweb.com",
    "bidswitch.net",
    "sharethrough.com",
    "smartadserver.com",
    // Tracking
    "scorecardresearch.com",
    "quantserve.com",
    "bluekai.com",
    "krxd.net",
    "exelator.com",
    "demdex.net",
    "agkn.com",
    "rlcdn.com",
    "turn.com",
    "mookie1.com",
    "intentmedia.net",
];

/// Known CDN domains (not inherently tracking, but third-party resource loading).
pub const CDN_DOMAINS: &[&str] = &[
    "fonts.googleapis.com",
    "fonts.gstatic.com",
    "cdnjs.cloudflare.com",
    "cdn.jsdelivr.net",
    "unpkg.com",
    "ajax.googleapis.com",
    "stackpath.bootstrapcdn.com",
    "maxcdn.bootstrapcdn.com",
    "code.jquery.com",
    "use.fontawesome.com",
    "kit.fontawesome.com",
    "ka-f.fontawesome.com",
    "use.typekit.net",
    "p.typekit.net",
    "cdn.tailwindcss.com",
];

/// Check if a domain matches any entry in a blocklist.
/// Handles subdomain matching (e.g., "scripts.hotjar.com" matches "hotjar.com").
pub fn domain_matches_list(domain: &str, list: &[&str]) -> bool {
    let domain_lower = domain.to_lowercase();
    list.iter().any(|entry| {
        domain_lower == *entry
            || domain_lower.ends_with(&format!(".{entry}"))
    })
}
