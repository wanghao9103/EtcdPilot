<template>
  <section class="panel">
    <div class="section-head">
      <div>
        <h2>{{ t('clusters.title') }}</h2>
        <p class="hint section-desc">{{ t('clusters.desc') }}</p>
      </div>
      <button class="ghost btn-icon" type="button" @click="loadClusters" :disabled="loading">
        <span v-if="loading" class="spinner"></span>
        {{ loading ? t('common.refreshing') : t('common.refresh') }}
      </button>
    </div>

    <p v-if="error" class="message error">{{ error }}</p>

    <div class="summary" v-if="clusters.length">
      <article class="card stat-card">
        <h3>{{ t('clusters.total') }}</h3>
        <p class="stat-value">{{ clusters.length }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('clusters.healthy') }}</h3>
        <p class="stat-value">{{ healthyCount }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('clusters.nodeTotal') }}</h3>
        <p class="stat-value">{{ totalMembers }}</p>
      </article>
    </div>

    <div class="cluster-list">
      <article v-for="cluster in clusters" :key="cluster.id" class="card cluster-card">
        <div class="cluster-row">
          <div class="cluster-main">
            <div class="cluster-title-row">
              <span class="status-dot" :class="connectionClass(cluster.id)"></span>
              <h3>{{ cluster.name }}</h3>
              <span class="badge" :class="cluster.readonly ? 'warn' : 'ok'">
                {{ cluster.readonly ? t('common.readonly') : t('common.writable') }}
              </span>
            </div>
            <p class="cluster-desc">
              {{ sourceLabel(cluster.source) }}
              <span v-if="clusterMembers[cluster.id]?.length">
                · {{ t('common.nodes', { count: clusterMembers[cluster.id].length }) }}
              </span>
            </p>
          </div>
          <div class="actions">
            <button
              type="button"
              class="primary btn-sm"
              :disabled="isBusy(cluster.id, 'test')"
              @click="test(cluster.id)"
            >
              {{ isBusy(cluster.id, "test") ? t('clusters.testing') : t('clusters.testConn') }}
            </button>
            <button
              type="button"
              class="btn-sm"
              :disabled="isBusy(cluster.id, 'members')"
              @click="loadMembers(cluster.id)"
            >
              {{ isBusy(cluster.id, "members") ? t('common.loading') : t('clusters.viewNodes') }}
            </button>
          </div>
        </div>

        <div v-if="clusterTestResult[cluster.id]" class="test-result">
          <span class="badge" :class="isConnected(cluster.id) ? 'ok' : 'error'">
            {{ connectionText(cluster.id) }}
          </span>
        </div>

        <div v-if="clusterMembers[cluster.id]?.length" class="member-preview">
          <p class="preview-title">{{ t('clusters.nodeOverview') }}</p>
          <div class="member-chips">
            <span v-for="member in clusterMembers[cluster.id]" :key="member.id" class="member-chip">
              {{ member.name || t('clusters.unnamedNode') }}
            </span>
          </div>
        </div>

        <details class="tech-details">
          <summary>{{ t('clusters.techDetails') }}</summary>
          <div class="tech-body">
            <div class="tech-actions">
              <button
                type="button"
                class="btn-sm"
                :disabled="isBusy(cluster.id, 'status')"
                @click="loadStatus(cluster.id)"
              >
                {{ isBusy(cluster.id, "status") ? t('common.loading') : t('clusters.loadStatus') }}
              </button>
            </div>
            <div class="tech-row">
              <span>{{ t('clusters.clusterId') }}</span>
              <code class="mono">{{ cluster.id }}</code>
            </div>
            <div v-if="endpointStatuses[cluster.id]?.length" class="endpoint-status-list">
              <article
                v-for="endpoint in endpointStatuses[cluster.id]"
                :key="endpoint.endpoint"
                class="endpoint-status-card"
                :class="{ reachable: endpoint.reachable, failed: !endpoint.reachable }"
              >
                <header>
                  <span class="status-dot" :class="endpoint.reachable ? 'online' : 'offline'"></span>
                  <strong class="mono">{{ endpoint.endpoint }}</strong>
                </header>
                <div class="endpoint-metrics">
                  <span>{{ t("clusters.version") }}: {{ endpoint.version || t("common.notSet") }}</span>
                  <span>{{ t("clusters.leader") }}: {{ endpoint.leader ?? t("common.notSet") }}</span>
                  <span>{{ t("clusters.raftIndex") }}: {{ endpoint.raft_index ?? t("common.notSet") }}</span>
                  <span>{{ t("clusters.dbSize") }}: {{ formatBytes(endpoint.db_size) }}</span>
                </div>
                <p v-if="endpoint.error" class="endpoint-error">{{ endpoint.error }}</p>
              </article>
            </div>
            <div v-if="clusterStatus[cluster.id]" class="tech-row">
              <span>{{ t('clusters.statusData') }}</span>
              <pre class="pre-block">{{ formatStatus(clusterStatus[cluster.id]) }}</pre>
            </div>
            <div v-if="clusterMembers[cluster.id]?.length">
              <p class="tech-meta-title">{{ t('clusters.nodeDetails') }}</p>
              <ul class="tech-members">
                <li v-for="member in clusterMembers[cluster.id]" :key="member.id">
                  <strong>{{ member.name || t('clusters.unnamed') }}</strong>
                  <code class="mono">{{ member.id }}</code>
                  <span class="hint">Peer: {{ member.peer_urls?.join(", ") || "-" }}</span>
                  <span class="hint">Client: {{ member.client_urls?.join(", ") || "-" }}</span>
                </li>
              </ul>
            </div>
          </div>
        </details>
      </article>

      <div v-if="!loading && clusters.length === 0" class="empty-state">
        <div class="empty-icon">⬡</div>
        <p>{{ t('clusters.empty') }}</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";

interface EndpointStatusItem {
  endpoint: string;
  reachable: boolean;
  version?: string;
  leader?: number;
  raft_term?: number;
  raft_index?: number;
  raft_applied_index?: number;
  raft_used_db_size?: number;
  db_size?: number;
  errors: string[];
  is_learner?: boolean;
  error?: string;
}

interface EndpointStatusResponse {
  cluster_id: string;
  endpoints: EndpointStatusItem[];
}

const { t } = useI18n();
const clusters = ref<any[]>([]);
const clusterStatus = ref<Record<string, any>>({});
const endpointStatuses = ref<Record<string, EndpointStatusItem[]>>({});
const clusterMembers = ref<Record<string, any[]>>({});
const clusterTestResult = ref<Record<string, string>>({});
const busyById = ref<Record<string, "test" | "status" | "members" | "">>({});
const loading = ref(false);
const error = ref("");

const healthyCount = computed(
  () => clusters.value.filter((c) => clusterTestResult.value[c.id]?.includes("true")).length,
);
const totalMembers = computed(() =>
  Object.values(clusterMembers.value).reduce((sum, list) => sum + (list?.length || 0), 0),
);

const sourceLabel = (source?: string) => {
  if (source === "config") return t("clusters.sourceConfig");
  if (source === "db") return t("clusters.sourceDb");
  return t("clusters.sourceSystem");
};

const isConnected = (id: string) => clusterTestResult.value[id]?.includes("true");
const connectionClass = (id: string) => {
  const result = clusterTestResult.value[id];
  if (!result) return "unknown";
  return isConnected(id) ? "online" : "offline";
};
const connectionText = (id: string) =>
  isConnected(id) ? t("common.connected") : t("common.disconnected");

const loadClusters = async () => {
  loading.value = true;
  error.value = "";
  try {
    clusters.value = (await api.get("/clusters")).data || [];
    for (const cluster of clusters.value) {
      await test(cluster.id);
    }
  } catch (e: any) {
    clusters.value = [];
    error.value = e?.message || t("clusters.loadFailed");
  } finally {
    loading.value = false;
  }
};

const isBusy = (id: string, action: "test" | "status" | "members") => busyById.value[id] === action;

const markBusy = (id: string, action: "test" | "status" | "members" | "") => {
  if (action === "") {
    delete busyById.value[id];
    return;
  }
  busyById.value[id] = action;
};

const test = async (id: string) => {
  markBusy(id, "test");
  try {
    const resp = await api.post(`/clusters/${id}/test`);
    clusterTestResult.value[id] = `connected: ${resp.data?.connected}`;
  } catch (err: any) {
    clusterTestResult.value[id] = `connected: false (${err?.message || "unknown"})`;
  } finally {
    markBusy(id, "");
  }
};

const loadStatus = async (id: string) => {
  markBusy(id, "status");
  try {
    const [statusResp, endpointResp] = await Promise.all([
      api.get(`/clusters/${id}/status`),
      api.get<EndpointStatusResponse>(`/clusters/${id}/endpoints/status`),
    ]);
    clusterStatus.value[id] = statusResp.data;
    endpointStatuses.value[id] = endpointResp.data.endpoints || [];
  } catch (err: any) {
    clusterStatus.value[id] = { error: err?.message || "failed" };
    endpointStatuses.value[id] = [];
  } finally {
    markBusy(id, "");
  }
};

const loadMembers = async (id: string) => {
  markBusy(id, "members");
  try {
    const data = await api.get(`/clusters/${id}/members`);
    clusterMembers.value[id] = data.data?.members || [];
  } catch (err: any) {
    clusterMembers.value[id] = [{ id: "error", name: err?.message || t("common.loadFailed") }];
  } finally {
    markBusy(id, "");
  }
};

const formatStatus = (payload: unknown) => {
  if (!payload) return "";
  return JSON.stringify(payload, null, 2);
};

const formatBytes = (value?: number) => {
  if (!value || value < 0) return t("common.notSet");
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB`;
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
};

onMounted(loadClusters);
</script>

<style scoped>
.cluster-list {
  display: grid;
  gap: 14px;
}

.cluster-card {
  padding: 18px;
}

.cluster-row {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.cluster-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.cluster-title-row h3 {
  margin: 0;
  font-size: 17px;
  color: var(--text);
}

.cluster-desc {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.test-result {
  margin-top: 12px;
}

.member-preview {
  margin-top: 14px;
  padding: 12px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  border: 1px solid var(--border);
}

.preview-title {
  margin: 0 0 8px;
  font-size: 11px;
  color: var(--muted);
  font-weight: 600;
  letter-spacing: 0.04em;
}

.member-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.member-chip {
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  background: var(--primary-dim);
  color: var(--primary);
  border: 1px solid var(--border);
}

.tech-actions {
  margin-bottom: 4px;
}

.endpoint-status-list {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.endpoint-status-card {
  display: grid;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.18);
}

.endpoint-status-card.reachable {
  border-color: rgba(74, 222, 128, 0.22);
}

.endpoint-status-card.failed {
  border-color: rgba(248, 113, 113, 0.22);
}

.endpoint-status-card header {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.endpoint-status-card header strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.endpoint-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 6px 12px;
  color: var(--muted);
  font-size: 12px;
}

.endpoint-error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
  line-height: 1.5;
  word-break: break-word;
}

.tech-meta-title {
  margin: 0 0 6px;
  font-size: 11px;
  color: var(--muted);
}

.tech-members {
  margin: 0;
  padding: 0;
  list-style: none;
}

.tech-members li {
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
}

.tech-members li:last-child {
  border-bottom: none;
}
</style>
