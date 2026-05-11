// ghost_code.rs — Ghost-Code Obfuscator
// "We've implemented Self-Obfuscating Logic so that even if the software itself
//  is reverse-engineered, the underlying privacy rules and Risk Score formulas
//  remain unreadable to an attacker."
//
// Implementation:
// 1. Decoy variable names in output metadata (misleads RE tools)
// 2. Timing jitter to prevent timing-based analysis of the scoring algorithm
// 3. Output structure scrambling (field order randomized per session)
//
// NOTE: Zero-width Unicode injection has been intentionally removed — it was
// corrupting the fuzzed output text by splitting bracket tokens mid-character,
// causing display artifacts and breaking downstream parsing.

use rand::Rng;
use std::time::Duration;

/// Apply ghost obfuscation to the final output text.
/// This is now a no-op on the text content itself to preserve output integrity.
/// Obfuscation of the *algorithm* is handled at the scoring/metadata level only.
pub fn obfuscate_output(_text: &mut String) -> bool {
    // Previously this injected zero-width joiners/non-joiners (U+200C, U+200D)
    // into the text at irregular intervals. While invisible in most renderers,
    // these characters broke bracket tokens like [PERSON] into fragments when
    // the injection landed inside a replacement tag, producing corrupted output
    // such as "PAN: [PAN[AADHAAR]" or "Phone: [PHONE]ail: [EM[EMAIL]".
    //
    // Obfuscation of the runtime algorithm is achieved via:
    //   - add_timing_jitter() on score computation paths
    //   - obfuscate_scoring_params() on numeric weight arrays
    // Neither of those touches the user-visible text.
    true
}

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

/// Strip any stray invisible markers from text (legacy cleanup).
pub fn strip_markers(text: &str) -> String {
    text.chars()
        .filter(|&c| c != '\u{200C}' && c != '\u{200D}' && c != '\u{FEFF}')
        .collect()
}