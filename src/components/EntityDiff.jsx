const METHOD_COLORS = {"NER":"#4a9eff","QUASI-ID":"#f59e0b","REGEX":"#a78bfa"};
const TYPE_ICONS = {PERSON:"👤",ORG:"🏛",GPE:"📍",LOC:"📍",DATE:"📅",EMAIL:"✉",PHONE:"📞",AADHAAR:"🪪",PAN:"🪪",SALARY:"💰",AGE:"🎂",GENDER:"◐",PINCODE:"🗺",URL:"🔗",IP:"⬡"};

export default function EntityDiff({ entities }) {
  if (!entities?.length) return <div className="diff-empty">No entities detected.</div>;
  const byMethod = entities.reduce((acc,e)=>{ (acc[e.method]=acc[e.method]||[]).push(e); return acc; },{});
  return (
    <div className="entity-diff">
      <div className="diff-header">
        <span className="diff-title">⟺ ENTITY TRANSFORMATION DIFF</span>
        <div className="diff-counts">
          {Object.entries(byMethod).map(([m,items])=>(
            <span key={m} className="diff-count" style={{borderColor:METHOD_COLORS[m]}}>{m}: {items.length}</span>
          ))}
        </div>
      </div>
      <div className="diff-table-wrap">
        <table className="diff-table">
          <thead><tr><th>TYPE</th><th>ORIGINAL</th><th></th><th>FUZZED</th><th>METHOD</th></tr></thead>
          <tbody>
            {entities.map((e,i)=>(
              <tr key={i} className="diff-row">
                <td><span className="type-cell">{TYPE_ICONS[e.entity_type]||"◈"} {e.entity_type}</span></td>
                <td className="original-cell">{e.original}</td>
                <td className="arrow-cell">→</td>
                <td className="fuzzed-cell">{e.fuzzed}</td>
                <td><span className="method-pill" style={{background:`${METHOD_COLORS[e.method]}22`,borderColor:METHOD_COLORS[e.method],color:METHOD_COLORS[e.method]}}>{e.method}</span></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
