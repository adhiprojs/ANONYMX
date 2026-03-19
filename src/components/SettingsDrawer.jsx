const QUASI_OPTIONS = [{id:"age",label:"Age / DoB"},{id:"zip",label:"ZIP / Pincode"},{id:"gender",label:"Gender"},{id:"race",label:"Race / Ethnicity"},{id:"salary",label:"Salary / Income"},{id:"dob",label:"Date of Birth"}];

export default function SettingsDrawer({ config, onChange, onClose }) {
  const update = (key,val) => onChange(c=>({...c,[key]:val}));
  const toggleQuasi = (id) => { const c=config.quasi_fields; update("quasi_fields",c.includes(id)?c.filter(x=>x!==id):[...c,id]); };
  return (
    <div className="drawer-overlay" onClick={onClose}>
      <div className="drawer" onClick={e=>e.stopPropagation()}>
        <div className="drawer-header"><span>⚙ PIPELINE SETTINGS</span><button className="drawer-close" onClick={onClose}>✕</button></div>
        <div className="drawer-body">
          <Section title="SENSITIVITY LEVEL">
            <div className="sens-row">
              {[1,2,3].map(l=><button key={l} className={`sens-btn ${config.sensitivity===l?"active":""}`} onClick={()=>update("sensitivity",l)}><span className="sens-num">{l}</span><span className="sens-name">{["LOW","MEDIUM","HIGH"][l-1]}</span></button>)}
            </div>
            <p className="setting-note">{config.sensitivity===1&&"Partial info kept. Best for internal use."}{config.sensitivity===2&&"Balanced — recommended for cross-team sharing."}{config.sensitivity===3&&"Maximum anonymization."}</p>
          </Section>
          <Section title="K-ANONYMITY TARGET">
            <div className="k-input-row"><input type="range" min={2} max={20} value={config.k_target} onChange={e=>update("k_target",Number(e.target.value))} className="k-slider"/><span className="k-value-display">k = {config.k_target}</span></div>
            <p className="setting-note">Each record must be indistinguishable from {config.k_target} others.</p>
          </Section>
          <Section title="QUASI-IDENTIFIER FIELDS">
            <div className="quasi-grid">{QUASI_OPTIONS.map(q=><button key={q.id} className={`quasi-chip ${config.quasi_fields.includes(q.id)?"active":""}`} onClick={()=>toggleQuasi(q.id)}>{q.label}</button>)}</div>
          </Section>
          <Section title="SECURITY FEATURES">
            <Toggle label="Digital Watermarking" desc="Embed forensic watermark for leak tracing" value={config.embed_watermark} onChange={v=>update("embed_watermark",v)}/>
            <Toggle label="Honey Trap" desc="Detect brute-force and serve fake data" value={config.enable_honey_trap} onChange={v=>update("enable_honey_trap",v)}/>
          </Section>
          <Section title="DEPARTMENT TAG">
            <input className="dept-input" value={config.department_tag} onChange={e=>update("department_tag",e.target.value)} placeholder="e.g. DEPT-A, USER-42" maxLength={32}/>
            <p className="setting-note">Embedded in watermark for forensic tracing.</p>
          </Section>
        </div>
      </div>
    </div>
  );
}
function Section({ title, children }) {
  return <div className="drawer-section"><div className="drawer-section-title">{title}</div>{children}</div>;
}
function Toggle({ label, desc, value, onChange }) {
  return (
    <div className="toggle-row">
      <div><div className="toggle-label">{label}</div><div className="toggle-desc">{desc}</div></div>
      <button className={`toggle-btn ${value?"on":"off"}`} onClick={()=>onChange(!value)}>{value?"ON":"OFF"}</button>
    </div>
  );
}
