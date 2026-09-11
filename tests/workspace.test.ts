import { describe, expect, it } from "vitest";
import { createDictionaryWorkspace } from "../src/domain/workspace";

const dictionary = (name: string, body: string) => ({
  path: `/rime/${name}.dict.yaml`,
  content: `---\nname: ${name}\n...\n${body}`,
});

describe("DictionaryWorkspace", () => {
  it("searches all documents and reports entry sources", () => {
    const workspace = createDictionaryWorkspace("/rime/root.dict.yaml", [
      dictionary("root", "甲\ta\t10\n"),
      dictionary("extra", "乙\ta\t20\n"),
    ]);

    expect(workspace.search("a")).toMatchObject([
      { phrase: "甲", sourceName: "root.dict.yaml" },
      { phrase: "乙", sourceName: "extra.dict.yaml" },
    ]);
    expect(workspace.collisions("a")).toHaveLength(2);
  });

  it("adds to the selected root and edits entries in their source files", () => {
    const workspace = createDictionaryWorkspace("/rime/root.dict.yaml", [
      dictionary("root", "甲\ta\t10\n"),
      dictionary("extra", "乙\tb\t20\n"),
    ]);

    expect(workspace.add({ phrase: "丙", code: "c", weight: 30 }).ok).toBe(true);
    const imported = workspace.search("乙")[0];
    expect(workspace.update(imported, { phrase: "乙", code: "b", weight: 99 }).ok).toBe(true);

    const changes = workspace.changes();
    expect(changes).toHaveLength(2);
    expect(changes.find((item) => item.path.endsWith("root.dict.yaml"))?.content).toContain("丙\tc\t30");
    expect(changes.find((item) => item.path.endsWith("extra.dict.yaml"))?.content).toContain("乙\tb\t99");
  });

  it("rejects exact duplicates across the whole group", () => {
    const workspace = createDictionaryWorkspace("/rime/root.dict.yaml", [
      dictionary("root", "甲\ta\t10\n"),
      dictionary("extra", "乙\tb\t20\n"),
    ]);

    expect(workspace.add({ phrase: "乙", code: "b", weight: null })).toEqual({ ok: false, error: "duplicate" });
  });
});
