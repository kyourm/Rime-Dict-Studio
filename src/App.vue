<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import appIcon from "./assets/app-icon.png";
import { createDictionaryWorkspace, type DictionaryWorkspace, type WorkspaceEntry } from "./domain/workspace";
import { bootstrap, readDictionaryGroup, rememberSelection, saveDictionaries } from "./services/rime";
import { ACCENT_COLORS, applyThemePreference, loadThemePreference, saveThemePreference, THEME_MODES, type AccentColor, type ThemeMode } from "./services/theme";

const { t } = useI18n();
const ENTRY_RENDER_BATCH_SIZE = 100;
const COLLISION_RENDER_LIMIT = 20;
const MINIMUM_WEIGHT = 1;
const WEIGHT_STEP = 1;
const currentFile = ref("");
const workspace = ref<DictionaryWorkspace | null>(null);
const query = ref("");
const loading = ref(true);
const saving = ref(false);
const dirty = ref(false);
const showForm = ref(false);
const editingId = ref<string | null>(null);
const editingEntry = ref<WorkspaceEntry | null>(null);
const toast = ref<{ text: string; kind: "success" | "error" } | null>(null);
const renderLimit = ref(ENTRY_RENDER_BATCH_SIZE);
const workspaceRevision = ref(0);
const draft = reactive({ phrase: "", code: "", weight: "" });
const savedTheme = loadThemePreference(window.localStorage);
const themeMode = ref<ThemeMode>(savedTheme.mode);
const accentColor = ref<AccentColor>(savedTheme.accent);

const visibleEntries = computed(() => { workspaceRevision.value; return workspace.value?.search(query.value) ?? []; });
const renderedEntries = computed(() => visibleEntries.value.slice(0, renderLimit.value));
const fileName = computed(() => currentFile.value.split(/[\\/]/).pop() ?? "");
const collisions = computed(() => { workspaceRevision.value; return workspace.value?.collisions(draft.code).filter((entry) => !editingEntry.value || entry.sourcePath !== editingEntry.value.sourcePath || entry.id !== editingEntry.value.id) ?? []; });

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
    const loaded = await readDictionaryGroup(path);
    renderLimit.value = ENTRY_RENDER_BATCH_SIZE;
    workspace.value = createDictionaryWorkspace(loaded.rootPath, loaded.files);
    workspaceRevision.value += 1;
    currentFile.value = loaded.rootPath;
    dirty.value = false;
    await rememberSelection(loaded.rootPath);
    notify(loaded.warnings.length ? t("message.loadedWithWarnings", { count: loaded.warnings.length }) : t("message.loaded"));
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

function resetForm(entry?: WorkspaceEntry) {
  editingId.value = entry?.id ?? null;
  editingEntry.value = entry ?? null;
  draft.phrase = entry?.phrase ?? "";
  draft.code = entry?.code ?? "";
  draft.weight = entry?.weight?.toString() ?? "";
  showForm.value = true;
}

function submitEntry() {
  if (!workspace.value) return;
  const input = { phrase: draft.phrase, code: draft.code, weight: draft.weight === "" ? null : Number(draft.weight) };
  const result = editingEntry.value ? workspace.value.update(editingEntry.value, input) : workspace.value.add(input);
  if (!result.ok) {
    notify(t(result.error === "duplicate" ? "message.duplicate" : "message.invalid"), "error");
    return;
  }
  markWorkspaceChanged();
  showForm.value = false;
}

function markWorkspaceChanged() {
  dirty.value = true;
  workspaceRevision.value += 1;
}

function adjustWeight(entry: WorkspaceEntry, delta: number) {
  if (!workspace.value) return;
  const current = entry.weight ?? MINIMUM_WEIGHT;
  const result = workspace.value.update(entry, { phrase: entry.phrase, code: entry.code, weight: Math.max(MINIMUM_WEIGHT, current + delta) });
  if (!result.ok) { notify(t("message.invalid"), "error"); return; }
  markWorkspaceChanged();
}

async function save() {
  if (!workspace.value || !currentFile.value) return;
  saving.value = true;
  try {
    const changes = workspace.value.changes();
    await saveDictionaries(changes);
    workspace.value.markSaved();
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
    <section v-else-if="!workspace" class="state-card empty-state">
      <img :src="appIcon" alt="" />
      <strong>{{ t('status.noFile') }}</strong>
      <button class="primary" @click="chooseFile">{{ t('nav.chooseFile') }}</button>
    </section>
    <template v-else-if="workspace">
      <section class="toolbar">
        <label class="search"><span>⌕</span><input v-model="query" :placeholder="t('editor.search')" /></label>
        <span class="entry-count">{{ t('editor.groupSummary', { files: workspace.fileCount, entries: workspace.entryCount }) }}</span>
        <button class="primary" @click="resetForm()">＋ {{ t('editor.add') }}</button>
      </section>

      <section class="table-card">
        <div class="table-head"><span>{{ t('editor.phrase') }}</span><span>{{ t('editor.code') }}</span><span>{{ t('editor.weight') }}</span><span>{{ t('editor.source') }}</span></div>
        <div v-for="entry in renderedEntries" :key="`${entry.sourcePath}:${entry.id}`" class="entry-row" role="button" tabindex="0" @click="resetForm(entry)" @keydown.enter="resetForm(entry)">
          <strong>{{ entry.phrase }}</strong><code>{{ entry.code }}</code>
          <span class="weight-control" @click.stop>
            <button :aria-label="t('editor.decreaseWeight')" @click="adjustWeight(entry, -1)">−</button>
            <span>{{ entry.weight ?? '—' }}</span>
            <button :aria-label="t('editor.increaseWeight')" @click="adjustWeight(entry, 1)">＋</button>
          </span>
          <span class="source" :title="entry.sourcePath">{{ entry.sourceName }}</span>
        </div>
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
          <label><span>{{ t('editor.code') }}</span><input v-model="draft.code" required autocapitalize="off" /></label>
          <section v-if="draft.code && collisions.length" class="collision-panel">
            <strong>{{ t('editor.collisions', { count: collisions.length }) }}</strong>
            <div v-for="entry in collisions.slice(0, COLLISION_RENDER_LIMIT)" :key="`${entry.sourcePath}:${entry.id}`" class="collision-row">
              <span>{{ entry.phrase }}</span><span>{{ entry.weight ?? '—' }}</span><small>{{ entry.sourceName }}</small>
            </div>
            <small v-if="collisions.length > COLLISION_RENDER_LIMIT">{{ t('editor.moreCollisions', { count: collisions.length - COLLISION_RENDER_LIMIT }) }}</small>
          </section>
          <label><span>{{ t('editor.weight') }} · {{ t('editor.optional') }}</span><input v-model="draft.weight" type="number" :min="MINIMUM_WEIGHT" :step="WEIGHT_STEP" /></label>
        </div>
        <div class="modal-footer"><button type="button" class="secondary" @click="showForm = false">{{ t('editor.cancel') }}</button><button class="primary" type="submit">{{ t(editingId ? 'editor.confirmEdit' : 'editor.confirmAdd') }}</button></div>
      </form>
    </div>
    <Transition name="toast"><div v-if="toast" class="toast" :class="toast.kind">{{ toast.text }}</div></Transition>
  </main>
</template>
