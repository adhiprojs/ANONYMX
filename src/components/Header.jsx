import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../ThemeProvider";

export default function Header({ onSettings }) {
  const [session, setSession] = useState(null);
  const { currentTheme, autoRotate } = useTheme();
  useEffect(() => {
    const fetch = async () => {
      try { setSession(await invoke("get_session_info")); } catch (_) {}
    };
    fetch();
    const t = setInterval(fetch, 4000);
    return () => clearInterval(t);
  }, []);
  return (
    <header className="header">
      <div className="header-brand">
        <div className="brand-hex">⬡</div>
        <div className="brand-text">
          <span className="brand-name">ANONYMX</span>
          <span className="brand-sub">ZERO-TRUST DATA ANONYMIZATION</span>
        </div>
        <div className="brand-classified"><span>CLASSIFIED TOOL</span></div>
      </div>
      <div className="header-status">
        {/* Theme indicator */}
        <div className="theme-indicator" title={currentTheme.personality}>
          <span className="theme-indicator-dot" />
          <span>{currentTheme.name}</span>
          {autoRotate && <span className="theme-indicator-rotate">⟳</span>}
        </div>
        {session && (
          <>
            <StatusPill label="SESSION" value={`#${session.session_id}`} />
            <StatusPill label="DOCS" value={session.docs_processed} />
            <StatusPill label="WM ISSUED" value={session.watermarks_issued} />
            {session.honey_alerts > 0 && <StatusPill label="⚠ ALERTS" value={session.honey_alerts} danger />}
          </>
        )}
        <div className="header-offline"><span className="offline-dot" />OFFLINE · NO DISK WRITES</div>
        <button className="settings-btn" onClick={onSettings}>⚙</button>
      </div>
    </header>
  );
}
function StatusPill({ label, value, danger }) {
  return (
    <div className={`status-pill ${danger ? "danger" : ""}`}>
      <span className="pill-label">{label}</span>
      <span className="pill-value">{value}</span>
    </div>
  );
}
