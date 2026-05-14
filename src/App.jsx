import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

import Header from "./components/Header";
import InputStage from "./components/InputStage";
import PipelineVisual from "./components/PipelineVisual";
import AuditReport from "./components/AuditReport";
import EntityDiff from "./components/EntityDiff";
import OutputPanel from "./components/OutputPanel";
import WatermarkPanel from "./components/WatermarkPanel";
import SettingsDrawer from "./components/SettingsDrawer";
import HoneyAlert from "./components/HoneyAlert";

const PIPELINE_STAGES = [
  { id: "input",     label: "INPUT",       icon: "▶" },
  { id: "ner",       label: "NER + REGEX", icon: "◈" },
  { id: "fuzzing",   label: "FUZZING",     icon: "⟳" },
  { id: "kanon",     label: "K-ANONYMITY", icon: "K" },
  { id: "honey",     label: "HONEY TRAP",  icon: "⬡" },
  { id: "watermark", label: "WATERMARK",   icon: "∿" },
  { id: "ghost",     label: "GHOST CODE",  icon: "◻" },
];

export default function App() {
  const [activeStage, setActiveStage] = useState(null);
  const [result, setResult]           = useState(null);
  const [loading, setLoading]         = useState(false);
  const [error, setError]             = useState(null);
  const [showSettings, setShowSettings] = useState(false);
  const [honeyAlert, setHoneyAlert]   = useState(false);
  const [view, setView]               = useState("output"); // "output" | "diff" | "watermark"
  const [inputFile, setInputFile]     = useState(null);

  const [config, setConfig] = useState({
    sensitivity: 1,
    k_target: 5,
    quasi_fields: ["age", "dob", "gender", "zip"],
    embed_watermark: true,
    enable_honey_trap: true,
    department_tag: "DEPT-A",
  });

  const runPipeline = useCallback(async (text) => {
    setLoading(true);
    setError(null);
    setResult(null);
    setActiveStage("input");

    // Simulate stage progression for UI
    const stages = ["ner", "fuzzing", "kanon", "honey", "watermark", "ghost"];
    let stageIdx = 0;

    const stageTimer = setInterval(() => {
      if (stageIdx < stages.length) {
        setActiveStage(stages[stageIdx]);
        stageIdx++;
      } else {
        clearInterval(stageTimer);
      }
    }, 320);

    try {
      const res = await invoke("process_document", {
        request: {
          text,
          sensitivity: config.sensitivity,
          quasi_fields: config.quasi_fields,
          k_target: config.k_target,
          embed_watermark: config.embed_watermark,
          enable_honey_trap: config.enable_honey_trap,
          department_tag: config.department_tag,
        },
      });

      clearInterval(stageTimer);
      setActiveStage("done");
      setResult(res);

      if (res.honey_triggered) {
        setHoneyAlert(true);
        setTimeout(() => setHoneyAlert(false), 6000);
      }
    } catch (e) {
      clearInterval(stageTimer);
      setActiveStage(null);
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [config]);

  const handleFileOpen = useCallback(async () => {
    try {
      const path = await open({
        filters: [{ name: "Documents", extensions: ["txt","csv","json","md","log","pdf"] }],
      });
      if (!path) return;
      setInputFile(path);
      const content = await invoke("read_file", { path: String(path) });
      await runPipeline(content);
    } catch (e) {
      setError(`File Open Error: ${e}`);
    }
  }, [runPipeline]);

  const handleExport = useCallback(async () => {
    if (!result) return;
    
    let defaultPath = "anonymized_output.txt";
    let filters = [{ name: "Text files", extensions: ["txt", "md"] }];

    if (inputFile) {
        const pathStr = String(inputFile);
        const match = pathStr.match(/([^/\\]+)\.([^.]+)$/);
        if (match) {
            const filename = match[1];
            const ext = match[2];
            defaultPath = `${filename}_anonymized.${ext}`;
            filters = [{ name: `${ext.toUpperCase()} File`, extensions: [ext] }];
        } else {
            const fallbackMatch = pathStr.match(/([^/\\]+)$/);
            const filename = fallbackMatch ? fallbackMatch[1] : "output";
            defaultPath = `${filename}_anonymized.txt`;
        }
    }

    const path = await save({ defaultPath, filters });
    if (!path) return;
    await invoke("write_file", { path, content: result.watermarked_text });
  }, [result, inputFile]);

  return (
    <div className="app">
      <Header onSettings={() => setShowSettings(true)} />

      {honeyAlert && <HoneyAlert onDismiss={() => setHoneyAlert(false)} />}

      <div className="app-layout">
        {/* Left — Input + Pipeline */}
        <aside className="left-col">
          <InputStage
            onProcess={(text) => { setInputFile(null); runPipeline(text); }}
            onFileOpen={handleFileOpen}
            loading={loading}
            config={config}
          />
          <PipelineVisual stages={PIPELINE_STAGES} activeStage={activeStage} result={result} />
        </aside>

        {/* Right — Results */}
        <main className="right-col">
          {error && (
            <div className="error-bar">
              <span className="err-icon">⚠</span>
              <span>{error}</span>
            </div>
          )}

          {result && (
            <div className="result-tabs">
              {["output", "diff", "watermark"].map((v) => (
                <button
                  key={v}
                  className={`rtab ${view === v ? "active" : ""}`}
                  onClick={() => setView(v)}
                >
                  {v === "output" ? "📄 Output" : v === "diff" ? "⟺ Entity Diff" : "∿ Watermark"}
                </button>
              ))}
            </div>
          )}

          {result && view === "output" && (
            <>
              <AuditReport report={result.k_report} processingTime={result.processing_time_ms} />
              <OutputPanel
                result={result}
                onExport={handleExport}
              />
            </>
          )}

          {result && view === "diff" && (
            <EntityDiff entities={result.entities} />
          )}

          {result && view === "watermark" && (
            <WatermarkPanel
              watermarkId={result.watermark_id}
              departmentTag={config.department_tag}
            />
          )}

          {!result && !loading && !error && (
            <div className="empty-state">
              <div className="empty-hex">⬡</div>
              <p className="empty-title">AnonymX</p>
              <p className="empty-sub">Paste text or open a file to begin the anonymization pipeline</p>
              <div className="empty-features">
                {["Smart Fuzzing", "K-Anonymity Audit", "Honey Trap", "Ghost Code", "Watermarking", "Zero Footprint"]
                  .map(f => <span key={f} className="empty-feat">{f}</span>)}
              </div>
            </div>
          )}
        </main>
      </div>

      {showSettings && (
        <SettingsDrawer
          config={config}
          onChange={setConfig}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}
