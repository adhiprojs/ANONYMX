export default function HoneyAlert({ onDismiss }) {
  return (
    <div className="honey-alert">
      <div className="honey-alert-inner">
        <span className="honey-icon">⬡</span>
        <div className="honey-text">
          <strong>HONEY TRAP TRIGGERED</strong>
          <span>Suspicious reverse-engineering attempt detected. Fake ghost data served. Attempt flagged.</span>
        </div>
        <button className="honey-dismiss" onClick={onDismiss}>✕</button>
      </div>
    </div>
  );
}
