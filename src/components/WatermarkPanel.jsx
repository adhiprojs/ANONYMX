import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function WatermarkPanel({ watermarkId, departmentTag }) {
  const [verifyText, setVerifyText] = useState("");
  const [verifyResult, setVerifyResult] = useState(null);
  const [checking, setChecking] = useState(false);
  const handleVerify = async () => {
    setChecking(true);
    try { setVerifyResult(await invoke("verify_watermark", { text: verifyText })); }
    catch(e) { setVerifyResult({ found: false, error: String(e) }); }
    finally { setChecking(false); }
  };
  return (
    <div className="watermark-panel">
      <div className="wm-header"><span className="wm-title">∿ FORENSIC WATERMARK SYSTEM</span></div>
      {watermarkId && (
        <div className="wm-current">
          <div className="wm-current-title">CURRENT DOCUMENT WATERMARK</div>
          <div className="wm-fields">
            <WmField label="Watermark ID" value={watermarkId}/>
            <WmField label="Department Tag" value={departmentTag}/>
            <WmField label="Encoding" value="Whitespace Steganography"/>
          </div>
          <div className="wm-info">If this document is found where it shouldn't be, the watermark traces the source department.</div>
        </div>
      )}
      <div className="wm-verify">
        <div className="wm-verify-title">VERIFY A DOCUMENT</div>
        <p className="wm-verify-desc">Paste a suspected leaked document to extract its watermark and trace the source.</p>
        <textarea className="wm-verify-input" value={verifyText} onChange={e=>setVerifyText(e.target.value)} placeholder="Paste document content…" rows={5} spellCheck={false}/>
        <button className="btn-process" onClick={handleVerify} disabled={checking||!verifyText.trim()} style={{width:"100%",marginTop:"0.75rem"}}>
          {checking?"SCANNING…":"SCAN FOR WATERMARK →"}
        </button>
        {verifyResult && (
          <div className={`wm-result ${verifyResult.found?"found":"not-found"}`}>
            {verifyResult.found
              ? <><div className="wm-found-title">⚠ WATERMARK DETECTED</div><WmField label="Watermark ID" value={verifyResult.watermark_id||"—"}/><WmField label="Source Dept" value={verifyResult.department_tag||"—"}/></>
              : <div className="wm-not-found">✓ No watermark detected</div>
            }
          </div>
        )}
      </div>
    </div>
  );
}
function WmField({ label, value }) {
  return <div className="wm-field"><span className="wm-field-label">{label}</span><span className="wm-field-value">{value}</span></div>;
}
