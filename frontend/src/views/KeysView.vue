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
        :class="{ active: editingKey === row.key }"
        @click.stop="loadOne(row.key)"
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
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";
import { useAuthStore } from "../stores/auth";
import ValueEditor from "../components/ValueEditor.vue";
import { looksLikeJson, formatJson } from "../utils/json";

interface ClusterInfo {
  id: string;
  name: string;
}

const { t } = useI18n();
const clusters = ref<ClusterInfo[]>([]);
const clusterId = ref("");
const prefix = ref("/services/");
const keyword = ref("");
const rows = ref<any[]>([]);
const editingKey = ref("");
const editingValue = ref("");
const errorMsg = ref("");
const okMsg = ref("");
const loadingList = ref(false);
const loadingOne = ref(false);
const saving = ref(false);
const deleting = ref(false);
const valueEditorRef = ref<InstanceType<typeof ValueEditor> | null>(null);

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
  } catch (err: any) {
    rows.value = [];
    errorMsg.value = err.message || t("keys.queryFailed");
  } finally {
    loadingList.value = false;
  }
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
  valueEditorRef.value?.resetMode();
};

onMounted(loadClusters);
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
</style>
