import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("Windows 构建子系统", () => {
  it("仅在 Windows 正式构建中隐藏控制台", () => {
    const source = readFileSync(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8");

    expect(source.replace(/\s/g, "")).toContain('#![cfg_attr(all(not(debug_assertions),target_os="windows"),windows_subsystem="windows")]');
  });
});
