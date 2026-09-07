export interface DictionaryEntry {
  id: string;
  phrase: string;
  code: string;
  weight: number | null;
  original?: string;
}

type BodyLine =
  | { kind: "raw"; value: string }
  | { kind: "entry"; entry: DictionaryEntry };

export interface DictionaryDocument {
  header: string;
  lines: BodyLine[];
  entries: DictionaryEntry[];
  trailingNewline: boolean;
  lineEnding: "\n" | "\r\n";
}

export interface EntryDraft {
  phrase: string;
  code: string;
  weight: number | null;
}

export type EditResult = { ok: true } | { ok: false; error: "empty-phrase" | "invalid-code" | "invalid-weight" | "duplicate" | "not-found" };

const ENTRY_PATTERN = /^([^\t]+)\t([a-zA-Z]+)(?:\t(\d+))?$/;

function normalizedDraft(draft: EntryDraft): EntryDraft {
  return { phrase: draft.phrase.trim(), code: draft.code.trim().toLowerCase(), weight: draft.weight };
}

function validate(document: DictionaryDocument, draft: EntryDraft, currentId?: string): EditResult {
  if (!draft.phrase) return { ok: false, error: "empty-phrase" };
  if (!/^[a-z]+$/.test(draft.code)) return { ok: false, error: "invalid-code" };
  if (draft.weight !== null && (!Number.isInteger(draft.weight) || draft.weight < 1)) return { ok: false, error: "invalid-weight" };
  const duplicate = document.entries.some((entry) => entry.id !== currentId && entry.phrase === draft.phrase && entry.code.toLowerCase() === draft.code);
  return duplicate ? { ok: false, error: "duplicate" } : { ok: true };
}

export function parseDictionary(content: string): DictionaryDocument {
  const lineEnding = content.includes("\r\n") ? "\r\n" : "\n";
  const trailingNewline = content.endsWith("\n");
  const allLines = content.split(lineEnding);
  if (trailingNewline) allLines.pop();
  const markerIndex = allLines.findIndex((line) => line.trim() === "...");
  if (markerIndex < 0) throw new Error("missing-marker");
  const header = `${allLines.slice(0, markerIndex + 1).join(lineEnding)}${lineEnding}`;
  const entries: DictionaryEntry[] = [];
  const lines: BodyLine[] = allLines.slice(markerIndex + 1).map((value, index) => {
    const match = value.match(ENTRY_PATTERN);
    if (!match) return { kind: "raw", value };
    const entry: DictionaryEntry = {
      id: `line-${markerIndex + 1 + index}`,
      phrase: match[1],
      code: match[2],
      weight: match[3] === undefined ? null : Number(match[3]),
      original: value,
    };
    entries.push(entry);
    return { kind: "entry", entry };
  });
  return { header, lines, entries, trailingNewline, lineEnding };
}

export function serializeDictionary(document: DictionaryDocument): string {
  const body = document.lines.map((line) => line.kind === "raw" ? line.value : line.entry.original ?? [line.entry.phrase, line.entry.code, line.entry.weight].filter((value) => value !== null).join("\t")).join(document.lineEnding);
  return `${document.header}${body}${document.trailingNewline ? document.lineEnding : ""}`;
}

export function searchEntries(document: DictionaryDocument, query: string): DictionaryEntry[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return document.entries;
  return document.entries.filter((entry) => entry.phrase.toLocaleLowerCase().includes(needle) || entry.code.toLocaleLowerCase().includes(needle));
}

export function addEntry(document: DictionaryDocument, input: EntryDraft): EditResult {
  const draft = normalizedDraft(input);
  const result = validate(document, draft);
  if (!result.ok) return result;
  const entry: DictionaryEntry = { id: `new-${crypto.randomUUID()}`, ...draft };
  document.entries.push(entry);
  document.lines.push({ kind: "entry", entry });
  return { ok: true };
}

export function updateEntry(document: DictionaryDocument, id: string, input: EntryDraft): EditResult {
  const draft = normalizedDraft(input);
  const result = validate(document, draft, id);
  if (!result.ok) return result;
  const entry = document.entries.find((candidate) => candidate.id === id);
  if (!entry) return { ok: false, error: "not-found" };
  Object.assign(entry, draft);
  entry.original = undefined;
  return { ok: true };
}
