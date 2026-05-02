export default function AuditReport({ report, processingTime }) {
  if (!report) return null;
  const verdictColor = {SAFE:"var(--safe-color)",MARGINAL:"var(--accent)",UNSAFE:"var(--unsafe-color)"}[report.verdict]||"var(--accent)";
  const scorePercent = (report.score*100).toFixed(0);
  const arcDash = (report.score*220).toFixed(0);
  return (
    <div className="audit-report">
      <div className="audit-header">
        <span className="audit-title">K-ANONYMITY AUDIT REPORT</span>
        <span className={`verdict-badge verdict-${report.verdict}`}>{report.verdict}</span>
      </div>
      <div className="audit-body">
        <div className="score-gauge">
          <svg viewBox="0 0 100 60" className="gauge-svg">
            <path d="M10,55 A45,45 0 0,1 90,55" fill="none" stroke="var(--border)" strokeWidth="6" strokeLinecap="round"/>
            <path d="M10,55 A45,45 0 0,1 90,55" fill="none" stroke={verdictColor} strokeWidth="6" strokeLinecap="round" strokeDasharray={`${arcDash} 220`} style={{transition:"stroke-dasharray 0.8s ease"}}/>
            <text x="50" y="52" textAnchor="middle" className="gauge-pct" fill={verdictColor}>{scorePercent}%</text>
            <text x="50" y="62" textAnchor="middle" className="gauge-label" fill="var(--text-muted)">PRIVACY SCORE</text>
          </svg>
        </div>
        <div className="k-stats">
          <div className="k-stat"><span className="k-stat-val" style={{color:verdictColor}}>{report.k_value}</span><span className="k-stat-label">Calculated k</span></div>
          <div className="k-divider"/>
          <div className="k-stat"><span className="k-stat-val">{report.target_met?"✓":"✗"}</span><span className="k-stat-label">Target Met</span></div>
        </div>
        <div className="risk-factors">
          <div className="risk-title">RISK FACTORS</div>
          {report.risk_factors.length===0
            ? <div className="risk-none">✓ No risk factors detected</div>
            : report.risk_factors.map((r,i)=><div key={i} className="risk-item"><span className="risk-dot">▸</span><span>{r}</span></div>)
          }
        </div>
      </div>
      <div className="audit-footer">Processed in {processingTime}ms · RAM-only · No disk trace</div>
    </div>
  );
}
