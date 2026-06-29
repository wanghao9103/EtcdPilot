<template>
  <section class="panel">
    <div class="section-head">
      <div>
        <h2>{{ t('keys.title') }}</h2>
        <p class="hint section-desc">{{ t('keys.desc') }}</p>
      </div>
    </div>

    <div class="filter-bar">
      <label class="filter-item">
        <span>{{ t('keys.env') }}</span>
        <select v-model="clusterId">
          <option v-for="c in clusters" :key="c.id" :value="c.id">{{ c.name }}</option>
        </select>
      </label>
      <label class="filter-item grow">
        <span>{{ t('keys.dataDir') }}</span>
        <input v-model="prefix" :placeholder="t('keys.dataDirPh')" @keydown.enter="queryKeys" />
      </label>
      <button type="button" class="primary btn-icon" @click="queryKeys" :disabled="loadingList">
        <span v-if="loadingList" class="spinner"></span>
        {{ loadingList ? t('common.querying') : t('common.query') }}
      </button>
    </div>

    <div class="edit-panel">
      <p class="panel-label">{{ t('keys.editData') }}</p>
      <div class="edit-grid">
        <label class="edit-field">
          <span>{{ t('keys.dataPath') }}</span>
          <input v-model="editingKey" :placeholder="t('keys.dataPathPh')" class="mono" @keydown.enter="queryByKey" />
        </label>
        <div class="edit-field content-field">
          <span class="edit-field-label">{{ t('keys.content') }}</span>
          <ValueEditor
            ref="valueEditorRef"
            v-model="editingValue"
            :placeholder="t('keys.contentPh')"
          />
        </div>
      </div>
      <div class="edit-actions">
        <button type="button" class="primary" @click="saveItem" :disabled="!canWrite || !editingKey.trim() || saving">
          {{ saving ? t('common.saving') : t('common.save') }}
        </button>
        <button
          type="button"
          class="danger"
          @click="deleteItem"
          :disabled="!canDelete || !editingKey.trim() || deleting"
        >
          {{ deleting ? t('common.deleting') : t('common.delete') }}
        </button>
        <button type="button" @click="queryByKey" :disabled="!canRead || !editingKey.trim() || loadingOne">
          {{ loadingOne ? t('common.loading') : t('keys.reload') }}
        </button>
        <button type="button" @click="clearForm" class="ghost">{{ t('common.clear') }}</button>
      </div>
    </div>

    <p v-if="errorMsg" class="message error">{{ errorMsg }}</p>
    <p v-if="okMsg" class="message ok">{{ okMsg }}</p>

    <div class="list-header">
      <p class="list-title">
        {{ t('keys.results') }}
        <span class="hint">（{{ t('common.rows', { count: filteredRows.length }) }}）</span>
      </p>
      <input v-model="keyword" class="search-input" :placeholder="t('keys.searchInResults')" />
    </div>

    <div class="data-list">
      <article
        v-for="row in filteredRows"
        :key="row.key + row.revision"
        class="data-card interactive-card"
        :class="{ active: editingKey === row.key || selectedKey?.key === row.key }"
        @click.stop="selectKey(row); loadOne(row.key)"
      >
        <div class="data-card-head">
          <h4 class="mono">{{ shortKey(row.key) }}</h4>
          <button type="button" class="btn-sm" @click.stop="copyText(row.value)">{{ t('keys.copyContent') }}</button>
        </div>
        <p class="data-value" :title="row.value">{{ truncate(row.value) }}</p>
        <details class="tech-inline" @click.stop>
          <summary>{{ t('keys.techInfo') }}</summary>
          <div class="tech-inline-body">
            <span>{{ t('keys.fullPath') }}：<code class="mono">{{ row.key }}</code></span>
            <span>{{ t('keys.version') }}：{{ row.version }} · {{ t('keys.revision') }}：{{ row.revision }}</span>
            <span v-if="row.lease">{{ t('keys.linkedLease') }}：{{ row.lease }}</span>
          </div>
        </details>
      </article>

      <div v-if="filteredRows.length === 0" class="empty-state">
        <div class="empty-icon">◉</div>
        <p v-if="keyword">{{ t('keys.noMatch', { keyword }) }}</p>
        <p v-else>{{ t('keys.empty') }}</p>
      </div>
    </div>

    <aside class="key-detail-panel">
      <div v-if="selectedKey" class="key-detail-content">
        <header class="key-detail-head">
          <div>
            <p class="info-label">{{ t("keys.selectedKey") }}</p>
            <h3 class="mono">{{ selectedKey.key }}</h3>
          </div>
        </header>
        <div class="detail-tabs" role="tablist">
          <button :class="{ active: detailTab === 'details' }" @click="detailTab = 'details'">
            {{ t("keys.details") }}
          </button>
          <button :class="{ active: detailTab === 'history' }" @click="detailTab = 'history'; loadHistory()">
            {{ t("keys.history") }}
          </button>
          <button :class="{ active: detailTab === 'watch' }" @click="detailTab = 'watch'">
            {{ t("keys.watch") }}
          </button>
        </div>
        <section v-if="detailTab === 'details'" class="key-detail-section">
          <dl class="detail-list">
            <dt>{{ t("keys.revision") }}</dt>
            <dd>{{ selectedKey.revision }}</dd>
            <dt>{{ t("keys.version") }}</dt>
            <dd>{{ selectedKey.version }}</dd>
            <dt>{{ t("keys.linkedLease") }}</dt>
            <dd>{{ selectedKey.lease || t("common.notSet") }}</dd>
          </dl>
          <pre class="value-preview mono">{{ selectedKey.value }}</pre>
        </section>
        <section v-if="detailTab === 'history'" class="key-detail-section">
          <div class="revision-query">
            <input v-model="revisionInput" :placeholder="t('keys.revision')" />
            <button class="primary" @click="loadRevision" :disabled="historyLoading">
              {{ historyLoading ? t("common.querying") : t("common.query") }}
            </button>
          </div>
          <p v-if="historyError" class="message error">{{ historyError }}</p>
          <p v-if="historyCompacted" class="message warn">{{ t("keys.historyCompacted") }}</p>
          <article v-if="revisionItem" class="history-item">
            <strong>{{ t("keys.revision") }} {{ revisionItem.revision }}</strong>
            <pre class="value-preview mono">{{ revisionItem.value }}</pre>
          </article>
          <article v-for="item in history" :key="item.revision" class="history-item">
            <strong>{{ t("keys.revision") }} {{ item.revision }}</strong>
            <span>{{ t("keys.version") }} {{ item.version }}</span>
            <pre class="value-preview mono">{{ item.value }}</pre>
          </article>
        </section>
        <section v-if="detailTab === 'watch'" class="key-detail-section">
          <div class="watch-controls">
            <input v-model="watchPrefix" :placeholder="selectedKey?.key || prefix" />
            <button class="primary" @click="startWatch" :disabled="watchStatus === 'running'">
              {{ t("keys.startWatch") }}
            </button>
            <button class="ghost" @click="toggleWatchPause" :disabled="watchStatus === 'idle'">
              {{ watchPaused ? t("keys.resumeWatch") : t("keys.pauseWatch") }}
            </button>
            <button class="ghost" @click="stopWatch" :disabled="watchStatus === 'idle'">
              {{ t("keys.stopWatch") }}
            </button>
            <button class="ghost" @click="clearWatchEvents">{{ t("common.clear") }}</button>
          </div>
          <div class="watch-filters">
            <button :class="{ active: watchFilter === 'all' }" @click="watchFilter = 'all'">
              {{ t("common.all") }}
            </button>
            <button :class="{ active: watchFilter === 'put' }" @click="watchFilter = 'put'">PUT</button>
            <button :class="{ active: watchFilter === 'delete' }" @click="watchFilter = 'delete'">DELETE</button>
          </div>
          <p class="hint">{{ t("keys.watchStatus") }}: {{ watchStatus }}</p>
          <article
            v-for="event in visibleWatchEvents"
            :key="`${event.revision}-${event.key}-${event.event_type}`"
            class="watch-event"
          >
            <header>
              <strong>{{ event.event_type.toUpperCase() }}</strong>
              <span>{{ t("keys.revision") }} {{ event.revision }}</span>
            </header>
            <p class="mono">{{ event.key }}</p>
            <pre v-if="event.value" class="value-preview mono">{{ event.value }}</pre>
          </article>
        </section>
      </div>
      <div v-else class="detail-placeholder">{{ t("keys.selectKeyHint") }}</div>
    </aside>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";
import { useAuthStore } from "../stores/auth";
import ValueEditor from "../components/ValueEditor.vue";
import { looksLikeJson, formatJson } from "../utils/json";

interface ClusterInfo {
  id: string;
  name: string;
}

interface KvItem {
  key: string;
  value: string;
  revision: number;
  version: number;
  create_revision: number;
  mod_revision: number;
  lease?: number | null;
}

type KeyDetailTab = "details" | "history" | "watch";

interface KvHistoryResponse {
  key: string;
  compacted: boolean;
  items: KvItem[];
}

interface WatchEventItem {
  event_type: "put" | "delete" | "error";
  key: string;
  value?: string;
  revision: number;
  lease?: number;
}

const { t } = useI18n();
const clusters = ref<ClusterInfo[]>([]);
const clusterId = ref("");
const prefix = ref("/services/");
const keyword = ref("");
const rows = ref<KvItem[]>([]);
const editingKey = ref("");
const editingValue = ref("");
const errorMsg = ref("");
const okMsg = ref("");
const loadingList = ref(false);
const loadingOne = ref(false);
const saving = ref(false);
const deleting = ref(false);
const valueEditorRef = ref<InstanceType<typeof ValueEditor> | null>(null);
const selectedKey = ref<KvItem | null>(null);
const detailTab = ref<KeyDetailTab>("details");
const history = ref<KvItem[]>([]);
const historyCompacted = ref(false);
const historyLoading = ref(false);
const historyError = ref("");
const revisionInput = ref("");
const revisionItem = ref<KvItem | null>(null);
const watchSource = ref<EventSource | null>(null);
const watchEvents = ref<WatchEventItem[]>([]);
const watchPrefix = ref("");
const watchStatus = ref<"idle" | "running" | "paused" | "error">("idle");
const watchFilter = ref<"all" | "put" | "delete">("all");
const watchPaused = ref(false);

const auth = useAuthStore();
const canRead = computed(() => auth.permissions.includes("key:read"));
const canWrite = computed(() => auth.permissions.includes("key:write"));
const canDelete = computed(() => auth.permissions.includes("key:delete"));

const filteredRows = computed(() => {
  const q = keyword.value.trim().toLowerCase();
  if (!q) return rows.value;
  return rows.value.filter(
    (row) => row.key.toLowerCase().includes(q) || String(row.value).toLowerCase().includes(q),
  );
});

const visibleWatchEvents = computed(() => {
  if (watchFilter.value === "all") return watchEvents.value;
  return watchEvents.value.filter((item) => item.event_type === watchFilter.value);
});

const shortKey = (key: string) => {
  const parts = key.split("/").filter(Boolean);
  if (parts.length <= 2) return key;
  return `…/${parts.slice(-2).join("/")}`;
};

const truncate = (val: string, max = 80) => {
  if (!val) return t("common.empty");
  return val.length > max ? `${val.slice(0, max)}…` : val;
};

const copyText = async (text: string) => {
  const copiedMsg = t("keys.copied");
  try {
    await navigator.clipboard.writeText(text || "");
    okMsg.value = copiedMsg;
    errorMsg.value = "";
    setTimeout(() => {
      if (okMsg.value === copiedMsg) okMsg.value = "";
    }, 2000);
  } catch {
    errorMsg.value = t("keys.copyFailed");
  }
};

const clearNotice = () => {
  errorMsg.value = "";
  okMsg.value = "";
};

const loadClusters = async () => {
  try {
    clusters.value = (await api.get<ClusterInfo[]>("/clusters")).data || [];
    if (clusters.value.length && !clusterId.value) {
      clusterId.value = clusters.value[0].id;
    }
  } catch {
    clusters.value = [];
  }
};

const queryKeys = async () => {
  clearNotice();
  loadingList.value = true;
  try {
    rows.value =
      (await api.get(`/clusters/${clusterId.value}/kv`, { params: { prefix: prefix.value } })).data || [];
    selectedKey.value = null;
    history.value = [];
    revisionItem.value = null;
  } catch (err: any) {
    rows.value = [];
    errorMsg.value = err.message || t("keys.queryFailed");
  } finally {
    loadingList.value = false;
  }
};

const selectKey = (row: KvItem) => {
  selectedKey.value = row;
  detailTab.value = "details";
  revisionInput.value = String(row.revision);
  revisionItem.value = null;
  history.value = [];
  historyCompacted.value = false;
  historyError.value = "";
};

const loadOne = async (key: string) => {
  if (!canRead.value) {
    errorMsg.value = t("keys.noReadPerm");
    return;
  }
  clearNotice();
  loadingOne.value = true;
  try {
    const item = await api.get(`/clusters/${clusterId.value}/kv/item`, { params: { key } });
    if (item.data) {
      selectedKey.value = item.data;
      revisionInput.value = String(item.data.revision);
      const loadedKey = item.data.key || key;
      const isNewKey = editingKey.value !== loadedKey;
      editingKey.value = loadedKey;
      const raw = item.data.value || "";
      if (looksLikeJson(raw)) {
        const formatted = formatJson(raw);
        editingValue.value = formatted.ok ? formatted.value : raw;
      } else {
        editingValue.value = raw;
      }
      if (isNewKey) {
        valueEditorRef.value?.prepareForContent();
      }
      okMsg.value = t("keys.loaded");
    } else {
      errorMsg.value = t("keys.notFound");
    }
  } catch (err: any) {
    errorMsg.value = err.message || t("common.loadFailed");
  } finally {
    loadingOne.value = false;
  }
};

const loadHistory = async () => {
  if (!clusterId.value || !selectedKey.value) return;
  historyLoading.value = true;
  historyError.value = "";
  try {
    const resp = await api.get<KvHistoryResponse>(`/clusters/${clusterId.value}/kv/history`, {
      params: { key: selectedKey.value.key, limit: 20 },
    });
    history.value = resp.data.items || [];
    historyCompacted.value = resp.data.compacted;
  } catch (err: any) {
    historyError.value = err.message || t("keys.historyLoadFailed");
  } finally {
    historyLoading.value = false;
  }
};

const loadRevision = async () => {
  if (!clusterId.value || !selectedKey.value) return;
  const revision = Number(revisionInput.value);
  if (!Number.isFinite(revision) || revision <= 0) {
    historyError.value = t("keys.invalidRevision");
    return;
  }
  historyLoading.value = true;
  historyError.value = "";
  try {
    const resp = await api.get<KvItem | null>(`/clusters/${clusterId.value}/kv/item`, {
      params: { key: selectedKey.value.key, revision },
    });
    revisionItem.value = resp.data;
  } catch (err: any) {
    historyError.value = err.message || t("keys.historyLoadFailed");
  } finally {
    historyLoading.value = false;
  }
};

const appendWatchEvent = (event: WatchEventItem) => {
  if (watchPaused.value) return;
  watchEvents.value = [event, ...watchEvents.value].slice(0, 500);
};

const startWatch = () => {
  if (!clusterId.value) return;
  stopWatch();
  const prefixToWatch = watchPrefix.value.trim() || selectedKey.value?.key || prefix.value;
  if (!prefixToWatch) return;
  watchStatus.value = "running";
  watchPaused.value = false;
  const params = new URLSearchParams({ prefix: prefixToWatch });
  const source = new EventSource(`/api/clusters/${clusterId.value}/kv/watch?${params.toString()}`);
  source.addEventListener("put", (event) =>
    appendWatchEvent(JSON.parse((event as MessageEvent).data)),
  );
  source.addEventListener("delete", (event) =>
    appendWatchEvent(JSON.parse((event as MessageEvent).data)),
  );
  source.addEventListener("error", () => {
    watchStatus.value = "error";
  });
  watchSource.value = source;
};

const stopWatch = () => {
  watchSource.value?.close();
  watchSource.value = null;
  watchStatus.value = "idle";
  watchPaused.value = false;
};

const toggleWatchPause = () => {
  watchPaused.value = !watchPaused.value;
  watchStatus.value = watchPaused.value ? "paused" : "running";
};

const clearWatchEvents = () => {
  watchEvents.value = [];
};

const queryByKey = async () => {
  if (!editingKey.value.trim()) {
    errorMsg.value = t("keys.needPath");
    return;
  }
  await loadOne(editingKey.value.trim());
};

const saveItem = async () => {
  clearNotice();
  if (!editingKey.value.trim()) {
    errorMsg.value = t("keys.needPath");
    return;
  }
  if (!canWrite.value) {
    errorMsg.value = t("keys.noWritePerm");
    return;
  }
  const check = valueEditorRef.value?.validate();
  if (check && !check.valid) {
    errorMsg.value = t("keys.jsonInvalidOnSave");
    return;
  }
  saving.value = true;
  try {
    await api.put(`/clusters/${clusterId.value}/kv/item`, {
      key: editingKey.value.trim(),
      value: editingValue.value,
      lease: null,
    });
    okMsg.value = t("keys.saved");
    await queryKeys();
  } catch (err: any) {
    errorMsg.value = err.message || t("keys.saveFailed");
  } finally {
    saving.value = false;
  }
};

const deleteItem = async () => {
  clearNotice();
  if (!canDelete.value) {
    errorMsg.value = t("keys.noDeletePerm");
    return;
  }
  if (!editingKey.value.trim()) {
    errorMsg.value = t("keys.needPath");
    return;
  }
  const confirmed = window.confirm(t("keys.deleteConfirm", { key: editingKey.value }));
  if (!confirmed) return;

  deleting.value = true;
  try {
    await api.delete(`/clusters/${clusterId.value}/kv/item`, {
      params: { key: editingKey.value.trim() },
    });
    okMsg.value = t("keys.deleted");
    if (editingKey.value.startsWith(prefix.value)) {
      await queryKeys();
    }
    clearForm();
  } catch (err: any) {
    errorMsg.value = err.message || t("keys.deleteFailed");
  } finally {
    deleting.value = false;
  }
};

const clearForm = () => {
  editingKey.value = "";
  editingValue.value = "";
  selectedKey.value = null;
  history.value = [];
  revisionItem.value = null;
  valueEditorRef.value?.resetMode();
};

onMounted(loadClusters);
onUnmounted(() => {
  stopWatch();
});
</script>

<style scoped>
.data-list {
  display: grid;
  gap: 10px;
}

.data-card {
  cursor: pointer;
}

.data-card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.data-card-head h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 500;
  word-break: break-all;
}

.data-value {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
  line-height: 1.5;
  word-break: break-all;
}

.key-detail-panel {
  margin-top: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
  padding: 16px;
}

.key-detail-content {
  display: grid;
  gap: 14px;
}

.key-detail-head h3 {
  margin: 4px 0 0;
  font-size: 15px;
  word-break: break-all;
}

.detail-tabs {
  display: inline-flex;
  gap: 4px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  width: fit-content;
}

.detail-tabs button {
  min-height: 32px;
  padding: 5px 12px;
  background: transparent;
  color: var(--muted);
}

.detail-tabs button.active {
  color: var(--primary);
  background: var(--primary-dim);
}

.key-detail-section {
  display: grid;
  gap: 12px;
}

.revision-query {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.value-preview {
  max-height: 240px;
  overflow: auto;
  margin: 0;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--code-text);
  background: rgba(0, 0, 0, 0.24);
  white-space: pre-wrap;
  word-break: break-word;
}

.history-item {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.watch-controls,
.watch-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.watch-controls input {
  min-width: min(280px, 100%);
  flex: 1;
}

.watch-filters button.active {
  color: var(--primary);
  background: var(--primary-dim);
}

.watch-event {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
}

.watch-event header {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  color: var(--muted);
  font-size: 12px;
}

.watch-event p {
  margin: 0;
  word-break: break-all;
}

.detail-placeholder {
  color: var(--muted);
  text-align: center;
  padding: 20px;
}
</style>
