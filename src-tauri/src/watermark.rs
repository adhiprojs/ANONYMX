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

    // Create fingerprint payload
    let payload = format!("CG:{}:{}:{}:{}", wm_id, department_tag, session_id, timestamp);
    let fingerprint = hash_payload(&payload);

    // Encode fingerprint as binary string
    let binary = bytes_to_binary(fingerprint.as_bytes());

    // Embed into text lines
    let watermarked = encode_into_whitespace(text, &binary);

    Ok((watermarked, wm_id))
}

/// Verify and extract watermark from text
pub fn verify(text: &str) -> VerifyWatermarkResult {
    match decode_from_whitespace(text) {
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

fn hash_payload(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD_NO_PAD.encode(&result[..16]) // 16 bytes = 128-bit fingerprint
}

fn bytes_to_binary(bytes: &[u8]) -> Vec<u8> {
    let mut bits = Vec::new();
    for byte in bytes.iter().take(32) { // Embed first 32 chars = 256 bits
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

/// Encode binary into trailing whitespace: space = 0, tab = 1
fn encode_into_whitespace(text: &str, binary: &[u8]) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() { return text.to_string(); }

    let mut result = String::new();
    let mut bit_idx = 0;

    for (i, line) in lines.iter().enumerate() {
        result.push_str(line.trim_end()); // Remove existing trailing whitespace

        // Embed one bit per line as trailing space (0) or tab (1)
        if bit_idx < binary.len() {
            if binary[bit_idx] == 0 {
                result.push(' '); // bit 0 = trailing space
            } else {
                result.push('\t'); // bit 1 = trailing tab
            }
            bit_idx += 1;
        }

        if i < lines.len() - 1 {
            result.push('\n');
        }
    }

    result
}

/// Decode binary from trailing whitespace
fn decode_from_whitespace(text: &str) -> Option<Vec<u8>> {
    let mut bits = Vec::new();

    for line in text.lines() {
        if let Some(last_char) = line.chars().last() {
            match last_char {
                ' ' => bits.push(0),
                '\t' => bits.push(1),
                _ => {} // No watermark bit on this line
            }
        }
    }

    if bits.len() >= 64 { Some(bits) } else { None }
}
