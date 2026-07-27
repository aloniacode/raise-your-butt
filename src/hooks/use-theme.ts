import { useCallback, useEffect, useState } from "react";

export type Theme = "light" | "dark" | "system";

const STORAGE_KEY = "theme";

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(theme: Theme) {
  const isDark =
    theme === "dark" || (theme === "system" && getSystemTheme() === "dark");
  document.documentElement.classList.toggle("dark", isDark);
}

/**
 * Theme hook: light / dark / system. Defaults to "system" (follows the OS).
 *
 * The inline script in index.html applies the initial theme before React
 * mounts to avoid FOUC; this hook keeps <html class="dark"> in sync afterwards
 * and persists the choice to localStorage.
 */
export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    if (typeof window === "undefined") return "system";
    return (localStorage.getItem(STORAGE_KEY) as Theme | null) ?? "system";
  });

  // Apply on every change + listen to the OS theme while in "system" mode.
  useEffect(() => {
    applyTheme(theme);
    localStorage.setItem(STORAGE_KEY, theme);

    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => applyTheme("system");
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  const setTheme = useCallback((t: Theme) => setThemeState(t), []);

  return { theme, setTheme };
}
