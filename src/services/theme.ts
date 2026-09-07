export type ThemeMode = "system" | "light" | "dark";
export type AccentColor = "system" | "blue" | "teal" | "orange";

export interface ThemePreference {
  mode: ThemeMode;
  accent: AccentColor;
}

const STORAGE_KEY = "rime-dict-studio-theme";
const DEFAULT_PREFERENCE: ThemePreference = { mode: "system", accent: "system" };
const MODES: ThemeMode[] = ["system", "light", "dark"];
const ACCENTS: AccentColor[] = ["system", "blue", "teal", "orange"];

export function loadThemePreference(storage: Storage): ThemePreference {
  try {
    const value: unknown = JSON.parse(storage.getItem(STORAGE_KEY) ?? "null");
    if (typeof value !== "object" || value === null) return { ...DEFAULT_PREFERENCE };
    const candidate = value as Partial<ThemePreference>;
    if (!MODES.includes(candidate.mode as ThemeMode) || !ACCENTS.includes(candidate.accent as AccentColor)) return { ...DEFAULT_PREFERENCE };
    return { mode: candidate.mode as ThemeMode, accent: candidate.accent as AccentColor };
  } catch {
    return { ...DEFAULT_PREFERENCE };
  }
}

export function saveThemePreference(storage: Storage, preference: ThemePreference): void {
  storage.setItem(STORAGE_KEY, JSON.stringify(preference));
}

export function applyThemePreference(root: HTMLElement, preference: ThemePreference): void {
  if (preference.mode === "system") root.removeAttribute("data-theme");
  else root.dataset.theme = preference.mode;
  root.dataset.accent = preference.accent;
}
