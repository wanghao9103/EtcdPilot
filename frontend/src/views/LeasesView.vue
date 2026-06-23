<template>
  <section class="panel">
    <div class="section-head">
      <div>
        <h2>{{ t('leases.title') }}</h2>
        <p class="hint section-desc">{{ t('leases.desc') }}</p>
      </div>
      <button class="primary btn-icon" type="button" @click="loadList" :disabled="loading || !canRead">
        <span v-if="loading" class="spinner"></span>
        {{ loading ? t('common.refreshing') : t('common.refresh') }}
      </button>
    </div>

    <p v-if="!canRead" class="message error">{{ t('leases.noPerm') }}</p>
    <p v-if="error" class="message error">{{ error }}</p>
    <p v-if="ok" class="message ok">{{ ok }}</p>

    <div class="filter-bar">
      <label class="filter-item">
        <span>{{ t('keys.env') }}</span>
        <select v-model="clusterId" @change="loadList">
          <option v-for="c in clusters" :key="c.id" :value="c.id">{{ c.name }}</option>
        </select>
      </label>
      <label class="filter-item grow">
        <span>{{ t('common.search') }}</span>
        <input v-model="keyword" :placeholder="t('leases.searchPh')" />
      </label>
    </div>

    <div class="summary" v-if="list.length">
      <article class="card stat-card">
        <h3>{{ t('leases.active') }}</h3>
        <p class="stat-value">{{ activeCount }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('leases.expiring') }}</h3>
        <p class="stat-value">{{ expiringCount }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('leases.expired') }}</h3>
        <p class="stat-value">{{ expiredCount }}</p>
      </article>
    </div>

    <div class="lease-list">
      <article
        v-for="item in filteredList"
        :key="item.id"
        class="lease-card interactive-card"
        :class="{ active: selected?.id === item.id }"
        @click="loadDetailFor(item.id)"
      >
        <div class="lease-head">
          <div>
            <p class="lease-label">{{ t('leases.leaseId') }}</p>
            <p class="lease-id mono">{{ shortId(item.id) }}</p>
          </div>
          <span class="status-badge" :class="ttlStatus(item).class">{{ ttlStatus(item).text }}</span>
        </div>
        <div class="lease-info">
          <div>
            <span class="info-label">{{ t('leases.remaining') }}</span>
            <span class="info-value">{{ formatTtl(item.ttl) }}</span>
          </div>
          <div>
            <span class="info-label">{{ t('leases.linkedData') }}</span>
            <span class="info-value">{{ t('common.rows', { count: item.keys_count }) }}</span>
          </div>
        </div>
      </article>

      <div v-if="filteredList.length === 0 && !loading" class="empty-state">
        <div class="empty-icon">◷</div>
        <p v-if="keyword">{{ t('leases.noMatch') }}</p>
        <p v-else>{{ t('leases.empty') }}</p>
      </div>
    </div>

    <details v-if="selected" class="tech-details" open>
      <summary>{{ t('leases.techDetails') }}</summary>
      <div class="tech-body">
        <div class="tech-row">
          <span>{{ t('leases.fullId') }}</span>
          <code class="mono">{{ selected.id }}</code>
        </div>
        <div class="tech-row">
          <span>{{ t('leases.grantedTtl') }}</span>
          <span>{{ t('common.seconds', { n: selected.granted_ttl }) }}</span>
        </div>
        <pre class="pre-block">{{ JSON.stringify(selected, null, 2) }}</pre>
      </div>
    </details>
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

const { t } = useI18n();
const auth = useAuthStore();
const clusters = ref<ClusterInfo[]>([]);
const clusterId = ref("");
const keyword = ref("");
const list = ref<any[]>([]);
const selected = ref<any | null>(null);
const error = ref("");
const ok = ref("");
const loading = ref(false);

const canRead = computed(() => auth.permissions.includes("lease:read"));

const filteredList = computed(() => {
  const q = keyword.value.trim();
  if (!q) return list.value;
  return list.value.filter((item) => String(item.id).includes(q));
});

const activeCount = computed(() => list.value.filter((i) => i.ttl > 60).length);
const expiringCount = computed(() => list.value.filter((i) => i.ttl > 0 && i.ttl <= 60).length);
const expiredCount = computed(() => list.value.filter((i) => i.ttl <= 0).length);

const shortId = (id: string | number) => {
  const s = String(id);
  return s.length > 10 ? `${s.slice(0, 6)}…${s.slice(-4)}` : s;
};

const formatTtl = (ttl: number) => {
  if (ttl <= 0) return t("common.expired");
  if (ttl < 60) return t("common.seconds", { n: ttl });
  if (ttl < 3600) return t("common.minutes", { n: Math.floor(ttl / 60) });
  return t("common.hoursMinutes", { h: Math.floor(ttl / 3600), m: Math.floor((ttl % 3600) / 60) });
};

const ttlStatus = (item: { ttl: number }) => {
  if (item.ttl <= 0) return { class: "offline", text: t("common.expired") };
  if (item.ttl <= 60) return { class: "warn", text: t("common.expiring") };
  return { class: "online", text: t("common.valid") };
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

const loadList = async () => {
  if (!canRead.value) {
    list.value = [];
    return;
  }
  error.value = "";
  ok.value = "";
  selected.value = null;
  loading.value = true;
  try {
    list.value = (await api.get(`/clusters/${clusterId.value}/leases`)).data || [];
  } catch (err: any) {
    list.value = [];
    error.value = err.message || t("leases.loadFailed");
  } finally {
    loading.value = false;
  }
};

const loadDetailFor = async (id: string | number) => {
  if (!canRead.value) return;
  error.value = "";
  ok.value = "";
  try {
    selected.value = (await api.get(`/clusters/${clusterId.value}/leases/${id}`)).data || null;
    if (selected.value) {
      ok.value = t("leases.detailLoaded");
    }
  } catch (err: any) {
    selected.value = null;
    error.value = err.message || t("leases.detailFailed");
  }
};

onMounted(async () => {
  await loadClusters();
  if (canRead.value) {
    await loadList();
  }
});
</script>

<style scoped>
.lease-list {
  display: grid;
  gap: 10px;
}

.lease-card {
  cursor: pointer;
}

.lease-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 12px;
}

.lease-label {
  margin: 0;
  font-size: 11px;
  color: var(--muted);
}

.lease-id {
  margin: 2px 0 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
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

.status-badge.warn {
  color: var(--warning);
  background: var(--warning-dim);
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.status-badge.offline {
  color: var(--muted);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border);
}

.lease-info {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.info-label {
  display: block;
  font-size: 11px;
  color: var(--muted);
  margin-bottom: 2px;
}

.info-value {
  font-size: 14px;
  color: var(--text);
  font-weight: 500;
}
</style>
