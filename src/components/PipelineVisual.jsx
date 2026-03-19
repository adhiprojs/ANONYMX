export default function PipelineVisual({ stages, activeStage, result }) {
  const getStatus = (id) => {
    if (!activeStage) return "idle";
    if (activeStage === "done") return "done";
    const ids = stages.map(s=>s.id);
    const ai = ids.indexOf(activeStage), ti = ids.indexOf(id);
    if (ti < ai) return "done";
    if (ti === ai) return "active";
    return "idle";
  };
  return (
    <div className="pipeline-visual">
      <div className="pipeline-label">PROCESSING PIPELINE</div>
      <div className="pipeline-stages">
        {stages.map((stage,i) => {
          const status = getStatus(stage.id);
          return (
            <div key={stage.id} className="pipeline-step-wrap">
              <div className={`pipeline-step status-${status}`}>
                <div className="step-icon">{stage.icon}</div>
                <div className="step-label">{stage.label}</div>
                {status==="active" && <div className="step-pulse"/>}
                {status==="done" && <div className="step-check">✓</div>}
              </div>
              {i < stages.length-1 && <div className={`pipeline-connector ${status==="done"?"lit":""}`}/>}
            </div>
          );
        })}
      </div>
      {result && (
        <div className="pipeline-summary">
          <SummaryItem label="Entities Found" value={result.entities?.length??0}/>
          <SummaryItem label="K-Value" value={result.k_report?.k_value??"—"}/>
          <SummaryItem label="Privacy Score" value={result.k_report?`${(result.k_report.score*100).toFixed(0)}%`:"—"}/>
          <SummaryItem label="Time" value={`${result.processing_time_ms}ms`}/>
        </div>
      )}
    </div>
  );
}
function SummaryItem({ label, value }) {
  return (
    <div className="summary-item">
      <span className="summary-label">{label}</span>
      <span className="summary-value">{value}</span>
    </div>
  );
}
