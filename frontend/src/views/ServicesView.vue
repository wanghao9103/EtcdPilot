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

    <div class="view-switch" role="tablist" :aria-label="t('services.viewMode')">
      <button
        type="button"
        class="view-switch-btn"
        :class="{ active: viewMode === 'topology' }"
        role="tab"
        :aria-selected="viewMode === 'topology'"
        @click="viewMode = 'topology'"
      >
        {{ t('services.topology') }}
      </button>
      <button
        type="button"
        class="view-switch-btn"
        :class="{ active: viewMode === 'list' }"
        role="tab"
        :aria-selected="viewMode === 'list'"
        @click="viewMode = 'list'"
      >
        {{ t('services.list') }}
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

    <div v-if="viewMode === 'topology'" class="topology-layout">
      <section class="topology-map" :aria-label="t('services.topology')">
        <div v-if="topologyClusters.length" class="topology-clusters">
          <div class="topology-tools" aria-hidden="true">
            <span>Fit</span>
            <span>+</span>
            <span>-</span>
          </div>
          <div class="topology-lanes" aria-hidden="true">
            <span>{{ t('services.clusterNode') }}</span>
            <span>{{ t('services.serviceNode') }}</span>
            <span>{{ t('services.instanceNode') }}</span>
          </div>
          <article v-for="cluster in topologyClusters" :key="cluster.id" class="topology-cluster">
            <button
              type="button"
              class="topology-node cluster-node"
              :class="{ selected: selectedTopologyNode?.id === cluster.id }"
              @click.stop="selectTopologyNode(cluster, $event)"
              @mouseenter="showTopologyTooltip(cluster, $event)"
              @mousemove="moveTopologyTooltip"
              @mouseleave="hideTopologyTooltip"
              @focus="showTopologyTooltip(cluster, $event)"
              @blur="hideTopologyTooltip"
            >
              <span class="node-icon cluster-icon" aria-hidden="true"></span>
              <span class="node-health online" aria-hidden="true"></span>
              <span class="node-body">
                <span class="node-kicker">{{ t('services.clusterNode') }}</span>
                <strong>{{ cluster.label }}</strong>
                <span>{{ t('services.serviceSummary', { services: cluster.services.length, instances: cluster.instanceCount }) }}</span>
              </span>
            </button>

            <div class="topology-services">
              <article v-for="service in cluster.services" :key="service.id" class="topology-service">
                <button
                  type="button"
                  class="topology-node service-node"
                  :class="{ selected: selectedTopologyNode?.id === service.id }"
                  @click.stop="selectTopologyNode(service, $event)"
                  @mouseenter="showTopologyTooltip(service, $event)"
                  @mousemove="moveTopologyTooltip"
                  @mouseleave="hideTopologyTooltip"
                  @focus="showTopologyTooltip(service, $event)"
                  @blur="hideTopologyTooltip"
                >
                  <span class="node-icon service-icon" aria-hidden="true"></span>
                  <span
                    class="node-health"
                    :class="service.reachableCount === service.instances.length ? 'online' : 'offline'"
                    aria-hidden="true"
                  ></span>
                  <span class="node-body">
                    <span class="node-kicker">{{ t('services.serviceNode') }}</span>
                    <strong>{{ service.label }}</strong>
                    <span>{{ t('services.instanceSummary', { count: service.instances.length, reachable: service.reachableCount }) }}</span>
                  </span>
                </button>

                <div class="topology-instances">
                  <button
                    v-for="instance in service.instances"
                    :key="instance.id"
                    type="button"
                    class="topology-node instance-node"
                    :class="[instance.status, { selected: selectedTopologyNode?.id === instance.id }]"
                    @click.stop="selectTopologyNode(instance, $event)"
                    @mouseenter="showTopologyTooltip(instance, $event)"
                    @mousemove="moveTopologyTooltip"
                    @mouseleave="hideTopologyTooltip"
                    @focus="showTopologyTooltip(instance, $event)"
                    @blur="hideTopologyTooltip"
                  >
                    <span class="node-icon instance-icon" aria-hidden="true"></span>
                    <span class="node-health" :class="instance.status" aria-hidden="true"></span>
                    <span class="node-body">
                      <span class="node-kicker">{{ t('services.instanceNode') }}</span>
                      <strong>{{ instance.label }}</strong>
                      <span>{{ instance.subtitle }}</span>
                    </span>
                  </button>
                </div>
              </article>
            </div>
          </article>
          <div class="topology-legend" aria-hidden="true">
            <span><i class="legend-dot online"></i>{{ t('common.online') }}</span>
            <span><i class="legend-dot offline"></i>{{ t('common.offline') }}</span>
          </div>
        </div>

        <div v-else-if="!loading" class="empty-state topology-empty">
          <div class="empty-icon">□</div>
          <p v-if="keyword">{{ t('services.noMatch', { keyword }) }}</p>
          <p v-else>{{ t('services.topologyEmpty') }}</p>
        </div>
      </section>

      <Teleport to="body">
        <aside
          v-if="hoveredTopologyNode"
          class="topology-detail topology-tooltip"
          :class="{ pinned: isTopologyTooltipPinned }"
          :style="topologyTooltipStyle"
          :aria-label="t('services.nodeDetails')"
          @mouseenter="keepTopologyTooltip"
          @mouseleave="hideTopologyTooltip"
          @click.stop
        >
          <div class="detail-heading">
            <span class="node-kicker">{{ topologyTypeText(hoveredTopologyNode.type) }}</span>
            <h3>{{ hoveredTopologyNode.label }}</h3>
            <span v-if="hoveredTopologyNode.status" class="status-badge" :class="hoveredTopologyNode.status">
              {{ statusText(hoveredTopologyNode.status) }}
            </span>
          </div>

          <dl class="detail-list">
            <template v-for="row in hoveredTopologyNode.details" :key="row.label">
              <dt>{{ row.label }}</dt>
              <dd :class="{ mono: row.mono }">{{ row.value }}</dd>
            </template>
          </dl>

          <div v-if="hoveredTopologyNode.endpoints?.length" class="detail-section">
            <span class="info-label">{{ t('services.address') }}</span>
            <div class="endpoint-list">
              <a
                v-for="ep in hoveredTopologyNode.endpoints"
                :key="ep"
                class="endpoint-link"
                :href="ep"
                target="_blank"
                rel="noopener noreferrer"
              >
                {{ friendlyEndpoint(ep) }}
              </a>
            </div>
          </div>
        </aside>
      </Teleport>
    </div>

    <div v-if="viewMode === 'list'" class="service-list">
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

    <div v-if="viewMode === 'list' && !loading && filteredServices.length === 0" class="empty-state">
      <div class="empty-icon">▣</div>
      <p v-if="keyword">{{ t('services.noMatch', { keyword }) }}</p>
      <p v-else>{{ t('services.empty') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
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

type ViewMode = "topology" | "list";
type TopologyNodeType = "cluster" | "service" | "instance";

interface DetailRow {
  label: string;
  value: string;
  mono?: boolean;
}

interface TopologyNode {
  id: string;
  type: TopologyNodeType;
  label: string;
  subtitle: string;
  status?: "online" | "offline";
  details: DetailRow[];
  endpoints?: string[];
}

interface TopologyInstance extends TopologyNode {
  type: "instance";
  status: "online" | "offline";
  service: EnrichedService;
}

interface TopologyService extends TopologyNode {
  type: "service";
  instances: TopologyInstance[];
  reachableCount: number;
}

interface TopologyCluster extends TopologyNode {
  type: "cluster";
  services: TopologyService[];
  instanceCount: number;
}

const { t } = useI18n();
const auth = useAuthStore();
const clusters = ref<ClusterInfo[]>([]);
const services = ref<ServiceItem[]>([]);
const selectedClusterId = ref("");
const prefixInput = ref("/services/");
const keyword = ref("");
const showAdvanced = ref(false);
const viewMode = ref<ViewMode>("topology");
const loading = ref(false);
const errorMessage = ref("");
const selectedTopologyNodeId = ref("");
const hoveredTopologyNode = ref<TopologyNode | null>(null);
const topologyTooltip = ref({ x: 0, y: 0 });
const isTopologyTooltipPinned = ref(false);
let topologyTooltipHideTimer: ReturnType<typeof window.setTimeout> | undefined;

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

const firstValue = (...values: string[]) =>
  values.find((value) => value.trim().length > 0) || t("common.notSet");

const buildInstanceNode = (item: EnrichedService): TopologyInstance => {
  const label = firstValue(item.view.instanceId, item.view.displayName);
  const subtitle = item.view.endpoints[0]
    ? friendlyEndpoint(item.view.endpoints[0])
    : t("services.noAddress");

  return {
    id: `instance:${item.cluster_id}:${item.key}`,
    type: "instance",
    label,
    subtitle,
    status: item.view.status,
    service: item,
    endpoints: item.view.endpoints,
    details: [
      { label: t("services.env"), value: item.cluster_name },
      { label: t("services.serviceNode"), value: item.view.displayName },
      { label: t("services.instanceId"), value: label, mono: true },
      { label: t("services.version"), value: firstValue(item.view.version) },
      { label: t("services.namespace"), value: firstValue(item.view.namespace) },
      { label: t("services.storagePath"), value: item.key, mono: true },
      { label: t("services.configPath"), value: firstValue(item.view.appConfig), mono: true },
    ],
  };
};

const topologyClusters = computed<TopologyCluster[]>(() => {
  const clusterMap = new Map<string, { name: string; items: EnrichedService[] }>();
  for (const item of filteredServices.value) {
    const group = clusterMap.get(item.cluster_id) || { name: item.cluster_name, items: [] };
    group.items.push(item);
    clusterMap.set(item.cluster_id, group);
  }

  return [...clusterMap.entries()]
    .map(([clusterId, cluster]) => {
      const serviceMap = new Map<string, EnrichedService[]>();
      for (const item of cluster.items) {
        const serviceKey = item.view.displayName || item.service_name || t("services.unknownService");
        serviceMap.set(serviceKey, [...(serviceMap.get(serviceKey) || []), item]);
      }

      const serviceNodes = [...serviceMap.entries()]
        .map(([serviceName, serviceItems]) => {
          const instances = serviceItems.map(buildInstanceNode);
          const reachable = instances.filter((instance) => instance.status === "online").length;
          return {
            id: `service:${clusterId}:${serviceName}`,
            type: "service" as const,
            label: serviceName,
            subtitle: t("services.instanceSummary", {
              count: instances.length,
              reachable,
            }),
            instances,
            reachableCount: reachable,
            details: [
              { label: t("services.env"), value: cluster.name },
              { label: t("services.serviceNode"), value: serviceName },
              { label: t("services.instances"), value: String(instances.length) },
              { label: t("services.reachable"), value: String(reachable) },
            ],
          };
        })
        .sort((a, b) => a.label.localeCompare(b.label));

      const instanceCount = serviceNodes.reduce((sum, service) => sum + service.instances.length, 0);
      return {
        id: `cluster:${clusterId}`,
        type: "cluster" as const,
        label: cluster.name,
        subtitle: t("services.serviceSummary", {
          services: serviceNodes.length,
          instances: instanceCount,
        }),
        services: serviceNodes,
        instanceCount,
        details: [
          { label: t("services.env"), value: cluster.name },
          { label: t("services.clusterId"), value: clusterId, mono: true },
          { label: t("services.types"), value: String(serviceNodes.length) },
          { label: t("services.instances"), value: String(instanceCount) },
        ],
      };
    })
    .sort((a, b) => a.label.localeCompare(b.label));
});

const selectedTopologyNode = computed<TopologyNode | null>(() => {
  if (!selectedTopologyNodeId.value) return null;
  for (const cluster of topologyClusters.value) {
    if (cluster.id === selectedTopologyNodeId.value) return cluster;
    for (const service of cluster.services) {
      if (service.id === selectedTopologyNodeId.value) return service;
      const instance = service.instances.find((item) => item.id === selectedTopologyNodeId.value);
      if (instance) return instance;
    }
  }
  return null;
});

const topologyTooltipStyle = computed(() => ({
  left: `${topologyTooltip.value.x}px`,
  top: `${topologyTooltip.value.y}px`,
}));

const statusText = (status: "online" | "offline") =>
  status === "online" ? t("common.online") : t("common.offline");

const topologyTypeText = (type: TopologyNodeType) => {
  if (type === "cluster") return t("services.clusterNode");
  if (type === "service") return t("services.serviceNode");
  return t("services.instanceNode");
};

const selectTopologyNode = (node: TopologyNode, event?: MouseEvent) => {
  keepTopologyTooltip();
  selectedTopologyNodeId.value = node.id;
  hoveredTopologyNode.value = node;
  isTopologyTooltipPinned.value = true;
  if (event) {
    placeTooltipNearPointer(event);
  }
};

const placeTooltipNearPointer = (event: MouseEvent) => {
  const width = 320;
  const padding = 16;
  const offset = 18;
  const height = Math.min(520, window.innerHeight - padding * 2);
  const x =
    event.clientX + width + offset > window.innerWidth
      ? Math.max(padding, event.clientX - width - offset)
      : event.clientX + offset;
  const y = Math.min(Math.max(padding, event.clientY + offset), window.innerHeight - height - padding);
  topologyTooltip.value = { x, y };
};

const placeTooltipNearElement = (event: FocusEvent) => {
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const width = 320;
  const padding = 16;
  const height = Math.min(520, window.innerHeight - padding * 2);
  const x =
    rect.right + width + padding > window.innerWidth
      ? Math.max(padding, rect.left - width - padding)
      : rect.right + padding;
  const y = Math.min(Math.max(padding, rect.top), window.innerHeight - height - padding);
  topologyTooltip.value = { x, y };
};

const showTopologyTooltip = (node: TopologyNode, event: MouseEvent | FocusEvent) => {
  if (isTopologyTooltipPinned.value) return;
  keepTopologyTooltip();
  hoveredTopologyNode.value = node;
  if (event instanceof MouseEvent) {
    placeTooltipNearPointer(event);
  } else {
    placeTooltipNearElement(event);
  }
};

const moveTopologyTooltip = (event: MouseEvent) => {
  if (hoveredTopologyNode.value && !isTopologyTooltipPinned.value) {
    placeTooltipNearPointer(event);
  }
};

const keepTopologyTooltip = () => {
  if (topologyTooltipHideTimer) {
    window.clearTimeout(topologyTooltipHideTimer);
    topologyTooltipHideTimer = undefined;
  }
};

const hideTopologyTooltip = () => {
  if (isTopologyTooltipPinned.value) return;
  keepTopologyTooltip();
  topologyTooltipHideTimer = window.setTimeout(() => {
    hoveredTopologyNode.value = null;
    topologyTooltipHideTimer = undefined;
  }, 180);
};

const closeTopologyTooltip = () => {
  keepTopologyTooltip();
  hoveredTopologyNode.value = null;
  isTopologyTooltipPinned.value = false;
  selectedTopologyNodeId.value = "";
};

const onDocumentClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (target.closest(".topology-node") || target.closest(".topology-tooltip")) return;
  closeTopologyTooltip();
};

const onDocumentKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    closeTopologyTooltip();
  }
};

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
    selectedTopologyNodeId.value = "";
  } catch {
    services.value = [];
    selectedTopologyNodeId.value = "";
    errorMessage.value =
      viewMode.value === "topology" ? t("services.topologyLoadFailed") : t("services.loadFailed");
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  document.addEventListener("click", onDocumentClick);
  document.addEventListener("keydown", onDocumentKeydown);
  await loadClusters();
  if (canRead.value) {
    await loadServices();
  }
});

onUnmounted(() => {
  document.removeEventListener("click", onDocumentClick);
  document.removeEventListener("keydown", onDocumentKeydown);
  keepTopologyTooltip();
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

.view-switch {
  display: inline-flex;
  width: fit-content;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  gap: 4px;
}

.view-switch-btn {
  min-height: 34px;
  padding: 6px 14px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--muted);
  box-shadow: none;
}

.view-switch-btn.active {
  color: var(--primary);
  background: var(--primary-dim);
  border-color: var(--border-strong);
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

.topology-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: start;
}

.topology-map,
.topology-detail {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(5, 14, 28, 0.76);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), var(--shadow-sm);
}

.topology-map {
  position: relative;
  min-height: 560px;
  padding: 50px 18px 18px;
  overflow: auto;
  background:
    radial-gradient(circle at 24% 12%, rgba(34, 211, 238, 0.1), transparent 24%),
    linear-gradient(90deg, rgba(56, 189, 248, 0.05) 1px, transparent 1px),
    linear-gradient(rgba(56, 189, 248, 0.04) 1px, transparent 1px),
    rgba(3, 12, 24, 0.88);
  background-size: auto, 20px 20px, 20px 20px, auto;
}

.topology-map::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    linear-gradient(90deg, rgba(56, 189, 248, 0.08) 1px, transparent 1px),
    linear-gradient(rgba(56, 189, 248, 0.07) 1px, transparent 1px);
  background-size: 100px 100px;
  opacity: 0.42;
}

.topology-tools {
  position: absolute;
  top: 14px;
  left: 16px;
  z-index: 2;
  display: inline-flex;
  overflow: hidden;
  border: 1px solid rgba(96, 165, 250, 0.2);
  border-radius: var(--radius-xs);
  background: rgba(4, 12, 25, 0.82);
}

.topology-tools span {
  display: grid;
  min-width: 42px;
  height: 32px;
  place-items: center;
  border-right: 1px solid rgba(96, 165, 250, 0.16);
  color: #c9d7ee;
  font-size: 12px;
  font-weight: 700;
}

.topology-tools span:last-child {
  border-right: none;
}

.topology-clusters {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 18px;
  min-width: 900px;
}

.topology-lanes {
  display: grid;
  grid-template-columns: 240px minmax(220px, 0.75fr) minmax(260px, 1fr);
  gap: 34px;
  padding: 0 0 12px;
  color: #8ea5c8;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: none;
}

.topology-lanes span {
  padding-left: 4px;
}

.topology-cluster {
  position: relative;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  gap: 34px;
  align-items: start;
  padding: 0;
  border: 0;
  background: transparent;
}

.topology-services,
.topology-instances {
  position: relative;
  display: grid;
  gap: 14px;
}

.topology-services::before,
.topology-instances::before {
  content: "";
  position: absolute;
  top: 26px;
  bottom: 26px;
  left: -18px;
  width: 1px;
  background: linear-gradient(180deg, transparent, rgba(34, 211, 238, 0.58), transparent);
  box-shadow: 0 0 14px rgba(34, 211, 238, 0.3);
}

.topology-service {
  position: relative;
  display: grid;
  grid-template-columns: minmax(220px, 0.75fr) minmax(260px, 1fr);
  gap: 34px;
  align-items: start;
}

.topology-service::before,
.topology-service::after {
  content: "";
  position: absolute;
  pointer-events: none;
}

.topology-service::before {
  top: 26px;
  left: -34px;
  width: 34px;
  height: 1px;
  background: linear-gradient(90deg, rgba(34, 211, 238, 0.12), rgba(34, 211, 238, 0.75));
  box-shadow: 0 0 12px rgba(34, 211, 238, 0.42);
}

.topology-service::after {
  top: 26px;
  left: calc(42.85% - 2px);
  width: 36px;
  height: 1px;
  background: linear-gradient(90deg, rgba(34, 211, 238, 0.72), rgba(34, 211, 238, 0.12));
  box-shadow: 0 0 12px rgba(34, 211, 238, 0.32);
}

.topology-node {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  min-height: 58px;
  padding: 10px 12px;
  border: 1px solid rgba(125, 211, 252, 0.18);
  border-radius: var(--radius-sm);
  background:
    linear-gradient(135deg, rgba(148, 163, 184, 0.08), rgba(14, 165, 233, 0.04)),
    rgba(9, 20, 36, 0.88);
  color: var(--text);
  cursor: pointer;
  text-align: left;
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.22);
  transition: border-color 0.18s, background 0.18s, transform 0.18s, box-shadow 0.18s;
}

.topology-node:focus-visible {
  outline: 2px solid rgba(34, 211, 238, 0.86);
  outline-offset: 3px;
}

.topology-node::before {
  content: "";
  position: absolute;
  top: 50%;
  left: -34px;
  width: 34px;
  height: 1px;
  background: linear-gradient(90deg, rgba(34, 211, 238, 0.08), rgba(34, 211, 238, 0.75));
  box-shadow: 0 0 12px rgba(34, 211, 238, 0.34);
}

.cluster-node::before {
  display: none;
}

.node-icon {
  position: relative;
  display: inline-flex;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  flex-shrink: 0;
  border: 1px solid rgba(56, 189, 248, 0.42);
  background: rgba(14, 165, 233, 0.1);
}

.node-icon::before,
.node-icon::after {
  content: "";
  position: absolute;
  border-radius: 50%;
}

.node-icon::before {
  inset: 7px;
  border: 2px solid var(--primary);
  background: transparent;
  box-shadow: 0 0 14px var(--primary-glow);
}

.node-icon::after {
  display: none;
}

.cluster-icon {
  width: 52px;
  height: 52px;
  border: 0;
  border-radius: 14px;
  background: radial-gradient(circle, rgba(34, 211, 238, 0.16), transparent 68%);
}

.cluster-icon::before {
  inset: 7px 8px 10px;
  border: 3px solid #38bdf8;
  border-top-width: 5px;
  border-bottom-width: 5px;
  border-radius: 50% / 18%;
  box-shadow: inset 0 8px 0 rgba(56, 189, 248, 0.12), inset 0 -8px 0 rgba(56, 189, 248, 0.12), 0 0 18px rgba(56, 189, 248, 0.42);
}

.service-icon {
  background: rgba(14, 165, 233, 0.1);
}

.service-icon::before {
  border-color: #38bdf8;
  border-radius: 3px;
  transform: rotate(30deg) skewY(-30deg);
  box-shadow: 0 0 14px rgba(56, 189, 248, 0.34);
}

.instance-icon {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  border-color: rgba(148, 163, 184, 0.34);
  background: rgba(15, 23, 42, 0.94);
}

.instance-icon::before {
  inset: 4px;
  border: 0;
  border-radius: 999px;
}

.instance-node.online .instance-icon::before {
  background: var(--success);
  box-shadow: 0 0 14px rgba(74, 222, 128, 0.48);
}

.instance-node.offline .instance-icon::before {
  background: var(--danger);
  box-shadow: 0 0 14px rgba(248, 113, 113, 0.35);
}

.node-body {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.topology-node strong,
.node-body > span:last-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topology-node strong {
  font-size: 14px;
  line-height: 1.2;
}

.node-body > span:last-child {
  color: var(--muted);
  font-size: 12px;
  line-height: 1.25;
}

.topology-node:hover,
.topology-node.selected {
  border-color: rgba(56, 189, 248, 0.82);
  background:
    linear-gradient(135deg, rgba(34, 211, 238, 0.18), rgba(96, 165, 250, 0.08)),
    rgba(8, 22, 40, 0.96);
  box-shadow: 0 14px 32px rgba(0, 0, 0, 0.26), 0 0 0 1px rgba(56, 189, 248, 0.16);
  transform: translateY(-1px);
}

.cluster-node {
  min-height: 120px;
  align-self: center;
  border-color: rgba(56, 189, 248, 0.54);
  background:
    linear-gradient(135deg, rgba(56, 189, 248, 0.11), rgba(59, 130, 246, 0.04)),
    rgba(7, 19, 35, 0.96);
  box-shadow: 0 16px 34px rgba(0, 0, 0, 0.28), inset 0 0 0 1px rgba(56, 189, 248, 0.12);
}

.cluster-node:hover,
.cluster-node.selected {
  border-color: rgba(34, 211, 238, 0.95);
  box-shadow:
    0 18px 38px rgba(0, 0, 0, 0.32),
    0 0 0 2px rgba(34, 211, 238, 0.3),
    0 0 28px rgba(34, 211, 238, 0.2);
}

.cluster-node:hover .cluster-icon,
.cluster-node.selected .cluster-icon {
  background: radial-gradient(circle, rgba(34, 211, 238, 0.28), transparent 72%);
}

.instance-node.online {
  border-color: rgba(74, 222, 128, 0.22);
}

.instance-node.offline {
  border-color: rgba(248, 113, 113, 0.22);
}

.node-kicker {
  color: #8ea5c8;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0;
  text-transform: none;
}

.node-health {
  position: absolute;
  top: 9px;
  right: 9px;
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: var(--muted);
  box-shadow: 0 0 0 3px rgba(148, 163, 184, 0.12);
}

.node-health.online {
  background: var(--success);
  box-shadow: 0 0 0 3px rgba(74, 222, 128, 0.12), 0 0 12px rgba(74, 222, 128, 0.48);
}

.node-health.offline {
  background: var(--danger);
  box-shadow: 0 0 0 3px rgba(248, 113, 113, 0.12), 0 0 12px rgba(248, 113, 113, 0.35);
}

.topology-detail {
  min-height: 0;
  padding: 16px;
  background:
    radial-gradient(circle at 0 0, rgba(56, 189, 248, 0.08), transparent 35%),
    rgba(8, 18, 34, 0.9);
}

.topology-tooltip {
  position: fixed;
  z-index: 80;
  width: min(320px, calc(100vw - 32px));
  max-height: min(520px, calc(100vh - 32px));
  overflow: auto;
  pointer-events: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(34, 211, 238, 0.55) rgba(15, 23, 42, 0.8);
  backdrop-filter: blur(18px) saturate(1.15);
  transform: translate3d(0, 0, 0);
}

.topology-tooltip.pinned {
  border-color: rgba(34, 211, 238, 0.72);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.05),
    0 16px 42px rgba(0, 0, 0, 0.38),
    0 0 0 2px rgba(34, 211, 238, 0.16);
}

.topology-tooltip::-webkit-scrollbar {
  width: 8px;
}

.topology-tooltip::-webkit-scrollbar-track {
  background: rgba(15, 23, 42, 0.8);
}

.topology-tooltip::-webkit-scrollbar-thumb {
  background: rgba(34, 211, 238, 0.55);
  border-radius: 999px;
}

.detail-heading {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 4px 10px;
  margin-bottom: 16px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border);
}

.detail-heading h3 {
  grid-column: 1 / 2;
  margin: 0;
  color: var(--text);
  font-size: 18px;
  line-height: 1.3;
  word-break: break-word;
}

.detail-heading .status-badge {
  grid-column: 2 / 3;
  grid-row: 1 / 3;
  align-self: center;
  width: fit-content;
}

.detail-list {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0;
  margin: 0;
}

.detail-list dt {
  color: var(--muted);
  font-size: 12px;
  font-weight: 600;
  padding-top: 10px;
}

.detail-list dd {
  margin: 4px 0 0;
  padding-bottom: 10px;
  border-bottom: 1px solid rgba(125, 211, 252, 0.1);
  color: var(--text);
  font-size: 13px;
  line-height: 1.45;
  word-break: break-word;
}

.detail-section {
  display: grid;
  gap: 8px;
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.detail-placeholder {
  min-height: 480px;
  display: grid;
  place-items: center;
  color: var(--muted);
  text-align: center;
  line-height: 1.5;
}

.topology-legend {
  position: sticky;
  left: 0;
  bottom: 0;
  z-index: 2;
  display: inline-flex;
  gap: 22px;
  width: fit-content;
  margin-top: 18px;
  padding: 10px 14px;
  border: 1px solid rgba(125, 211, 252, 0.18);
  border-radius: var(--radius-sm);
  background: rgba(4, 12, 25, 0.86);
  color: #9fb1cc;
  font-size: 12px;
}

.topology-legend span {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.legend-dot {
  width: 9px;
  height: 9px;
  border-radius: 999px;
  background: var(--muted);
}

.legend-dot.online {
  background: var(--success);
  box-shadow: 0 0 10px rgba(74, 222, 128, 0.5);
}

.legend-dot.offline {
  background: var(--danger);
  box-shadow: 0 0 10px rgba(248, 113, 113, 0.4);
}

.topology-empty {
  margin: 0;
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

@media (max-width: 900px) {
  .topology-layout {
    grid-template-columns: 1fr;
  }
}
</style>
