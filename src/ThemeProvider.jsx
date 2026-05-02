import { createContext, useContext, useState, useEffect, useCallback, useRef } from "react";
import { THEMES, ROTATE_INTERVALS } from "./themes";

const ThemeContext = createContext(null);

const LS_KEYS = {
  theme:     "anonymx-theme",
  autoRot:   "anonymx-auto-rotate",
  interval:  "anonymx-rotate-interval",
  random:    "anonymx-rotate-random",
};

function applyTheme(themeId) {
  const theme = THEMES.find(t => t.id === themeId);
  if (!theme) return;
  document.body.setAttribute("data-theme", themeId);
  // Apply CSS variables directly for instant switch
  const root = document.documentElement;
  Object.entries(theme.vars).forEach(([key, value]) => {
    root.style.setProperty(key, value);
  });
}

function getNextTheme(currentId, random) {
  const idx = THEMES.findIndex(t => t.id === currentId);
  if (random) {
    let next;
    do { next = Math.floor(Math.random() * THEMES.length); } while (next === idx && THEMES.length > 1);
    return THEMES[next].id;
  }
  return THEMES[(idx + 1) % THEMES.length].id;
}

function getRandomTheme() {
  return THEMES[Math.floor(Math.random() * THEMES.length)].id;
}

export function ThemeProvider({ children }) {
  const [theme, setThemeState] = useState(() => {
    const saved = localStorage.getItem(LS_KEYS.theme);
    return saved && THEMES.find(t => t.id === saved) ? saved : THEMES[0].id;
  });

  const [autoRotate, setAutoRotateState] = useState(() => {
    return localStorage.getItem(LS_KEYS.autoRot) === "true";
  });

  const [rotateInterval, setRotateIntervalState] = useState(() => {
    const saved = localStorage.getItem(LS_KEYS.interval);
    return saved && ROTATE_INTERVALS.find(r => r.id === saved) ? saved : "15m";
  });

  const [randomMode, setRandomModeState] = useState(() => {
    return localStorage.getItem(LS_KEYS.random) === "true";
  });

  const timerRef = useRef(null);

  // Set theme
  const setTheme = useCallback((id) => {
    setThemeState(id);
    localStorage.setItem(LS_KEYS.theme, id);
    applyTheme(id);
  }, []);

  // Toggle auto-rotate
  const setAutoRotate = useCallback((val) => {
    setAutoRotateState(val);
    localStorage.setItem(LS_KEYS.autoRot, String(val));
  }, []);

  // Set interval
  const setRotateInterval = useCallback((val) => {
    setRotateIntervalState(val);
    localStorage.setItem(LS_KEYS.interval, val);
  }, []);

  // Set random mode
  const setRandomMode = useCallback((val) => {
    setRandomModeState(val);
    localStorage.setItem(LS_KEYS.random, String(val));
  }, []);

  // Apply theme on mount
  useEffect(() => {
    // Handle "On Launch" mode — pick a random theme on first mount
    const isLaunchMode = autoRotate && rotateInterval === "launch";
    if (isLaunchMode) {
      const randomId = getRandomTheme();
      setThemeState(randomId);
      localStorage.setItem(LS_KEYS.theme, randomId);
      applyTheme(randomId);
    } else {
      applyTheme(theme);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Auto-rotate timer
  useEffect(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }

    if (!autoRotate || rotateInterval === "launch") return;

    const intervalDef = ROTATE_INTERVALS.find(r => r.id === rotateInterval);
    if (!intervalDef || intervalDef.ms === 0) return;

    timerRef.current = setInterval(() => {
      setThemeState(prev => {
        const nextId = getNextTheme(prev, randomMode);
        localStorage.setItem(LS_KEYS.theme, nextId);
        applyTheme(nextId);
        return nextId;
      });
    }, intervalDef.ms);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [autoRotate, rotateInterval, randomMode]);

  const currentTheme = THEMES.find(t => t.id === theme) || THEMES[0];

  const value = {
    theme,
    currentTheme,
    setTheme,
    autoRotate,
    setAutoRotate,
    rotateInterval,
    setRotateInterval,
    randomMode,
    setRandomMode,
    themes: THEMES,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}
