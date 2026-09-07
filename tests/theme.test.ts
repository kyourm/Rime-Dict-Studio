import { describe, expect, it } from "vitest";
import { loadThemePreference, saveThemePreference, type ThemePreference } from "../src/services/theme";

function memoryStorage(initial?: string): Storage {
  const values = new Map<string, string>();
  if (initial) values.set("rime-dict-studio-theme", initial);
  return {
    get length() { return values.size; },
    clear: () => values.clear(),
    getItem: (key) => values.get(key) ?? null,
    key: (index) => [...values.keys()][index] ?? null,
    removeItem: (key) => values.delete(key),
    setItem: (key, value) => values.set(key, value),
  };
}

describe("主题偏好公共接口", () => {
  it("首次使用默认跟随系统并使用系统主题色", () => {
    expect(loadThemePreference(memoryStorage())).toEqual({ mode: "system", accent: "system" });
  });

  it("保存并恢复用户选择", () => {
    const storage = memoryStorage();
    const preference: ThemePreference = { mode: "dark", accent: "teal" };
    saveThemePreference(storage, preference);
    expect(loadThemePreference(storage)).toEqual(preference);
  });

  it("损坏的本地配置安全回退到系统主题", () => {
    expect(loadThemePreference(memoryStorage("not-json"))).toEqual({ mode: "system", accent: "system" });
  });
});
