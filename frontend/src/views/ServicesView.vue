<template>
  <section class="panel services-view">
    <div class="section-head">
      <div>
        <h2>{{ t('services.title') }}</h2>
        <p class="hint section-desc">{{ t('services.desc') }}</p>
      </div>
      <button class="primary btn-icon" @click="loadServices" :disabled="loading">
        <span v-if="loading" class="spinner"></span>
        {{ loading ? t('common.refreshing') : t('common.refresh') }}
      </button>
    </div>

    <div class="filter-bar">
      <label class="filter-item">
        <span>{{ t('services.env') }}</span>
        <select v-model="selectedClusterId" @change="loadServices">
          <option value="">{{ t('services.allEnv') }}</option>
          <option v-for="item in clusters" :key="item.id" :value="item.id">
            {{ item.name }}
          </option>
        </select>
      </label>
      <label class="filter-item grow">
        <span>{{ t('services.searchService') }}</span>
        <input v-model="keyword" :placeholder="t('services.searchPh')" />
      </label>
      <button type="button" class="ghost btn-sm advanced-toggle" @click="showAdvanced = !showAdvanced">
        {{ showAdvanced ? t('services.collapseAdvanced') : t('services.advanced') }}
      </button>
    </div>

    <div v-if="showAdvanced" class="toolbar advanced-panel">
      <div class="prefix-block grow">
        <span class="prefix-label">{{ t('services.pathPrefix') }}</span>
        <PathPrefixInput v-model="prefixInput" />
      </div>
      <div class="prefix-apply">
        <button type="button" class="primary" @click="loadServices" :disabled="loading">
          {{ t('common.apply') }}
        </button>
      </div>
    </div>

    <p v-if="errorMessage" class="message error">{{ errorMessage }}</p>

    <div class="summary">
      <article class="card stat-card">
        <h3>{{ t('services.instances') }}</h3>
        <p class="stat-value">{{ filteredServices.length }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('services.types') }}</h3>
        <p class="stat-value">{{ uniqueServiceNames }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('services.reachable') }}</h3>
        <p class="stat-value">{{ reachableCount }}</p>
      </article>
    </div>

    <div class="service-list">
      <article v-for="item in filteredServices" :key="item.key" class="service-card">
        <header class="service-header">
          <div class="service-title">
            <span class="status-dot" :class="item.view.status"></span>
            <div>
              <h3>{{ item.view.displayName }}</h3>
              <p class="service-subtitle">
                {{ item.view.namespace ? t('services.namespaceEnv', { ns: item.view.namespace }) : item.cluster_name }}
                <span v-if="item.view.version" class="version-tag">{{ item.view.version }}</span>
              </p>
            </div>
          </div>
          <span class="status-badge" :class="item.view.status">
            {{ statusText(item.view.status) }}
          </span>
        </header>

        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">{{ t('services.env') }}</span>
            <span class="info-value">{{ item.cluster_name }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">{{ t('services.instanceId') }}</span>
            <span class="info-value mono instance-id">{{ item.view.instanceId }}</span>
          </div>
        </div>

        <div class="address-section">
          <span class="info-label">{{ t('services.address') }}</span>
          <div v-if="item.view.endpoints.length" class="endpoint-list">
            <a
              v-for="ep in item.view.endpoints"
              :key="ep"
              class="endpoint-link"
              :href="ep"
              target="_blank"
              rel="noopener noreferrer"
            >
              {{ friendlyEndpoint(ep) }}
            </a>
          </div>
          <p v-else class="no-address">{{ t('services.noAddress') }}</p>
        </div>

        <details class="tech-details">
          <summary>{{ t('services.techDetails') }}</summary>
          <div class="tech-body">
            <div class="tech-row">
              <span>{{ t('services.storagePath') }}</span>
              <code class="mono">{{ item.key }}</code>
            </div>
            <div v-if="item.view.appConfig" class="tech-row">
              <span>{{ t('services.configPath') }}</span>
              <code class="mono">{{ item.view.appConfig }}</code>
            </div>
            <div v-if="item.view.extraMeta.length" class="tech-meta">
              <p class="tech-meta-title">{{ t('services.otherMeta') }}</p>
              <ul>
                <li v-for="entry in item.view.extraMeta" :key="entry.key">
                  <span>{{ entry.label }}</span>
                  <code class="mono">{{ entry.value }}</code>
                </li>
              </ul>
            </div>
          </div>
        </details>
      </article>
    </div>

    <div v-if="!loading && filteredServices.length === 0" class="empty-state">
      <div class="empty-icon">▣</div>
      <p v-if="keyword">{{ t('services.noMatch', { keyword }) }}</p>
      <p v-else>{{ t('services.empty') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";
import PathPrefixInput from "../components/PathPrefixInput.vue";
import { useAuthStore } from "../stores/auth";

interface ServiceItem {
  cluster_id: string;
  cluster_name: string;
  key: string;
  service_name: string;
  service_id: string;
  value: string;
  metadata?: Record<string, unknown>;
  address?: string;
}

interface ClusterInfo {
  id: string;
  name: string;
}

interface ServiceView {
  displayName: string;
  version: string;
  namespace: string;
  instanceId: string;
  endpoints: string[];
  status: "online" | "offline";
  appConfig: string;
  extraMeta: { key: string; label: string; value: string }[];
}

interface EnrichedService extends ServiceItem {
  view: ServiceView;
}

const { t } = useI18n();
const auth = useAuthStore();
const clusters = ref<ClusterInfo[]>([]);
const services = ref<ServiceItem[]>([]);
const selectedClusterId = ref("");
const prefixInput = ref("/services/");
const keyword = ref("");
const showAdvanced = ref(false);
const loading = ref(false);
const errorMessage = ref("");

const canRead = computed(() => auth.permissions.includes("cluster:read"));

const metaLabel = (key: string) => {
  const known = ["id", "name", "version", "endpoints", "metadata"] as const;
  if (known.includes(key as (typeof known)[number])) {
    return t(`services.meta.${key}`);
  }
  return key;
};

const parsePayload = (raw: string): Record<string, unknown> => {
  try {
    return JSON.parse(raw) as Record<string, unknown>;
  } catch {
    return {};
  }
};

const asStringArray = (value: unknown): string[] => {
  if (!Array.isArray(value)) return [];
  return value.map((item) => String(item)).filter(Boolean);
};

const buildView = (item: ServiceItem): ServiceView => {
  const payload = parsePayload(item.value);
  const metaObj = (payload.metadata as Record<string, unknown>) || {};
  const keyParts = item.key.split("/").filter(Boolean);
  const namespace = keyParts[0] === "services" ? keyParts[1] || "" : keyParts[0] || "";

  const displayName =
    (typeof payload.name === "string" && payload.name) ||
    (item.service_name !== "unknown" ? item.service_name : "") ||
    t("services.unknownService");

  const version = typeof payload.version === "string" ? payload.version : "";
  const instanceId =
    (typeof payload.id === "string" && payload.id) ||
    item.service_id ||
    keyParts[keyParts.length - 1] ||
    "";

  const endpoints = [
    ...new Set([
      ...(item.address ? [item.address] : []),
      ...asStringArray(payload.endpoints),
      ...asStringArray(metaObj.endpoints),
    ]),
  ];

  const appConfig =
    typeof metaObj["app-config"] === "string"
      ? metaObj["app-config"]
      : typeof metaObj.app_config === "string"
        ? metaObj.app_config
        : "";

  const hiddenKeys = new Set(["name", "version", "id", "address", "host", "port", "endpoints"]);
  const extraMeta = Object.entries({ ...payload, ...metaObj })
    .filter(([key, value]) => !hiddenKeys.has(key) && value !== undefined && value !== null)
    .map(([key, value]) => ({
      key,
      label: metaLabel(key),
      value:
        typeof value === "string" || typeof value === "number" || typeof value === "boolean"
          ? String(value)
          : JSON.stringify(value),
    }));

  return {
    displayName,
    version,
    namespace,
    instanceId,
    endpoints,
    status: endpoints.length > 0 ? "online" : "offline",
    appConfig,
    extraMeta,
  };
};

const enrichedServices = computed<EnrichedService[]>(() =>
  services.value.map((item) => ({ ...item, view: buildView(item) })),
);

const filteredServices = computed(() => {
  const q = keyword.value.trim().toLowerCase();
  if (!q) return enrichedServices.value;
  return enrichedServices.value.filter((item) => {
    const haystack = [
      item.view.displayName,
      item.view.namespace,
      item.cluster_name,
      item.view.instanceId,
      ...item.view.endpoints,
    ]
      .join(" ")
      .toLowerCase();
    return haystack.includes(q);
  });
});

const uniqueServiceNames = computed(
  () => new Set(filteredServices.value.map((s) => s.view.displayName)).size,
);
const reachableCount = computed(
  () => filteredServices.value.filter((s) => s.view.endpoints.length > 0).length,
);

const statusText = (status: "online" | "offline") =>
  status === "online" ? t("common.online") : t("common.offline");

const friendlyEndpoint = (url: string) => {
  try {
    const parsed = new URL(url);
    const proto = parsed.protocol.replace(":", "").toUpperCase();
    return `${proto} · ${parsed.host}`;
  } catch {
    return url;
  }
};

const loadClusters = async () => {
  try {
    clusters.value = (await api.get<ClusterInfo[]>("/clusters")).data || [];
  } catch {
    clusters.value = [];
  }
};

const loadServices = async () => {
  errorMessage.value = "";
  if (!canRead.value) {
    services.value = [];
    errorMessage.value = t("services.noPerm");
    return;
  }
  loading.value = true;
  try {
    const params: Record<string, string> = { prefix: prefixInput.value.trim() };
    if (selectedClusterId.value) {
      params.cluster_id = selectedClusterId.value;
    }
    services.value = (await api.get<ServiceItem[]>("/services", { params })).data || [];
  } catch (err: any) {
    services.value = [];
    errorMessage.value = err.message || t("services.loadFailed");
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  await loadClusters();
  if (canRead.value) {
    await loadServices();
  }
});
</script>

<style scoped>
.services-view {
  display: grid;
  gap: 16px;
}

.services-view .filter-bar {
  margin-bottom: 0;
}

.advanced-toggle {
  align-self: flex-end;
}

.advanced-panel {
  margin-bottom: 0;
  align-items: flex-end;
  gap: 12px;
}

.prefix-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.prefix-label {
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.prefix-apply {
  display: flex;
  align-items: flex-end;
  flex-shrink: 0;
  padding-bottom: 2px;
}

.service-list {
  display: grid;
  gap: 14px;
}

.service-card {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  background: rgba(0, 0, 0, 0.2);
  transition: border-color 0.2s;
}

.service-card:hover {
  border-color: var(--border-strong);
}

.service-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
}

.service-title {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.service-title h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
}

.service-subtitle {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--muted);
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.version-tag {
  padding: 1px 8px;
  border-radius: 4px;
  background: var(--primary-dim);
  color: var(--primary);
  font-size: 11px;
  font-weight: 500;
}

.status-dot {
  margin-top: 6px;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.status-badge.online {
  color: var(--success);
  background: var(--success-dim);
  border: 1px solid rgba(74, 222, 128, 0.3);
}

.status-badge.offline {
  color: var(--muted);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 11px;
  color: var(--muted);
  font-weight: 600;
  letter-spacing: 0.04em;
}

.info-value {
  font-size: 14px;
  color: var(--text);
}

.instance-id {
  font-size: 12px;
  word-break: break-all;
  line-height: 1.5;
}

.address-section {
  padding: 12px 14px;
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid var(--border);
}

.address-section .info-label {
  display: block;
  margin-bottom: 8px;
}

.endpoint-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.endpoint-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background: var(--primary-dim);
  border: 1px solid var(--border-strong);
  color: var(--primary);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  transition: background 0.2s;
  width: fit-content;
}

.endpoint-link:hover {
  background: rgba(34, 211, 238, 0.2);
}

.no-address {
  margin: 0;
  font-size: 13px;
  color: var(--muted);
}

.tech-details {
  margin-top: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.tech-details summary {
  padding: 10px 14px;
  cursor: pointer;
  font-size: 12px;
  color: var(--muted);
  background: rgba(0, 0, 0, 0.15);
  user-select: none;
  list-style: none;
}

.tech-details summary:hover {
  color: var(--text);
}

.tech-body {
  padding: 12px 14px;
  display: grid;
  gap: 10px;
}

.tech-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.tech-row > span:first-child {
  color: var(--muted);
  font-size: 11px;
}

.tech-row code {
  font-size: 11px;
  word-break: break-all;
  color: #a5f3fc;
}

.tech-meta-title {
  margin: 0 0 6px;
  font-size: 11px;
  color: var(--muted);
}

.tech-meta ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

.tech-meta li {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  padding: 5px 0;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}

.tech-meta li:last-child {
  border-bottom: none;
}

.tech-meta li span {
  color: var(--muted);
  flex-shrink: 0;
}

.tech-meta code {
  text-align: right;
  word-break: break-all;
  color: #a5f3fc;
  font-size: 11px;
}
</style>
