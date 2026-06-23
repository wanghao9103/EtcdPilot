<template>
  <section class="panel">
    <div class="section-head">
      <div>
        <h2>{{ t('audits.title') }}</h2>
        <p class="hint section-desc">{{ t('audits.desc') }}</p>
      </div>
      <button type="button" @click="load" :disabled="loading || !canRead" class="ghost btn-icon">
        <span v-if="loading" class="spinner"></span>
        {{ loading ? t('common.refreshing') : t('common.refresh') }}
      </button>
    </div>

    <p v-if="!canRead" class="message error">{{ t('audits.noPerm') }}</p>
    <p v-if="error" class="message error">{{ error }}</p>

    <div v-if="canRead" class="filter-bar">
      <label class="filter-item grow">
        <span>{{ t('common.search') }}</span>
        <input v-model="keyword" :placeholder="t('audits.searchPh')" />
      </label>
      <label class="filter-item">
        <span>{{ t('audits.resultFilter') }}</span>
        <select v-model="resultFilter">
          <option value="">{{ t('common.all') }}</option>
          <option value="success">{{ t('audits.successOnly') }}</option>
          <option value="failed">{{ t('audits.failedOnly') }}</option>
        </select>
      </label>
    </div>

    <div class="audit-list" v-if="canRead">
      <article v-for="row in filteredRows" :key="row.id" class="audit-card interactive-card">
        <div class="audit-head">
          <div class="audit-main">
            <span class="badge" :class="row.success ? 'ok' : 'error'">
              {{ row.success ? t('common.success') : t('common.failed') }}
            </span>
            <strong>{{ operationLabel(row.operation) }}</strong>
            <span class="hint">· {{ row.username }}</span>
          </div>
          <time class="audit-time">{{ formatTime(row.created_at) }}</time>
        </div>
        <p class="audit-desc">{{ describeAction(row) }}</p>
        <details class="tech-inline">
          <summary>{{ t('audits.techDetails') }}</summary>
          <div class="tech-inline-body">
            <span>{{ t('audits.env') }}：{{ clusterName(row.cluster_id) }}</span>
            <span>{{ t('audits.resourceType') }}：{{ row.resource_type }}</span>
            <span v-if="row.resource_key">{{ t('audits.path') }}：<code class="mono">{{ row.resource_key }}</code></span>
            <span v-if="row.error_message || row.request_summary">
              {{ t('audits.detail') }}：{{ row.error_message || row.request_summary }}
            </span>
            <span class="hint">{{ t('audits.userId') }}：{{ row.user_id }}</span>
          </div>
        </details>
      </article>

      <div v-if="filteredRows.length === 0 && !loading" class="empty-state">
        <div class="empty-icon">📋</div>
        <p v-if="keyword || resultFilter">{{ t('audits.noMatch') }}</p>
        <p v-else>{{ t('audits.empty') }}</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";
import { useAuthStore } from "../stores/auth";

interface ClusterInfo {
  id: string;
  name: string;
}

const { t, locale } = useI18n();
const auth = useAuthStore();
const rows = ref<any[]>([]);
const clusters = ref<ClusterInfo[]>([]);
const keyword = ref("");
const resultFilter = ref("");
const error = ref("");
const loading = ref(false);
const canRead = computed(() => auth.permissions.includes("audit:read"));

const OPERATION_KEYS = ["login", "logout", "read", "write", "delete", "create", "update", "test"] as const;

const operationLabel = (op: string) => {
  if (OPERATION_KEYS.includes(op as (typeof OPERATION_KEYS)[number])) {
    return t(`audits.ops.${op}`);
  }
  return op;
};

const clusterName = (id: string) => {
  const found = clusters.value.find((c) => c.id === id);
  return found ? found.name : id;
};

const describeAction = (row: any) => {
  const env = clusterName(row.cluster_id);
  const target = row.resource_key ? `「${shortPath(row.resource_key)}」` : row.resource_type;
  const action = operationLabel(row.operation);
  if (row.success) {
    return t("audits.describeSuccess", { user: row.username, env, action, target });
  }
  return t("audits.describeFailed", { user: row.username, env, action, target });
};

const shortPath = (path: string) => {
  const parts = path.split("/").filter(Boolean);
  if (parts.length <= 2) return path;
  return `…/${parts.slice(-2).join("/")}`;
};

const filteredRows = computed(() => {
  let list = rows.value;
  if (resultFilter.value === "success") list = list.filter((r) => r.success);
  if (resultFilter.value === "failed") list = list.filter((r) => !r.success);

  const q = keyword.value.trim().toLowerCase();
  if (!q) return list;
  return list.filter((row) =>
    [row.username, row.operation, row.resource_type, row.resource_key, row.cluster_id]
      .join(" ")
      .toLowerCase()
      .includes(q),
  );
});

const formatTime = (value: number) => {
  const ts = Number(value || 0) * 1000;
  return new Date(ts).toLocaleString(locale.value, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

const loadClusters = async () => {
  try {
    clusters.value = (await api.get<ClusterInfo[]>("/clusters")).data || [];
  } catch {
    clusters.value = [];
  }
};

const load = async () => {
  error.value = "";
  if (!canRead.value) {
    rows.value = [];
    return;
  }
  loading.value = true;
  try {
    rows.value = (await api.get("/audits")).data || [];
  } catch (err: any) {
    rows.value = [];
    error.value = err.message || t("audits.loadFailed");
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  await loadClusters();
  await load();
});
</script>

<style scoped>
.audit-list {
  display: grid;
  gap: 10px;
}

.audit-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.audit-main {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 14px;
}

.audit-time {
  font-size: 12px;
  color: var(--muted);
  white-space: nowrap;
}

.audit-desc {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
  line-height: 1.5;
}
</style>
