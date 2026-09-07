import { describe, expect, it } from "vitest";
import {
  addEntry,
  parseDictionary,
  searchEntries,
  serializeDictionary,
  updateEntry,
} from "../src/domain/dictionary";

const fixture = `---\nname: sample\nsort: by_weight\n# header comment\n...\n## 工作\n工号\taakg\t10\n竞业\tukog\t8\n# body comment\n`;

describe("词典编辑公共接口", () => {
  it("保留头部、分组与注释并解析词条", () => {
    const document = parseDictionary(fixture);
    expect(document.entries.map(({ phrase, code, weight }) => ({ phrase, code, weight }))).toEqual([
      { phrase: "工号", code: "aakg", weight: 10 },
      { phrase: "竞业", code: "ukog", weight: 8 },
    ]);
    expect(serializeDictionary(document)).toBe(fixture);
  });

  it("按词组或编码搜索且忽略大小写", () => {
    const document = parseDictionary(fixture);
    expect(searchEntries(document, "AAKG").map((entry) => entry.phrase)).toEqual(["工号"]);
    expect(searchEntries(document, "竞").map((entry) => entry.code)).toEqual(["ukog"]);
  });

  it("新增合法词条并拒绝完全重复", () => {
    const document = parseDictionary(fixture);
    expect(addEntry(document, { phrase: "编码配置", code: "xdcf", weight: 12 }).ok).toBe(true);
    expect(addEntry(document, { phrase: "工号", code: "aakg", weight: 1 })).toEqual({
      ok: false,
      error: "duplicate",
    });
  });

  it("修改词条并校验五笔编码和权重", () => {
    const document = parseDictionary(fixture);
    expect(updateEntry(document, document.entries[0].id, { phrase: "工号", code: "Aakg", weight: 999 }).ok).toBe(true);
    expect(document.entries[0]).toMatchObject({ code: "aakg", weight: 999 });
    expect(updateEntry(document, document.entries[0].id, { phrase: "工号", code: "a1", weight: 0 })).toEqual({
      ok: false,
      error: "invalid-code",
    });
  });
});
