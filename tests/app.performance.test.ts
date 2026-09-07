// @vitest-environment happy-dom
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../src/App.vue";
import { i18n } from "../src/i18n";

const dictionaryPath = "/mock/wubi86_jidian.dict.yaml";
const dictionaryBody = Array.from(
  { length: 90_000 },
  (_, index) => `词条${index}\tabcd\t${index + 1}`,
).join("\n");

vi.mock("../src/services/rime", () => ({
  bootstrap: vi.fn(async () => ({
    directory: "/mock",
    file: dictionaryPath,
    dictionaries: Array.from({ length: 80 }, (_, index) => `/mock/dictionary-${index}.dict.yaml`),
  })),
  readDictionary: vi.fn(async () => ({
    path: dictionaryPath,
    content: `---\nname: performance\n...\n${dictionaryBody}\n`,
  })),
  rememberSelection: vi.fn(async () => undefined),
  saveDictionary: vi.fn(async () => undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

describe("大词典界面", () => {
  beforeEach(() => vi.clearAllMocks());

  it("只渲染有限数量的词条，避免主线程被数万行 DOM 卡死", async () => {
    const startedAt = performance.now();
    const wrapper = mount(App, { global: { plugins: [i18n] } });
    await flushPromises();
    expect(wrapper.findAll(".entry-row").length).toBeLessThanOrEqual(300);
    expect(performance.now() - startedAt).toBeLessThan(2_000);
  });

  it("界面只显示当前文件选择入口，不列出目录中的其他词典", async () => {
    const wrapper = mount(App, { global: { plugins: [i18n] } });
    await flushPromises();
    expect(wrapper.find(".dictionary-list").exists()).toBe(false);
    expect(wrapper.find(".file-picker").exists()).toBe(true);
    expect(wrapper.find(".theme-control").exists()).toBe(true);
  });
});
