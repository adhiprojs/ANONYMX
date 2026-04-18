#![allow(dead_code, unused_imports)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fuzzer;
mod k_anonymity;
mod honey_trap;
mod ghost_code;
mod watermark;
mod memory_guard;

use std::sync::Mutex;
use std::process::Command;
use serde::{Deserialize, Serialize};

pub struct SessionState {
    pub id: String,
    pub docs_processed: u32,
    pub honey_alerts: u32,
    pub watermark_registry: Vec<watermark::WatermarkEntry>,
}

impl SessionState {
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            docs_processed: 0,
            honey_alerts: 0,
            watermark_registry: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessRequest {
    pub text: String,
    pub sensitivity: u8,
    pub quasi_fields: Vec<String>,
    pub k_target: u32,
    pub embed_watermark: bool,
    pub enable_honey_trap: bool,
    pub department_tag: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NerEntity {
    pub original: String,
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuzzedEntity {
    pub original: String,
    pub fuzzed: String,
    pub entity_type: String,
    pub method: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KAnonymityReport {
    pub score: f32,
    pub k_value: u32,
    pub target_met: bool,
    pub risk_factors: Vec<String>,
    pub verdict: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessResult {
    pub original_text: String,
    pub fuzzed_text: String,
    pub watermarked_text: String,
    pub entities: Vec<FuzzedEntity>,
    pub k_report: KAnonymityReport,
    pub honey_triggered: bool,
    pub watermark_id: Option<String>,
    pub ghost_applied: bool,
    pub processing_time_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifyWatermarkResult {
    pub found: bool,
    pub watermark_id: Option<String>,
    pub department_tag: Option<String>,
    pub timestamp: Option<String>,
}

fn run_ner(text: &str) -> Vec<NerEntity> {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sidecars/ner_engine.py");

    let output = Command::new("/home/adhithya/ANONYMX/.venv/bin/python3")
        .arg(&script)
        .arg("--text")
        .arg(text)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            serde_json::from_str::<Vec<NerEntity>>(&stdout).unwrap_or_default()
        }
        Ok(out) => {
            eprintln!("NER stderr: {}", String::from_utf8_lossy(&out.stderr));
            Vec::new()
        }
        Err(e) => {
            eprintln!("NER failed to run: {}", e);
            Vec::new()
        }
    }
}

#[tauri::command]
async fn process_document(
    request: ProcessRequest,
    state: tauri::State<'_, Mutex<SessionState>>,
) -> Result<ProcessResult, String> {
    let start = std::time::Instant::now();

    let mut secure_buffer = memory_guard::SecureBuffer::new(&request.text);

    // Run spaCy NER
    let raw_entities = run_ner(&request.text);

    let (fuzzed_text, fuzzed_entities) = fuzzer::fuzz_document(
        &request.text,
        raw_entities,
        &request.quasi_fields,
        request.sensitivity,
    );
    secure_buffer.update(&fuzzed_text);

    let k_report = k_anonymity::audit(
        &fuzzed_text,
        &fuzzed_entities,
        request.k_target,
        request.sensitivity,
    );

    let honey_triggered = if request.enable_honey_trap {
        let triggered = honey_trap::check_and_respond(&fuzzed_text);
        if triggered {
            let mut s = state.lock().unwrap();
            s.honey_alerts += 1;
        }
        triggered
    } else {
        false
    };

    let (watermarked_text, watermark_id) = if request.embed_watermark {
        let session_id = state.lock().unwrap().id.clone();
        let (wm_text, wm_id) = watermark::embed(
            &fuzzed_text,
            &request.department_tag,
            &session_id,
        )?;
        state.lock().unwrap().watermark_registry.push(watermark::WatermarkEntry {
            id: wm_id.clone(),
            department_tag: request.department_tag.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
        (wm_text, Some(wm_id))
    } else {
        (fuzzed_text.clone(), None)
    };

    let mut final_text = watermarked_text.clone();
    let ghost_applied = ghost_code::obfuscate_output(&mut final_text);

    {
        let mut s = state.lock().unwrap();
        s.docs_processed += 1;
    }

    secure_buffer.zeroize();

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(ProcessResult {
        original_text: request.text.clone(),
        fuzzed_text,
        watermarked_text: final_text,
        entities: fuzzed_entities,
        k_report,
        honey_triggered,
        watermark_id,
        ghost_applied,
        processing_time_ms: elapsed,
    })
}

#[tauri::command]
async fn verify_watermark(
    text: String,
    state: tauri::State<'_, Mutex<SessionState>>,
) -> Result<VerifyWatermarkResult, String> {
    let result = watermark::verify(&text);
    if let Some(ref wm_id) = result.watermark_id {
        let registry = state.lock().unwrap();
        let entry = registry.watermark_registry.iter().find(|e| &e.id == wm_id);
        if let Some(e) = entry {
            return Ok(VerifyWatermarkResult {
                found: true,
                watermark_id: Some(wm_id.clone()),
                department_tag: Some(e.department_tag.clone()),
                timestamp: Some(e.timestamp.clone()),
            });
        }
    }
    Ok(result)
}

#[tauri::command]
async fn generate_honey_data(count: u32) -> Result<String, String> {
    honey_trap::generate_fake_dataset(count).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_session_info(state: tauri::State<'_, Mutex<SessionState>>) -> serde_json::Value {
    let s = state.lock().unwrap();
    serde_json::json!({
        "session_id": &s.id[..8],
        "docs_processed": s.docs_processed,
        "honey_alerts": s.honey_alerts,
        "watermarks_issued": s.watermark_registry.len()
    })
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("Write error: {}", e))
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(SessionState::new()))
        .invoke_handler(tauri::generate_handler![
            process_document,
            verify_watermark,
            generate_honey_data,
            get_session_info,
            read_file,
            write_file,
        ])
        .run(tauri::generate_context!())
        .expect("Context-Guard failed to start");
}
