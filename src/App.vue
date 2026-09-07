<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import appIcon from "./assets/app-icon.png";
import { addEntry, parseDictionary, searchEntries, serializeDictionary, updateEntry, type DictionaryDocument, type DictionaryEntry, type EditResult } from "./domain/dictionary";
import { bootstrap, readDictionary, rememberSelection, saveDictionary } from "./services/rime";
import { ACCENT_COLORS, applyThemePreference, loadThemePreference, saveThemePreference, THEME_MODES, type AccentColor, type ThemeMode } from "./services/theme";

const { t } = useI18n();
const ENTRY_RENDER_BATCH_SIZE = 300;
const currentFile = ref("");
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
const savedTheme = loadThemePreference(window.localStorage);
const themeMode = ref<ThemeMode>(savedTheme.mode);
const accentColor = ref<AccentColor>(savedTheme.accent);

const visibleEntries = computed(() => document.value ? searchEntries(document.value, query.value) : []);
const renderedEntries = computed(() => visibleEntries.value.slice(0, renderLimit.value));
const fileName = computed(() => currentFile.value.split(/[\\/]/).pop() ?? "");

watch(query, () => { renderLimit.value = ENTRY_RENDER_BATCH_SIZE; });
watch([themeMode, accentColor], () => {
  const preference = { mode: themeMode.value, accent: accentColor.value };
  applyThemePreference(documentElement(), preference);
  saveThemePreference(window.localStorage, preference);
});

function documentElement(): HTMLElement { return window.document.documentElement; }

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
    await rememberSelection(loaded.path);
    notify(t("message.loaded"));
  } catch (error) {
    notify(`${t("message.failed")}: ${String(error)}`, "error");
  } finally { loading.value = false; }
}

async function chooseFile() {
  try {
    const selected = await open({ directory: false, multiple: false, filters: [{ name: t("nav.dictionaryFilter"), extensions: ["yaml"] }], title: t("nav.chooseFile") });
    if (typeof selected === "string") await loadFile(selected);
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
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
  applyThemePreference(documentElement(), { mode: themeMode.value, accent: accentColor.value });
  try {
    const state = await bootstrap();
    if (state.file) await loadFile(state.file);
  } catch (error) { notify(`${t("message.failed")}: ${String(error)}`, "error"); }
  finally { loading.value = false; }
});
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div class="brand"><img :src="appIcon" alt="" /><strong>Rime Dict Studio</strong></div>
      <details class="theme-control">
        <summary :title="t('theme.title')" :aria-label="t('theme.title')">◐</summary>
        <div class="theme-panel">
          <span class="setting-label">{{ t('theme.mode') }}</span>
          <div class="segmented">
            <button v-for="mode in THEME_MODES" :key="mode" :class="{ active: themeMode === mode }" @click.prevent="themeMode = mode">{{ t(`theme.${mode}`) }}</button>
          </div>
          <span class="setting-label">{{ t('theme.color') }}</span>
          <div class="swatches">
            <button v-for="color in ACCENT_COLORS" :key="color" :class="[`swatch-${color}`, { active: accentColor === color }]" :aria-label="t(`theme.${color}`)" :title="t(`theme.${color}`)" @click.prevent="accentColor = color" />
          </div>
        </div>
      </details>
    </header>

    <section class="filebar">
      <div class="current-file"><span class="file-icon">⌘</span><div><strong>{{ fileName || t('status.noFileShort') }}</strong><small v-if="fileName">{{ dirty ? t('status.unsaved') : t('status.saved') }}</small></div></div>
      <button class="file-picker secondary" @click="chooseFile">{{ t('nav.chooseFile') }}</button>
    </section>

    <section v-if="loading" class="state-card"><span class="spinner" />{{ t('status.loading') }}</section>
    <section v-else-if="!document" class="state-card empty-state">
      <img :src="appIcon" alt="" />
      <strong>{{ t('status.noFile') }}</strong>
      <button class="primary" @click="chooseFile">{{ t('nav.chooseFile') }}</button>
    </section>
    <template v-else>
      <section class="toolbar">
        <label class="search"><span>⌕</span><input v-model="query" :placeholder="t('editor.search')" /></label>
        <span class="entry-count">{{ visibleEntries.length }} {{ t('editor.entries') }}</span>
        <button class="primary" @click="resetForm()">＋ {{ t('editor.add') }}</button>
      </section>

      <section class="table-card">
        <div class="table-head"><span>{{ t('editor.phrase') }}</span><span>{{ t('editor.code') }}</span><span>{{ t('editor.weight') }}</span><span /></div>
        <button v-for="entry in renderedEntries" :key="entry.id" class="entry-row" @click="resetForm(entry)">
          <strong>{{ entry.phrase }}</strong><code>{{ entry.code }}</code><span>{{ entry.weight ?? '—' }}</span><span class="edit">{{ t('editor.edit') }}</span>
        </button>
        <div v-if="visibleEntries.length === 0" class="empty-list">{{ t('status.empty') }}</div>
        <button v-else-if="renderedEntries.length < visibleEntries.length" class="load-more" @click="renderLimit += ENTRY_RENDER_BATCH_SIZE">{{ t('editor.loadMore', { shown: renderedEntries.length, total: visibleEntries.length }) }}</button>
      </section>

      <footer><span>{{ t('editor.manualDeploy') }}</span><button class="save" :disabled="!dirty || saving" @click="save">{{ t(saving ? 'editor.saving' : 'editor.save') }}</button></footer>
    </template>

    <div v-if="showForm" class="modal-backdrop" @click.self="showForm = false">
      <form class="modal" @submit.prevent="submitEntry">
        <div class="modal-header"><strong>{{ t(editingId ? 'editor.confirmEdit' : 'editor.add') }}</strong><button type="button" class="icon-button" @click="showForm = false">×</button></div>
        <div class="modal-body">
          <label><span>{{ t('editor.phrase') }}</span><input v-model="draft.phrase" autofocus required /></label>
          <label><span>{{ t('editor.code') }}</span><input v-model="draft.code" pattern="[A-Za-z]+" required autocapitalize="off" /></label>
          <label><span>{{ t('editor.weight') }} · {{ t('editor.optional') }}</span><input v-model="draft.weight" type="number" min="1" step="1" /></label>
        </div>
        <div class="modal-footer"><button type="button" class="secondary" @click="showForm = false">{{ t('editor.cancel') }}</button><button class="primary" type="submit">{{ t(editingId ? 'editor.confirmEdit' : 'editor.confirmAdd') }}</button></div>
      </form>
    </div>
    <Transition name="toast"><div v-if="toast" class="toast" :class="toast.kind">{{ toast.text }}</div></Transition>
  </main>
</template>
