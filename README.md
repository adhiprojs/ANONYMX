# AnonymX
### Zero-Trust Local Data Anonymization · Privacy by Design

> "We don't lock the data — we change it. No key exists to steal."

---

## What It Does

AnonymX transforms sensitive data into analytically-useful anonymous data using:
- **Smart Fuzzing** — generalizes specific values into categories ("22 yrs" → "20–25")
- **NER + Quasi-ID Detection** — spaCy en_core_web_lg + regex for names, Aadhaar, PAN, email, phone
- **K-Anonymity Auditor** — mathematical proof no individual can be singled out
- **Honey-Data Trap** — feeds realistic fake data to brute-force attackers
- **Ghost-Code Obfuscator** — runtime self-obfuscation of scoring logic
- **Digital Watermarking** — invisible forensic trace in output for leak detection
- **Zero-Footprint** — RAM-only, no disk writes, memory wiped after processing

---

## Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Tauri CLI
cargo install tauri-cli

# Node
npm install

# Python + spaCy
pip install spacy faker
python -m spacy download en_core_web_lg
```

## Run

```bash
# Development
cargo tauri dev

# Production build
cargo tauri build
```

## Project Structure

```
anonymx/
├── src/                        # React frontend (Vite)
│   ├── App.jsx                 # Root + pipeline orchestration
│   ├── styles.css              # Classified-document aesthetic
│   └── components/
│       ├── InputStage.jsx      # Text / File input
│       ├── PipelineVisual.jsx  # Live 7-stage pipeline display
│       ├── AuditReport.jsx     # K-Anonymity scorecard
│       ├── EntityDiff.jsx      # Before/after entity diff
│       ├── WatermarkPanel.jsx  # Watermark embed/verify
│       ├── HoneyStatus.jsx     # Honey trap status
│       └── SessionInfo.jsx     # RAM session indicator
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # IPC commands, RAM locking
│   │   ├── fuzzer.rs           # Smart generalization engine
│   │   ├── k_anonymity.rs      # K-anonymity scorer
│   │   ├── honey_trap.rs       # Honey data generator + detector
│   │   ├── ghost_code.rs       # Runtime logic obfuscation
│   │   ├── watermark.rs        # Digital watermarking (whitespace steg)
│   │   └── memory_guard.rs     # RAM-only processing + zeroize
│   ├── sidecars/
│   │   └── ner_engine.py       # spaCy NER + regex sidecar
│   └── tauri.conf.json
└── README.md
```
