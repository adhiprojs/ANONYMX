export default function OutputPanel({ result, onExport }) {
  if (!result) return null;
  return (
    <div className="output-panel">
      <div className="output-header">
        <span className="output-title">◈ ANONYMIZED OUTPUT</span>
        <div className="output-badges">
          {result.watermark_id && <span className="badge wm-badge">∿ WM: {result.watermark_id}</span>}
          {result.ghost_applied && <span className="badge ghost-badge">◻ Ghost Applied</span>}
          {result.honey_triggered && <span className="badge honey-badge">⬡ Honey Active</span>}
        </div>
        <button className="btn-export" onClick={onExport}>↓ EXPORT</button>
        <button className="btn-copy" onClick={() => navigator.clipboard.writeText(result.watermarked_text)}>📋 COPY</button>
      </div>
      <textarea className="output-textarea" value={result.watermarked_text} readOnly rows={12} spellCheck={false}/>
      <div className="output-note">Output contains invisible watermark + ghost markers. Use Export to save.</div>
    </div>
  );
}
