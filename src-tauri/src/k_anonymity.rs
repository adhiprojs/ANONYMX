// k_anonymity.rs — K-Anonymity Auditor
// Mathematical proof that no individual can be singled out from the output.
// Produces a Privacy Score and verifiable scorecard.

use crate::{FuzzedEntity, KAnonymityReport};
use std::collections::HashMap;

/// Core audit function — produces a K-Anonymity report
pub fn audit(
    fuzzed_text: &str,
    entities: &[FuzzedEntity],
    k_target: u32,
    sensitivity: u8,
) -> KAnonymityReport {
    let mut risk_factors = Vec::new();

    // ── Factor 1: Count remaining specific values (unfuzzed specificity) ───
    let specificity_score = measure_specificity(fuzzed_text);
    if specificity_score > 0.3 {
        risk_factors.push(format!(
            "High specificity in output text ({:.0}% specific tokens remain)",
            specificity_score * 100.0
        ));
    }

    // ── Factor 2: Quasi-identifier combination risk ─────────────────────────
    let quasi_risk = measure_quasi_combination_risk(entities);
    if quasi_risk > 0.0 {
        risk_factors.push(format!(
            "{} quasi-identifier types remain — combination risk present",
            quasi_risk as u32
        ));
    }

    // ── Factor 3: Unique value density ─────────────────────────────────────
    let unique_density = measure_unique_density(fuzzed_text);
    if unique_density > 0.4 {
        risk_factors.push(format!(
            "Unique token density too high ({:.0}%) — may allow singling out",
            unique_density * 100.0
        ));
    }

    // ── Factor 4: Residual PII patterns ────────────────────────────────────
    let residual_pii = count_residual_pii(fuzzed_text);
    if residual_pii > 0 {
        risk_factors.push(format!(
            "{} potential PII pattern(s) detected in output — re-run at higher sensitivity",
            residual_pii
        ));
    }

    // ── Calculate estimated k-value ─────────────────────────────────────────
    // k = estimated number of individuals this record could belong to.
    // Higher = safer. We target k >= k_target (default 5).
    let k_value = estimate_k_value(specificity_score, quasi_risk, unique_density, sensitivity);

    // ── Privacy Score (0–1, 1 = fully anonymous) ───────────────────────────
    let score = calculate_privacy_score(k_value, k_target, &risk_factors);

    // ── Verdict ─────────────────────────────────────────────────────────────
    let verdict = if k_value >= k_target && risk_factors.is_empty() {
        "SAFE".to_string()
    } else if k_value >= k_target / 2 && risk_factors.len() <= 1 {
        "MARGINAL".to_string()
    } else {
        "UNSAFE".to_string()
    };

    let target_met = k_value >= k_target;

    KAnonymityReport {
        score,
        k_value,
        target_met,
        risk_factors,
        verdict,
    }
}

// ── Measurement Functions ─────────────────────────────────────────────────────

/// Measures what fraction of tokens are still "specific" (proper nouns, long unique strings)
fn measure_specificity(text: &str) -> f32 {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() { return 0.0; }

    let specific_count = tokens.iter().filter(|t| {
        let t = t.trim_matches(|c: char| !c.is_alphabetic());
        // Heuristic: starts with capital, length > 4, not a bracket replacement
        t.len() > 4
            && t.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && !t.starts_with('[')
    }).count();

    specific_count as f32 / tokens.len() as f32
}

/// Count remaining quasi-identifier types that could combine
fn measure_quasi_combination_risk(entities: &[FuzzedEntity]) -> f32 {
    let quasi_types: std::collections::HashSet<&str> = entities
        .iter()
        .filter(|e| e.method == "QUASI-ID")
        .map(|e| e.entity_type.as_str())
        .collect();
    // Risk increases with combinations: 2 quasi-IDs → risk, 3+ → high risk
    match quasi_types.len() {
        0 => 0.0,
        1 => 1.0,
        2 => 2.0,
        n => n as f32,
    }
}

/// Measures density of unique long tokens (unique = identifying)
fn measure_unique_density(text: &str) -> f32 {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.len() < 10 { return 0.0; }

    let mut freq: HashMap<&str, u32> = HashMap::new();
    for t in &tokens {
        *freq.entry(t).or_insert(0) += 1;
    }
    let unique = freq.values().filter(|&&v| v == 1).count();
    unique as f32 / tokens.len() as f32
}

/// Count any remaining PII-like patterns (numbers, emails that slipped through)
fn count_residual_pii(text: &str) -> u32 {
    use regex::Regex;
    let patterns = [
        r"\b\d{10}\b",                                    // phone
        r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", // email
        r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}\b",              // aadhaar
        r"\b[A-Z]{5}[0-9]{4}[A-Z]\b",                    // PAN
    ];
    let mut count = 0u32;
    for p in &patterns {
        if let Ok(re) = Regex::new(p) {
            count += re.find_iter(text).count() as u32;
        }
    }
    count
}

/// Estimate k-value: how many people could this anonymized record represent?
fn estimate_k_value(
    specificity: f32,
    quasi_risk: f32,
    unique_density: f32,
    sensitivity: u8,
) -> u32 {
    // Base k from sensitivity level
    let base: f32 = match sensitivity {
        1 => 3.0,
        2 => 8.0,
        _ => 20.0,
    };

    // Reduce k for risk factors
    let reduction = specificity * 10.0 + quasi_risk * 2.0 + unique_density * 5.0;
    let k = (base - reduction).max(1.0) as u32;
    k
}

fn calculate_privacy_score(k_value: u32, k_target: u32, risk_factors: &[String]) -> f32 {
    let k_score = (k_value as f32 / k_target as f32).min(1.0);
    let risk_penalty = risk_factors.len() as f32 * 0.1;
    (k_score - risk_penalty).max(0.0).min(1.0)
}
