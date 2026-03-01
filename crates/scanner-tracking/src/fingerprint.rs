use regex::Regex;
use std::sync::LazyLock;

/// Known fingerprinting patterns to detect in inline scripts.
struct FingerprintPattern {
    name: &'static str,
    pattern: &'static str,
}

const PATTERNS: &[FingerprintPattern] = &[
    FingerprintPattern {
        name: "Canvas fingerprinting",
        pattern: r"canvas.*toDataURL|toDataURL.*canvas|getImageData",
    },
    FingerprintPattern {
        name: "WebGL fingerprinting",
        pattern: r#"UNMASKED_VENDOR_WEBGL|UNMASKED_RENDERER_WEBGL|getExtension\s*\(\s*['"]WEBGL_debug_renderer_info['"]\s*\)"#,
    },
    FingerprintPattern {
        name: "AudioContext fingerprinting",
        pattern: r"AudioContext|OfflineAudioContext|createOscillator.*createDynamicsCompressor",
    },
    FingerprintPattern {
        name: "FingerprintJS library",
        pattern: r"fingerprintjs|FingerprintJS|fpjs",
    },
    FingerprintPattern {
        name: "Hardware enumeration",
        pattern: r"navigator\.hardwareConcurrency.*navigator\.deviceMemory|navigator\.deviceMemory.*navigator\.hardwareConcurrency",
    },
];

static COMPILED_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p.pattern).ok().map(|re| (p.name, re)))
        .collect()
});

/// Detect fingerprinting patterns in HTML (particularly in inline scripts).
/// Returns a list of detected technique names.
pub fn detect_fingerprinting(html: &str) -> Vec<String> {
    let mut found = Vec::new();

    for (name, re) in COMPILED_PATTERNS.iter() {
        if re.is_match(html) {
            found.push((*name).to_string());
        }
    }

    found
}
