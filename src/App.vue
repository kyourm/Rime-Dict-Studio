<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import { addEntry, parseDictionary, searchEntries, serializeDictionary, updateEntry, type DictionaryDocument, type DictionaryEntry, type EditResult } from "./domain/dictionary";
import { bootstrap, listDictionaries, readDictionary, rememberSelection, saveDictionary } from "./services/rime";

const { t, locale } = useI18n();
const ENTRY_RENDER_BATCH_SIZE = 300;
const directory = ref("");
const currentFile = ref("");
const dictionaries = ref<string[]>([]);
const document = ref<DictionaryDocument | null>(null);
const query = ref("");
const loading = ref(true);
const saving = ref(false);
const dirty = ref(false);
const showForm = ref(false);
const editingId = ref<string | null>(null);
const toast = ref<{ text: string; kind: "success" | "error" } | null>(null);
const renderLimit = ref(ENTRY_RENDER_BATCH_SIZE);
const draft = reactive({ phrase: "", code: "", weight: "" });

const visibleEntries = computed(() => document.value ? searchEntries(document.value, query.value) : []);
const renderedEntries = computed(() => visibleEntries.value.slice(0, renderLimit.value));
const fileName = computed(() => currentFile.value.split(/[\\/]/).pop() ?? "");

watch(query, () => { renderLimit.value = ENTRY_RENDER_BATCH_SIZE; });

function notify(text: string, kind: "success" | "error" = "success") {
  toast.value = { text, kind };
  window.setTimeout(() => { toast.value = null; }, 2800);
}

async function loadFile(path: string) {
  loading.value = true;
  try {
    const loaded = await readDictionary(path);
    renderLimit.value = ENTRY_RENDER_BATCH_SIZE;
    document.value = parseDictionary(loaded.content);
    currentFile.value = loaded.path;
    dirty.value = false;
    if (directory.value) await rememberSelection(directory.value, loaded.path);
    notify(t("message.loaded"));
  } catch (error) {
    notify(`${t("message.failed")}: ${String(error)}`, "error");
  } finally { loading.value = false; }
}

async function chooseDirectory() {
  loading.value = true;
  try {
    const selected = await open({ directory: true, multiple: false, title: t("nav.chooseDirectory") });
    if (typeof selected !== "string") return;
    directory.value = selected;
    dictionaries.value = await listDictionaries(selected);
    currentFile.value = "";
    document.value = null;
    if (dictionaries.value.length > 0) await loadFile(dictionaries.value[0]);
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
  finally { loading.value = false; }
}

async function chooseFile() {
  loading.value = true;
  try {
    const selected = await open({ directory: false, multiple: false, filters: [{ name: t("nav.dictionaryFilter"), extensions: ["yaml"] }], title: t("nav.chooseFile") });
    if (typeof selected !== "string") return;
    directory.value = selected.replace(/[\\/][^\\/]+$/, "");
    dictionaries.value = await listDictionaries(directory.value);
    await loadFile(selected);
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
  finally { loading.value = false; }
}

function resetForm(entry?: DictionaryEntry) {
  editingId.value = entry?.id ?? null;
  draft.phrase = entry?.phrase ?? "";
  draft.code = entry?.code ?? "";
  draft.weight = entry?.weight?.toString() ?? "";
  showForm.value = true;
}

function submitEntry() {
  if (!document.value) return;
  const input = { phrase: draft.phrase, code: draft.code, weight: draft.weight === "" ? null : Number(draft.weight) };
  const result: EditResult = editingId.value ? updateEntry(document.value, editingId.value, input) : addEntry(document.value, input);
  if (!result.ok) {
    notify(t(result.error === "duplicate" ? "message.duplicate" : "message.invalid"), "error");
    return;
  }
  dirty.value = true;
  showForm.value = false;
}

async function save() {
  if (!document.value || !currentFile.value) return;
  saving.value = true;
  try {
    await saveDictionary(currentFile.value, serializeDictionary(document.value));
    dirty.value = false;
    notify(t("message.saved"));
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
  finally { saving.value = false; }
}

onMounted(async () => {
  try {
    const state = await bootstrap();
    directory.value = state.directory ?? "";
    dictionaries.value = state.dictionaries;
    if (state.file) await loadFile(state.file);
    else if (state.dictionaries.length > 0) await loadFile(state.dictionaries[0]);
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
  finally { loading.value = false; }
});
</script>

<template>
  <div class="shell">
    <aside class="sidebar">
      <div class="brand"><span class="brand-mark">中</span><div><strong>Rime</strong><small>DICT STUDIO</small></div></div>
      <div class="side-label">{{ t('nav.dictionary') }}</div>
      <div class="dictionary-list">
        <button v-for="path in dictionaries" :key="path" class="dict-item" :class="{ active: path === currentFile }" @click="loadFile(path)">
          <span class="file-glyph">辞</span><span><strong>{{ path.split(/[\\/]/).pop() }}</strong><small>{{ path === currentFile ? t(dirty ? 'status.unsaved' : 'status.saved') : '' }}</small></span>
        </button>
        <div v-if="dictionaries.length === 0" class="side-empty">{{ t('nav.noDictionaries') }}</div>
      </div>
      <div class="sidebar-actions">
        <button class="ghost" @click="chooseDirectory">＋ {{ t('nav.chooseDirectory') }}</button>
        <button class="ghost" @click="chooseFile">⌁ {{ t('nav.chooseFile') }}</button>
      </div>
    </aside>

    <main>
      <header>
        <div><p class="eyebrow">{{ t('app.eyebrow') }}</p><h1>{{ t('app.title') }}</h1><p class="subtitle">{{ t('app.subtitle') }}</p></div>
        <button class="locale" @click="locale = locale === 'zh-CN' ? 'en' : 'zh-CN'">{{ t('nav.switchLanguage') }}</button>
      </header>

      <section v-if="loading" class="state-card"><span class="spinner" />{{ t('status.loading') }}</section>
      <section v-else-if="!document" class="state-card empty-state"><span class="empty-icon">辞</span><h2>{{ t('status.noFile') }}</h2><button class="primary" @click="chooseDirectory">{{ t('nav.chooseDirectory') }}</button></section>
      <template v-else>
        <section class="toolbar">
          <div class="file-title"><span class="live-dot" /><div><strong>{{ fileName }}</strong><small>{{ visibleEntries.length }} {{ t('editor.entries') }}</small></div></div>
          <div class="toolbar-actions"><label class="search"><span>⌕</span><input v-model="query" :placeholder="t('editor.search')" /></label><button class="primary" @click="resetForm()">＋ {{ t('editor.add') }}</button></div>
        </section>

        <section class="table-card">
          <div class="table-head"><span>{{ t('editor.phrase') }}</span><span>{{ t('editor.code') }}</span><span>{{ t('editor.weight') }}</span><span /></div>
          <button v-for="entry in renderedEntries" :key="entry.id" class="entry-row" @click="resetForm(entry)">
            <strong>{{ entry.phrase }}</strong><code>{{ entry.code }}</code><span class="weight">{{ entry.weight ?? '—' }}</span><span class="edit">{{ t('editor.edit') }} →</span>
          </button>
          <div v-if="visibleEntries.length === 0" class="empty-list">◇<span>{{ t('status.empty') }}</span></div>
          <button v-else-if="renderedEntries.length < visibleEntries.length" class="load-more" @click="renderLimit += ENTRY_RENDER_BATCH_SIZE">
            {{ t('editor.loadMore', { shown: renderedEntries.length, total: visibleEntries.length }) }}
          </button>
        </section>

        <footer><span>{{ t('editor.manualDeploy') }}</span><div class="save-group"><span :class="{ dirty }">{{ t(dirty ? 'status.unsaved' : 'status.saved') }}</span><button class="save" :disabled="!dirty || saving" @click="save">{{ t(saving ? 'editor.saving' : 'editor.save') }}</button></div></footer>
      </template>
    </main>

    <div v-if="showForm" class="modal-backdrop" @click.self="showForm = false">
      <form class="modal" @submit.prevent="submitEntry">
        <div class="modal-header"><div><p class="eyebrow">{{ t('editor.eyebrow') }}</p><h2>{{ t(editingId ? 'editor.confirmEdit' : 'editor.add') }}</h2></div><button type="button" class="close" @click="showForm = false">×</button></div>
        <div class="modal-body">
          <label><span>{{ t('editor.phrase') }}</span><input v-model="draft.phrase" autofocus required /></label>
          <label><span>{{ t('editor.code') }}</span><input v-model="draft.code" pattern="[A-Za-z]+" required autocapitalize="off" /></label>
          <label><span>{{ t('editor.weight') }} <small>{{ t('editor.optional') }}</small></span><input v-model="draft.weight" type="number" min="1" step="1" /></label>
        </div>
        <div class="modal-footer"><button type="button" class="secondary" @click="showForm = false">{{ t('editor.cancel') }}</button><button class="primary" type="submit">{{ t(editingId ? 'editor.confirmEdit' : 'editor.confirmAdd') }}</button></div>
      </form>
    </div>
    <Transition name="toast"><div v-if="toast" class="toast" :class="toast.kind">{{ toast.kind === 'success' ? '✓' : '!' }} {{ toast.text }}</div></Transition>
  </div>
</template>
