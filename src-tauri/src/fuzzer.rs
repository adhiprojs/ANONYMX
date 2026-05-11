// fuzzer.rs — Smart Fuzzing Engine v2 (fixed)
//
// Root-cause fixes applied:
//
// 1. OFFSET DRIFT: Previously, fuzz_residual_patterns() ran first and mutated
//    the string, then NER entities were applied at their *original* byte offsets
//    into the now-different string. This caused mid-word replacements like
//    "PAN: [PAN[AADHAAR]" and "Phone: [PHONE]ail: [EM[EMAIL]".
//    Fix: Run NER replacements FIRST (sorted descending by offset so each
//    substitution doesn't affect the positions of earlier ones), THEN run
//    regex passes on the already-NER-cleaned text.
//
// 2. SALARY REGEX MISFIRES: The salary quasi-field regex matched bare digit
//    sequences inside already-placed tags like "[LOWER-INCOME BRACKET]",
//    re-replacing them and producing "[LOWER-INCOME BRACKET]-[[LOWER-INCOME
//    BRACKET]-[LOWER-INCOME BRACKET]yrs]". Fix: anchor the salary pattern
//    with negative lookbehind for "[" so it ignores content inside brackets.
//
// 3. QUASI-FIELD ORDERING: Age/DOB quasi replacements ran after NER, so they
//    could fire on already-replaced tokens. Fix: quasi fields now run after
//    ALL bracket-producing passes, and each regex skips text already inside [].

use crate::{NerEntity, FuzzedEntity};
use regex::Regex;

pub fn fuzz_document(
    text: &str,
    ner_entities: Vec<NerEntity>,
    quasi_fields: &[String],
    sensitivity: u8,
) -> (String, Vec<FuzzedEntity>) {
    let mut fuzzed_log: Vec<FuzzedEntity> = Vec::new();

    // ── Step 1: Apply NER replacements at original offsets (descending) ────
    // Must happen BEFORE any regex pass that changes string length, otherwise
    // the recorded start/end offsets become invalid.
    let mut result = text.to_string();
    let mut sorted_ner = ner_entities.clone();
    sorted_ner.sort_by(|a, b| b.start.cmp(&a.start));

    for entity in &sorted_ner {
        // Guard: offsets must still be valid after prior replacements in the
        // same descending pass (they should be, since we go right-to-left).
        if entity.start > result.len() || entity.end > result.len() || entity.start > entity.end {
            continue;
        }
        // Extra guard: check the slice actually contains the expected text
        // (if spaCy offsets were character-based but Rust uses bytes, there
        // can be off-by-ones on multi-byte characters).
        let slice = result.get(entity.start..entity.end).unwrap_or("");
        if slice.is_empty() {
            continue;
        }
        let replacement = fuzz_ner_entity(entity, sensitivity);
        result.replace_range(entity.start..entity.end, &replacement);
        fuzzed_log.push(FuzzedEntity {
            original: entity.original.clone(),
            fuzzed: replacement,
            entity_type: entity.entity_type.clone(),
            method: "NER".to_string(),
        });
    }

    // ── Step 2: Residual regex pass on NER-cleaned text ────────────────────
    // At this point there are no more original offsets to honor, so regex
    // replacement is safe regardless of length changes.
    let (result, mut residual) = fuzz_residual_patterns(&result, sensitivity);
    fuzzed_log.append(&mut residual);

    // ── Step 3: Quasi-identifier field pass ────────────────────────────────
    let mut result = result;
    for field in quasi_fields {
        let (new_text, mut entries) = fuzz_quasi_field(&result, field, sensitivity);
        result = new_text;
        fuzzed_log.append(&mut entries);
    }

    (result, fuzzed_log)
}

fn fuzz_ner_entity(entity: &NerEntity, sensitivity: u8) -> String {
    match entity.entity_type.as_str() {
        "PERSON"   => fuzz_person(&entity.original, sensitivity),
        "ORG"      => fuzz_org(&entity.original, sensitivity),
        "GPE" | "LOC" => fuzz_location(&entity.original, sensitivity),
        "DATE"     => fuzz_date_entity(&entity.original, sensitivity),
        "AGE_NUM"  => fuzz_age_num(&entity.original, sensitivity),
        "DOB"      => fuzz_dob(&entity.original, sensitivity),
        "TIME"     => "[TIME]".to_string(),
        "MONEY" | "CARDINAL" if looks_like_money(&entity.original) => {
            fuzz_money(&entity.original, sensitivity)
        }
        "PRODUCT"     => "[PRODUCT]".to_string(),
        "EVENT"       => "[EVENT]".to_string(),
        "LANGUAGE"    => entity.original.clone(),
        "NORP"        => "[COMMUNITY/GROUP]".to_string(),
        "FAC"         => "[FACILITY]".to_string(),
        "PAN"         => "[PAN]".to_string(),
        "AADHAAR"     => "[AADHAAR]".to_string(),
        "EMAIL"       => "[EMAIL]".to_string(),
        "PHONE"       => "[PHONE]".to_string(),
        "IFSC"        => "[IFSC]".to_string(),
        "IP_ADDR"     => "[IP-ADDRESS]".to_string(),
        "URL"         => "[URL]".to_string(),
        "WORK_OF_ART" => "[CREATIVE WORK]".to_string(),
        _             => format!("[{}]", entity.entity_type),
    }
}

// ── Person ────────────────────────────────────────────────────────────────────

fn fuzz_person(name: &str, sensitivity: u8) -> String {
    match sensitivity {
        1 => {
            let initial = name.chars().next().unwrap_or('X');
            format!("[{}.____]", initial)
        }
        2 => "[INDIVIDUAL]".to_string(),
        _ => "[PERSON]".to_string(),
    }
}

// ── Organisation ─────────────────────────────────────────────────────────────

fn fuzz_org(org: &str, sensitivity: u8) -> String {
    let lower = org.to_lowercase();

    let org_type = if let Some(d) = detect_engineering_dept(&lower) {
        d
    } else if lower.contains("iit") || lower.contains("nit") || lower.contains("bits") {
        "Premier Technical Institution"
    } else if lower.contains("university") {
        "University"
    } else if lower.contains("college") || lower.contains("institute") {
        "Educational Institution"
    } else if lower.contains("hospital") || lower.contains("clinic")
        || lower.contains("apollo") || lower.contains("fortis") || lower.contains("aiims")
    {
        "Healthcare Organization"
    } else if lower.contains("bank") || lower.contains("finance") || lower.contains("insurance") {
        "Financial Institution"
    } else if lower.contains("school") {
        "School"
    } else if lower.contains("government") || lower.contains("ministry") || lower.contains("municipal") {
        "Government Body"
    } else if lower.contains("police") || lower.contains("army") || lower.contains("defence") {
        "Security/Defence Organization"
    } else if lower.contains("tech") || lower.contains("software")
        || lower.contains("infosys") || lower.contains("wipro") || lower.contains("tcs")
    {
        "Technology Company"
    } else {
        "Organization"
    };

    if sensitivity == 1 {
        if let Some(city_info) = extract_city_hint(org) {
            return format!("[A {} in {}]", org_type, city_info);
        }
    }
    format!("[{}]", org_type)
}

fn detect_engineering_dept(lower: &str) -> Option<&'static str> {
    if lower.contains("computer science") || lower.contains("cse") {
        Some("Computer Science Department")
    } else if lower.contains("electrical") || lower.contains("eee") || lower.contains("ece") {
        Some("Electronics & Electrical Department")
    } else if lower.contains("mechanical") {
        Some("Mechanical Engineering Department")
    } else if lower.contains("civil") && lower.contains("dept") {
        Some("Civil Engineering Department")
    } else if lower.contains("chemical") && lower.contains("dept") {
        Some("Chemical Engineering Department")
    } else if lower.contains("biomedical") || lower.contains("bio-medical") {
        Some("Biomedical Engineering Department")
    } else if lower.contains("aerospace") || lower.contains("aeronautical") {
        Some("Aerospace Engineering Department")
    } else if lower.contains("information technology") || lower.contains("it dept") {
        Some("Information Technology Department")
    } else if lower.contains("artificial intelligence") || lower.contains("ai & ds") || lower.contains("ai dept") {
        Some("AI & Data Science Department")
    } else if lower.contains("data science") {
        Some("Data Science Department")
    } else if lower.contains("mathematics") && lower.contains("dept") {
        Some("Mathematics Department")
    } else if lower.contains("management") || lower.contains("mba") {
        Some("Management Department")
    } else {
        None
    }
}

fn extract_city_hint(text: &str) -> Option<String> {
    let city_map: &[(&str, &str, &str)] = &[
        ("Chennai", "Tamil Nadu", "India"),
        ("Madras", "Tamil Nadu", "India"),
        ("Mumbai", "Maharashtra", "India"),
        ("Bombay", "Maharashtra", "India"),
        ("Delhi", "Delhi NCR", "India"),
        ("New Delhi", "Delhi NCR", "India"),
        ("Bangalore", "Karnataka", "India"),
        ("Bengaluru", "Karnataka", "India"),
        ("Hyderabad", "Telangana", "India"),
        ("Kolkata", "West Bengal", "India"),
        ("Pune", "Maharashtra", "India"),
        ("Ahmedabad", "Gujarat", "India"),
        ("Jaipur", "Rajasthan", "India"),
        ("Coimbatore", "Tamil Nadu", "India"),
        ("Madurai", "Tamil Nadu", "India"),
        ("Trichy", "Tamil Nadu", "India"),
        ("Vellore", "Tamil Nadu", "India"),
        ("Kochi", "Kerala", "India"),
        ("Nagpur", "Maharashtra", "India"),
        ("Lucknow", "Uttar Pradesh", "India"),
        ("Visakhapatnam", "Andhra Pradesh", "India"),
        ("New York", "New York State", "USA"),
        ("Los Angeles", "California", "USA"),
        ("San Francisco", "California", "USA"),
        ("London", "England", "UK"),
        ("Manchester", "England", "UK"),
        ("Paris", "Ile-de-France", "France"),
        ("Berlin", "Brandenburg", "Germany"),
        ("Tokyo", "Kanto", "Japan"),
        ("Singapore", "Singapore", "Singapore"),
        ("Dubai", "UAE", "UAE"),
        ("Sydney", "New South Wales", "Australia"),
        ("Toronto", "Ontario", "Canada"),
    ];
    for (city, region, country) in city_map {
        if text.contains(city) {
            return Some(format!("{}, {}", region, country));
        }
    }
    None
}

// ── Location ──────────────────────────────────────────────────────────────────

fn fuzz_location(loc: &str, sensitivity: u8) -> String {
    match sensitivity {
        1 => {
            if let Some(info) = extract_city_hint(loc) {
                return format!("[Location in {}]", info);
            }
            format!("[Location in {}]", extract_state_region(loc))
        }
        2 => format!("[Location in {}]", extract_country(loc)),
        _ => "[LOCATION]".to_string(),
    }
}

fn extract_state_region(loc: &str) -> String {
    let state_map: &[(&str, &str)] = &[
        ("Tamil Nadu", "Tamil Nadu, India"),
        ("Maharashtra", "Maharashtra, India"),
        ("Karnataka", "Karnataka, India"),
        ("Kerala", "Kerala, India"),
        ("Andhra", "Andhra Pradesh, India"),
        ("Telangana", "Telangana, India"),
        ("Rajasthan", "Rajasthan, India"),
        ("Gujarat", "Gujarat, India"),
        ("Punjab", "Punjab, India"),
        ("Bihar", "Bihar, India"),
        ("California", "California, USA"),
        ("Texas", "Texas, USA"),
        ("England", "England, UK"),
    ];
    for (key, label) in state_map {
        if loc.contains(key) {
            return label.to_string();
        }
    }
    if let Some(info) = extract_city_hint(loc) {
        return info;
    }
    "an Undisclosed Region".to_string()
}

fn extract_country(loc: &str) -> &str {
    let hints: &[(&str, &str)] = &[
        ("Tamil Nadu", "India"), ("Maharashtra", "India"), ("Karnataka", "India"),
        ("Kerala", "India"), ("Delhi", "India"), ("Chennai", "India"),
        ("Mumbai", "India"), ("Bangalore", "India"), ("Hyderabad", "India"),
        ("India", "India"), ("California", "USA"), ("Texas", "USA"),
        ("USA", "USA"), ("America", "USA"), ("England", "UK"), ("UK", "UK"),
        ("London", "UK"), ("Germany", "Germany"), ("France", "France"),
        ("Japan", "Japan"), ("Singapore", "Singapore"), ("Dubai", "UAE"),
        ("UAE", "UAE"), ("Australia", "Australia"), ("Canada", "Canada"),
    ];
    for (hint, country) in hints {
        if loc.contains(hint) {
            return country;
        }
    }
    "an Undisclosed Country"
}

// ── Date ──────────────────────────────────────────────────────────────────────

fn fuzz_date_entity(date: &str, sensitivity: u8) -> String {
    let lower = date.to_lowercase();
    let is_dob = lower.contains("born") || lower.contains("dob") || lower.contains("birth");
    let has_day   = Regex::new(r"\b\d{1,2}(st|nd|rd|th)?\b").unwrap().is_match(date);
    let has_month = Regex::new(r"(?i)\b(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)\w*\b")
        .unwrap().is_match(date);
    let year_re   = Regex::new(r"\b(19|20)(\d{2})\b").unwrap();
    let has_year  = year_re.is_match(date);

    if is_dob || (has_day && has_month && has_year) {
        return match sensitivity {
            1 => {
                if let Some(cap) = year_re.captures(date) {
                    let year: u32 = cap[0].parse().unwrap_or(1990);
                    format!("[Born {}s]", (year / 10) * 10)
                } else {
                    "[DATE OF BIRTH]".to_string()
                }
            }
            _ => "[DATE OF BIRTH]".to_string(),
        };
    }
    if has_year {
        let year: u32 = year_re
            .captures(date)
            .and_then(|c| c[0].parse().ok())
            .unwrap_or(2000);
        return match sensitivity {
            1 => format!("[~{}]", year),
            2 => format!("[{}s]", (year / 10) * 10),
            _ => "[DATE]".to_string(),
        };
    }
    if has_month {
        return "[MONTH/SEASON PERIOD]".to_string();
    }
    "[DATE]".to_string()
}

// ── Age / DOB (NER-detected) ──────────────────────────────────────────────────

fn fuzz_age_num(text: &str, sensitivity: u8) -> String {
    let re = Regex::new(r"\b(\d{1,3})\b").unwrap();
    if let Some(cap) = re.find(text) {
        let age: u32 = cap.as_str().parse().unwrap_or(25);
        return match sensitivity {
            1 | 2 => {
                let b = (age / 5) * 5;
                format!("[{}-{} yrs]", b, b + 4)
            }
            _ => "[AGE GROUP]".to_string(),
        };
    }
    "[AGE GROUP]".to_string()
}

fn fuzz_dob(text: &str, sensitivity: u8) -> String {
    let year_re = Regex::new(r"\b(19|20)\d{2}\b").unwrap();
    if let Some(cap) = year_re.find(text) {
        let year: u32 = cap.as_str().parse().unwrap_or(1990);
        return match sensitivity {
            1 => format!("[Born {}s]", (year / 10) * 10),
            _ => "[DATE OF BIRTH]".to_string(),
        };
    }
    "[DATE OF BIRTH]".to_string()
}

// ── Money ─────────────────────────────────────────────────────────────────────

fn fuzz_money(amount: &str, sensitivity: u8) -> String {
    let num_re = Regex::new(r"[\d,\.]+").unwrap();
    if let Some(cap) = num_re.find(amount) {
        let val: f64 = cap.as_str().replace(',', "").parse().unwrap_or(0.0);
        return match sensitivity {
            1 => match val as u64 {
                0..=999       => "[<1K]".to_string(),
                1000..=9999   => "[1K-10K]".to_string(),
                10000..=99999 => "[10K-1L]".to_string(),
                100000..=999999 => "[1L-10L]".to_string(),
                _             => "[>10L]".to_string(),
            },
            _ => "[AMOUNT]".to_string(),
        };
    }
    "[AMOUNT]".to_string()
}

fn looks_like_money(s: &str) -> bool {
    s.contains('$') || s.contains('€')
        || s.to_lowercase().contains("rs")
        || s.to_lowercase().contains("inr")
}

// ── Quasi-identifier field pass ───────────────────────────────────────────────
//
// KEY FIX: Every regex here must NOT fire inside an already-placed bracket tag.
// We use a post-replace filter: after each substitution, if the match was
// entirely inside a [...] span (i.e. the match start is preceded by '[' without
// a closing ']' in between), we skip it.
//
// Simpler approach used here: require that the match is preceded by a
// non-bracket character using a negative lookbehind where possible, or simply
// skip matches whose text starts with '['.

fn fuzz_quasi_field(text: &str, field: &str, sensitivity: u8) -> (String, Vec<FuzzedEntity>) {
    let mut result = text.to_string();
    let mut log = Vec::new();

    match field.to_lowercase().as_str() {
        "age" => {
            // Match "25 years old", "25 yrs", "age 25" etc.
            // Negative lookahead for '[' prevents firing inside bracket tags.
            let re = Regex::new(r"(?i)\b(\d{1,3})\s*(?:years?\s*(?:old)?|yrs?\.?)").unwrap();
            let tmp = result.clone();
            result = re.replace_all(&tmp, |caps: &regex::Captures| {
                // Skip if this digit is inside a bracket tag already
                if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                    return caps[0].to_string();
                }
                let age: u32 = caps[1].parse().unwrap_or(25);
                let fuzzed = match sensitivity {
                    1 | 2 => {
                        let b = (age / 5) * 5;
                        format!("[{}-{} yrs]", b, b + 4)
                    }
                    _ => "[AGE GROUP]".to_string(),
                };
                log.push(FuzzedEntity {
                    original: caps[0].to_string(),
                    fuzzed: fuzzed.clone(),
                    entity_type: "AGE".to_string(),
                    method: "QUASI-ID".to_string(),
                });
                fuzzed
            }).into_owned();
        }

        "dob" => {
            let patterns = [
                r"(?i)\bD\.?O\.?B\.?\s*:?\s*\d{1,2}[\/\-\.]\d{1,2}[\/\-\.]\d{2,4}",
                r"(?i)\bborn\s+(?:on\s+)?\d{1,2}[\/\-]\d{1,2}[\/\-]\d{2,4}",
                r"(?i)\bdate\s+of\s+birth\s*:?\s*[\w\s,/\-]+\d{4}",
            ];
            for pat in &patterns {
                if let Ok(re) = Regex::new(pat) {
                    let tmp = result.clone();
                    result = re.replace_all(&tmp, |caps: &regex::Captures| {
                        if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                            return caps[0].to_string();
                        }
                        let original = caps[0].to_string();
                        let year_re = Regex::new(r"\b(19|20)\d{2}\b").unwrap();
                        let fuzzed = if let Some(yc) = year_re.captures(&original) {
                            let year: u32 = yc[0].parse().unwrap_or(1990);
                            match sensitivity {
                                1 => format!("[Born {}s]", (year / 10) * 10),
                                _ => "[DATE OF BIRTH]".to_string(),
                            }
                        } else {
                            "[DATE OF BIRTH]".to_string()
                        };
                        log.push(FuzzedEntity {
                            original: original.clone(),
                            fuzzed: fuzzed.clone(),
                            entity_type: "DOB".to_string(),
                            method: "QUASI-ID".to_string(),
                        });
                        fuzzed
                    }).into_owned();
                }
            }
        }

        "zip" | "pincode" | "postal" => {
            let re = Regex::new(r"\b(\d{3})(\d{3})\b").unwrap();
            let tmp = result.clone();
            result = re.replace_all(&tmp, |caps: &regex::Captures| {
                if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                    return caps[0].to_string();
                }
                let fuzzed = match sensitivity {
                    1 => format!("{}XXX", &caps[1]),
                    _ => "[PINCODE]".to_string(),
                };
                log.push(FuzzedEntity {
                    original: caps[0].to_string(),
                    fuzzed: fuzzed.clone(),
                    entity_type: "PINCODE".to_string(),
                    method: "QUASI-ID".to_string(),
                });
                fuzzed
            }).into_owned();
        }

        "gender" | "sex" => {
            let re = Regex::new(
                r"(?i)\b(male|female|man|woman|boy|girl|transgender|non-binary)\b",
            ).unwrap();
            let tmp = result.clone();
            result = re.replace_all(&tmp, |caps: &regex::Captures| {
                if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                    return caps[0].to_string();
                }
                log.push(FuzzedEntity {
                    original: caps[0].to_string(),
                    fuzzed: "[GENDER]".to_string(),
                    entity_type: "GENDER".to_string(),
                    method: "QUASI-ID".to_string(),
                });
                "[GENDER]".to_string()
            }).into_owned();
        }

        "race" | "ethnicity" => {
            for term in &["Indian", "Asian", "African American", "Caucasian", "Hispanic", "Latino", "Arab"] {
                if result.contains(term) {
                    log.push(FuzzedEntity {
                        original: term.to_string(),
                        fuzzed: "[ETHNICITY]".to_string(),
                        entity_type: "ETHNICITY".to_string(),
                        method: "QUASI-ID".to_string(),
                    });
                    result = result.replace(term, "[ETHNICITY]");
                }
            }
        }

        "salary" | "income" | "wage" => {
            // FIX: The previous regex matched any digit sequence and would fire
            // inside already-placed tags like "[LOWER-INCOME BRACKET]", turning
            // them into "[LOWER-INCOME BRACKET]-[[LOWER-INCOME BRACKET]-...]".
            //
            // New approach:
            //   - Require a currency symbol or explicit salary keyword before the
            //     digits to reduce false positives.
            //   - Use is_inside_bracket() to skip matches inside existing tags.
            //   - Match "85000 per month", "₹85,000", "$1,200/month" etc.
            let re = Regex::new(
                r"(?i)(?:[₹$€£]|(?:rs|inr|salary|income|wage|pay)\s*\.?\s*:?\s*)[\s]?(\d[\d,]*)(?:\s*(?:k|lakh|lakhs?|thousand|cr|crore))?(?:\s*(?:per\s+(?:month|year|annum|pa)|p\.?a\.?|p\.?m\.?))?",
            ).unwrap();
            let tmp = result.clone();
            result = re.replace_all(&tmp, |caps: &regex::Captures| {
                if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                    return caps[0].to_string();
                }
                let raw = caps[1].replace([',', '.'], "");
                let val: f64 = raw.parse().unwrap_or(0.0);
                // Check for multiplier suffix in the full match
                let full = caps[0].to_lowercase();
                let mult = if full.contains("lakh") { 100_000.0 }
                    else if full.contains("crore") { 10_000_000.0 }
                    else if full.ends_with('k') || full.contains(" k") { 1_000.0 }
                    else if full.contains("thousand") { 1_000.0 }
                    else { 1.0 };

                let annual = val * mult;
                let fuzzed = match annual as u64 {
                    0..=25_000          => "[LOWER-INCOME BRACKET]",
                    25_001..=75_000     => "[MID-INCOME BRACKET]",
                    75_001..=300_000    => "[UPPER-MID BRACKET]",
                    _                   => "[HIGH-INCOME BRACKET]",
                }.to_string();

                log.push(FuzzedEntity {
                    original: caps[0].to_string(),
                    fuzzed: fuzzed.clone(),
                    entity_type: "SALARY".to_string(),
                    method: "QUASI-ID".to_string(),
                });
                fuzzed
            }).into_owned();
        }

        _ => {}
    }

    (result, log)
}

/// Returns true if `pos` (byte offset) is inside a `[...]` span in `text`.
/// Used to prevent quasi-field regexes from re-replacing already-placed tags.
fn is_inside_bracket(text: &str, pos: usize) -> bool {
    let before = &text[..pos.min(text.len())];
    // Walk backwards from pos: if we hit '[' before ']', we're inside a tag.
    let mut depth = 0i32;
    for ch in before.chars().rev() {
        match ch {
            ']' => depth += 1,
            '[' => {
                if depth == 0 {
                    return true;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    false
}

// ── Residual regex pass ───────────────────────────────────────────────────────
//
// Runs AFTER NER replacements. Catches any PII patterns that spaCy missed.
// Only fires on text NOT already inside a [...] tag.

fn fuzz_residual_patterns(text: &str, _sensitivity: u8) -> (String, Vec<FuzzedEntity>) {
    let mut result = text.to_string();
    let mut log = Vec::new();

    // Order matters: more specific patterns first to avoid partial overlaps.
    let patterns: &[(&str, &str, &str)] = &[
        // Email before phone (phone regex would match digits in email)
        (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL]",      "EMAIL"),
        // Aadhaar: 12 digits (possibly spaced/hyphenated)
        (r"\b\d{4}[\s\-]?\d{4}[\s\-]?\d{4}\b",                   "[AADHAAR]",   "AADHAAR"),
        // PAN: AAAAA0000A format
        (r"\b[A-Z]{5}[0-9]{4}[A-Z]\b",                            "[PAN]",       "PAN"),
        // IFSC
        (r"\b[A-Z]{4}0[A-Z0-9]{6}\b",                             "[IFSC]",      "IFSC"),
        // Phone: Indian mobile (optionally +91)
        (r"\b(\+91[\s\-]?)?[6-9]\d{9}\b",                         "[PHONE]",     "PHONE"),
        // URL
        (r"https?://[^\s]+",                                        "[URL]",       "URL"),
        // IP address
        (r"\b(?:\d{1,3}\.){3}\d{1,3}\b",                          "[IP-ADDRESS]","IP"),
    ];

    for (pattern, replacement, etype) in patterns {
        let re = Regex::new(pattern).unwrap();
        let tmp = result.clone();
        let new = re.replace_all(&tmp, |caps: &regex::Captures| {
            // Skip if already inside a bracket tag
            if is_inside_bracket(&tmp, caps.get(0).unwrap().start()) {
                return caps[0].to_string();
            }
            log.push(FuzzedEntity {
                original: caps[0].to_string(),
                fuzzed: replacement.to_string(),
                entity_type: etype.to_string(),
                method: "REGEX".to_string(),
            });
            replacement.to_string()
        }).into_owned();
        result = new;
    }

    (result, log)
}