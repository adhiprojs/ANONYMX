import { useState } from "react";

const DEMO_TEXT = `Patient Name: Arjun Kumar
Age: 34 years old
Gender: Male
Address: 42, Gandhi Nagar, Chennai, Tamil Nadu - 600012
Phone: 9876543210
Email: arjun.kumar@hospital.in
Aadhaar: 4532 8901 2345
PAN: ABCDE1234F
Organization: Rajalakshmi Engineering College, Chennai
Salary: 85000 per month
Diagnosis: The patient, a CSE student who drives a blue Honda City, presented with rare Type-III allergy.
Referred by: Dr. Priya Sharma, Apollo Hospital, Bangalore
Date of Admission: March 15, 2024`;

export default function InputStage({ onProcess, onFileOpen, loading }) {
  const [text, setText] = useState("");
  const [mode, setMode] = useState("text");
  return (
    <div className="input-stage">
      <div className="input-header">
        <span className="stage-label">▶ INPUT DOCUMENT</span>
        <div className="input-mode-tabs">
          <button className={`mode-tab ${mode==="text"?"active":""}`} onClick={()=>setMode("text")}>TEXT</button>
          <button className={`mode-tab ${mode==="file"?"active":""}`} onClick={()=>setMode("file")}>FILE</button>
        </div>
      </div>
      {mode==="text" ? (
        <>
          <textarea className="input-textarea" value={text} onChange={e=>setText(e.target.value)} placeholder="Paste sensitive text here…" rows={11} spellCheck={false}/>
          <div className="input-actions">
            <button className="btn-ghost" onClick={()=>setText(DEMO_TEXT)}>Load Demo</button>
            <button className="btn-ghost" onClick={()=>setText("")} disabled={!text}>Clear</button>
            <span className="char-count">{text.length} chars</span>
            <button className="btn-process" onClick={()=>onProcess(text)} disabled={loading||!text.trim()}>
              {loading ? "PROCESSING…" : "RUN PIPELINE →"}
            </button>
          </div>
        </>
      ) : (
        <div className="file-zone" onClick={onFileOpen}>
          <div className="file-icon">◈</div>
          <p className="file-title">Click to open file</p>
          <p className="file-types">.txt · .csv · .json · .md · .log · .pdf</p>
          <p className="file-note">Processed in RAM · Never written to disk</p>
          <button className="btn-process file-btn" disabled={loading}>{loading?"PROCESSING…":"OPEN & PROCESS →"}</button>
        </div>
      )}
    </div>
  );
}
