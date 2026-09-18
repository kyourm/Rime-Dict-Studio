// @vitest-environment happy-dom
import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import App from "../src/App.vue";
import { i18n } from "../src/i18n";
import { deployRime, saveDictionaries } from "../src/services/rime";

const operations: string[] = [];

vi.mock("../src/services/rime", () => ({
  bootstrap: vi.fn(async () => ({ file: "/mock/user.dict.yaml" })),
  readDictionaryGroup: vi.fn(async () => ({
    rootPath: "/mock/user.dict.yaml",
    files: [{ path: "/mock/user.dict.yaml", content: "---\nname: user\n...\n已有\ta\t10\n" }],
    warnings: [],
  })),
  rememberSelection: vi.fn(async () => undefined),
  saveDictionaries: vi.fn(async () => { operations.push("save"); }),
  deploymentState: vi.fn(async () => ({ available: true, label: "squirrel", executable: "/mock/Squirrel", arguments: ["--reload"], workingDirectory: "/mock", experimental: false, custom: false })),
  saveDeploymentConfig: vi.fn(async () => undefined),
  deployRime: vi.fn(async () => { operations.push("deploy"); }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

describe("保存并重新部署", () => {
  it("先安全保存修改，再调用部署器", async () => {
    operations.length = 0;
    const wrapper = mount(App, { global: { plugins: [i18n] } });
    await flushPromises();
    await wrapper.find(".toolbar .primary").trigger("click");
    const inputs = wrapper.findAll(".modal-body input");
    await inputs[0].setValue("新增");
    await inputs[1].setValue("b");
    await wrapper.find(".modal").trigger("submit");
    await wrapper.find("footer .save").trigger("click");
    await flushPromises();

    expect(vi.mocked(saveDictionaries)).toHaveBeenCalledOnce();
    expect(vi.mocked(deployRime)).toHaveBeenCalledOnce();
    expect(operations).toEqual(["save", "deploy"]);
    expect(wrapper.text()).toContain("已保存");
  });
});
