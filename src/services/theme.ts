export const THEME_MODES = ["system", "light", "dark"] as const;
export const ACCENT_COLORS = ["system", "blue", "teal", "orange"] as const;
export type ThemeMode = typeof THEME_MODES[number];
export type AccentColor = typeof ACCENT_COLORS[number];

export interface ThemePreference {
  mode: ThemeMode;
  accent: AccentColor;
}

const STORAGE_KEY = "rime-dict-studio-theme";
const DEFAULT_PREFERENCE: ThemePreference = { mode: "system", accent: "system" };

export function loadThemePreference(storage: Storage): ThemePreference {
  try {
    const value: unknown = JSON.parse(storage.getItem(STORAGE_KEY) ?? "null");
    if (typeof value !== "object" || value === null) return { ...DEFAULT_PREFERENCE };
    const candidate = value as Partial<ThemePreference>;
    if (!THEME_MODES.includes(candidate.mode as ThemeMode) || !ACCENT_COLORS.includes(candidate.accent as AccentColor)) return { ...DEFAULT_PREFERENCE };
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
