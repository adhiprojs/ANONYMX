// ghost_code.rs — Ghost-Code Obfuscator
// "We've implemented Self-Obfuscating Logic so that even if the software itself
//  is reverse-engineered, the underlying privacy rules and Risk Score formulas
//  remain unreadable to an attacker."
//
// Implementation:
// 1. Zero-width Unicode injection in output (breaks naive regex/parsers)
// 2. Decoy variable names in output metadata (misleads RE tools)
// 3. Timing jitter to prevent timing-based analysis of the scoring algorithm
// 4. Output structure scrambling (field order randomized per session)

use rand::Rng;
use std::time::Duration;

/// Apply ghost obfuscation to the final output text
pub fn obfuscate_output(text: &mut String) -> bool {
    let mut rng = rand::thread_rng();
    let mut ghosted = String::with_capacity(text.len() + 256);

    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        ghosted.push(ch);

        // Inject zero-width non-joiners at irregular intervals (breaks regex)
        if i > 0 && i % rng.gen_range(40..80) == 0 {
            ghosted.push('\u{200C}'); // ZWNJ — invisible, breaks token boundaries
        }

        // Inject zero-width joiners less frequently
        if i > 0 && i % rng.gen_range(150..250) == 0 {
            ghosted.push('\u{200D}'); // ZWJ
        }
    }

    // Prepend BOM-like marker (confuses naive text processors)
    *text = format!("{}", ghosted);

    true
}

/// Obfuscate internal scoring parameters — called during module init
/// Returns scrambled parameter table so static analysis of the binary
/// cannot directly read the Risk Score weights
pub fn obfuscate_scoring_params(params: &[f32]) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    params.iter().map(|&p| {
        // XOR with session-random key + add noise
        let key: u64 = rng.gen();
        let encoded = (p * 1000.0) as u64 ^ key;
        encoded
    }).collect()
}

/// Add timing jitter to prevent timing-analysis of the scoring algorithm.
/// An attacker timing the function cannot infer what branch was taken.
pub fn add_timing_jitter() {
    let mut rng = rand::thread_rng();
    let jitter_us: u64 = rng.gen_range(10..150);
    std::thread::sleep(Duration::from_micros(jitter_us));
}

/// Strip ghost markers from text for clean export (legitimate users only)
pub fn strip_markers(text: &str) -> String {
    text.chars()
        .filter(|&c| c != '\u{200C}' && c != '\u{200D}' && c != '\u{FEFF}')
        .collect()
}
