import { addEntry, parseDictionary, serializeDictionary, updateEntry, type DictionaryDocument, type DictionaryEntry, type EditResult, type EntryDraft } from "./dictionary";

export interface DictionarySource { path: string; content: string }
export interface WorkspaceEntry extends DictionaryEntry { sourcePath: string; sourceName: string }
export interface DictionaryChange { path: string; content: string }
export interface DictionaryWorkspace {
  readonly rootPath: string;
  readonly fileCount: number;
  readonly entryCount: number;
  search(query: string): WorkspaceEntry[];
  collisions(code: string): WorkspaceEntry[];
  add(draft: EntryDraft): EditResult;
  update(entry: WorkspaceEntry, draft: EntryDraft): EditResult;
  changes(): DictionaryChange[];
  markSaved(): void;
}

interface WorkspaceDocument { document: DictionaryDocument; path: string; name: string; dirty: boolean }

function fileName(path: string): string { return path.split(/[\\/]/).pop() ?? path; }
function entriesOf(source: WorkspaceDocument): WorkspaceEntry[] {
  return source.document.entries.map((entry) => ({ ...entry, sourcePath: source.path, sourceName: source.name }));
}

export function createDictionaryWorkspace(rootPath: string, sources: DictionarySource[]): DictionaryWorkspace {
  const documents = new Map<string, WorkspaceDocument>();
  for (const source of sources) {
    documents.set(source.path, { path: source.path, name: fileName(source.path), document: parseDictionary(source.content), dirty: false });
  }
  const root = documents.get(rootPath);
  if (!root) throw new Error("missing-root");
  let allEntries: WorkspaceEntry[] = [];
  let codeIndex = new Map<string, WorkspaceEntry[]>();
  const rebuildIndex = () => {
    allEntries = Array.from(documents.values()).flatMap(entriesOf);
    codeIndex = new Map<string, WorkspaceEntry[]>();
    for (const entry of allEntries) {
      const matches = codeIndex.get(entry.code) ?? [];
      matches.push(entry);
      codeIndex.set(entry.code, matches);
    }
  };
  rebuildIndex();
  const duplicateExists = (draft: EntryDraft, current?: WorkspaceEntry) => allEntries.some((entry) =>
    !(current && entry.sourcePath === current.sourcePath && entry.id === current.id)
    && entry.phrase === draft.phrase.trim() && entry.code === draft.code,
  );

  return {
    rootPath,
    fileCount: documents.size,
    get entryCount() { return allEntries.length; },
    search(query) {
      const needle = query.trim().toLocaleLowerCase();
      return needle ? allEntries.filter((entry) => entry.phrase.toLocaleLowerCase().includes(needle) || entry.code.toLocaleLowerCase().includes(needle)) : allEntries;
    },
    collisions(code) { return codeIndex.get(code) ?? []; },
    add(draft) {
      if (duplicateExists(draft)) return { ok: false, error: "duplicate" };
      const result = addEntry(root.document, draft);
      if (result.ok) { root.dirty = true; rebuildIndex(); }
      return result;
    },
    update(entry, draft) {
      if (duplicateExists(draft, entry)) return { ok: false, error: "duplicate" };
      const source = documents.get(entry.sourcePath);
      if (!source) return { ok: false, error: "not-found" };
      const result = updateEntry(source.document, entry.id, draft);
      if (result.ok) { source.dirty = true; rebuildIndex(); }
      return result;
    },
    changes() { return Array.from(documents.values()).filter((source) => source.dirty).map((source) => ({ path: source.path, content: serializeDictionary(source.document) })); },
    markSaved() { for (const source of documents.values()) source.dirty = false; },
  };
}
