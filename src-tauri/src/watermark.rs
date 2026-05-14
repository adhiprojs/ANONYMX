// watermark.rs — Digital Forensic Watermarking
// "We've added Forensic Watermarking. If an anonymized file is found where it
//  shouldn't be, we can reverse-engineer the watermark to trace exactly which
//  department or person originally generated that fuzzed report."
//
// Technique: Whitespace Steganography
// We encode a SHA2-based fingerprint (dept_tag + session_id + timestamp)
// as a binary sequence using trailing spaces (0) and tabs (1) on each line.
// Invisible in rendered text. Recoverable with our verifier.

use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::VerifyWatermarkResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatermarkEntry {
    pub id: String,
    pub department_tag: String,
    pub timestamp: String,
}

/// Embed a watermark into the fuzzed text via whitespace steganography
pub fn embed(
    text: &str,
    department_tag: &str,
    session_id: &str,
) -> Result<(String, String), String> {
    let wm_id = Uuid::new_v4().to_string().replace('-', "")[..16].to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Create fingerprint payload (raw string, no hash)
    let payload = format!("CG:{}:{}:{}:{}", wm_id, department_tag, session_id, timestamp);

    // Encode fingerprint as binary string
    let binary = bytes_to_binary(payload.as_bytes());

    // Embed into text lines using zero-width characters
    let watermarked = encode_steganographic(text, &binary);

    Ok((watermarked, wm_id))
}

/// Verify and extract watermark from text
pub fn verify(text: &str) -> VerifyWatermarkResult {
    match decode_steganographic(text) {
        Some(binary) => {
            match binary_to_string(&binary) {
                Some(decoded) if decoded.starts_with("CG:") => {
                    let parts: Vec<&str> = decoded.splitn(5, ':').collect();
                    if parts.len() >= 3 {
                        return VerifyWatermarkResult {
                            found: true,
                            watermark_id: Some(parts[1].to_string()),
                            department_tag: Some(parts[2].to_string()),
                            timestamp: parts.get(4).map(|s| s.to_string()),
                        };
                    }
                    VerifyWatermarkResult { found: true, watermark_id: None, department_tag: None, timestamp: None }
                }
                _ => VerifyWatermarkResult { found: false, watermark_id: None, department_tag: None, timestamp: None }
            }
        }
        None => VerifyWatermarkResult { found: false, watermark_id: None, department_tag: None, timestamp: None }
    }
}

// ── Steganography Internals ───────────────────────────────────────────────────

fn bytes_to_binary(bytes: &[u8]) -> Vec<u8> {
    let mut bits = Vec::new();
    for byte in bytes.iter() {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }
    bits
}

fn binary_to_string(bits: &[u8]) -> Option<String> {
    if bits.len() % 8 != 0 { return None; }
    let mut bytes = Vec::new();
    for chunk in bits.chunks(8) {
        let byte = chunk.iter().fold(0u8, |acc, &b| (acc << 1) | b);
        if byte == 0 { break; }
        bytes.push(byte);
    }
    String::from_utf8(bytes).ok()
}

/// Encode binary using zero-width characters: U+200B (0), U+2060 (1)
/// Wrapped in U+FEFF delimiters, inserted after the first visible character.
fn encode_steganographic(text: &str, binary: &[u8]) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    
    let mut stego_chars = String::new();
    stego_chars.push('\u{FEFF}'); // Start delimiter
    for &bit in binary.iter() {
        if bit == 0 {
            stego_chars.push('\u{200B}'); // Zero-width space
        } else {
            stego_chars.push('\u{2060}'); // Word joiner
        }
    }
    stego_chars.push('\u{FEFF}'); // End delimiter

    // Insert after the first character if possible, otherwise just prepend
    let mut chars = text.chars();
    if let Some(first_char) = chars.next() {
        let mut result = String::new();
        result.push(first_char);
        result.push_str(&stego_chars);
        result.extend(chars);
        result
    } else {
        format!("{}{}", stego_chars, text)
    }
}

/// Decode binary from zero-width characters
fn decode_steganographic(text: &str) -> Option<Vec<u8>> {
    let mut in_payload = false;
    let mut bits = Vec::new();

    for ch in text.chars() {
        if ch == '\u{FEFF}' {
            if in_payload {
                // End delimiter found
                break;
            } else {
                // Start delimiter found
                in_payload = true;
            }
        } else if in_payload {
            if ch == '\u{200B}' {
                bits.push(0);
            } else if ch == '\u{2060}' {
                bits.push(1);
            }
        }
    }

    if bits.is_empty() { None } else { Some(bits) }
}
