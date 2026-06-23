<template>
  <section class="panel">
    <div class="section-head">
      <div>
        <h2>{{ greeting }}</h2>
        <p class="hint section-desc">{{ t('dashboard.welcome') }}</p>
      </div>
      <button class="ghost btn-icon" type="button" @click="load" :disabled="loading">
        <span v-if="loading" class="spinner"></span>
        {{ loading ? t('common.refreshing') : t('common.refresh') }}
      </button>
    </div>

    <p v-if="error" class="message error">{{ error }}</p>

    <div class="grid">
      <article class="card stat-card">
        <h3>{{ t('dashboard.envCount') }}</h3>
        <p class="stat-value">{{ clusters.length }}</p>
        <p class="stat-hint">{{ t('dashboard.envUnit') }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('dashboard.myRole') }}</h3>
        <p class="stat-value text">{{ roleLabel }}</p>
        <p class="stat-hint">{{ permissionHint }}</p>
      </article>
      <article class="card stat-card">
        <h3>{{ t('dashboard.writableEnv') }}</h3>
        <p class="stat-value">{{ writableCount }}</p>
        <p class="stat-hint">{{ t('dashboard.writableUnit') }}</p>
      </article>
    </div>

    <div class="section-block">
      <h3 class="block-title">{{ t('dashboard.shortcuts') }}</h3>
      <div class="quick-links">
        <router-link v-for="item in shortcuts" :key="item.to" :to="item.to" class="quick-link">
          <span class="ql-icon" aria-hidden="true" v-html="item.icon"></span>
          <div>
            <strong>{{ item.title }}</strong>
            <span class="ql-desc">{{ item.desc }}</span>
          </div>
        </router-link>
      </div>
    </div>

    <div v-if="clusters.length" class="section-block">
      <h3 class="block-title">{{ t('dashboard.envOverview') }}</h3>
      <div class="env-list">
        <article v-for="cluster in clusters" :key="cluster.id" class="env-card">
          <span class="env-name">{{ cluster.name }}</span>
          <span class="badge" :class="cluster.readonly ? 'warn' : 'ok'">
            {{ cluster.readonly ? t('common.readonly') : t('common.writable') }}
          </span>
        </article>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import api from "../api";
import { useAuthStore } from "../stores/auth";
import { roleLabelOf } from "../composables/useRoleLabel";
import { buildGreeting } from "../composables/useGreeting";

const { t } = useI18n();
const auth = useAuthStore();
const clusters = ref<any[]>([]);
const loading = ref(false);
const error = ref("");

const greeting = computed(() => buildGreeting(t, auth.username));
const roleLabel = computed(() => roleLabelOf(t, auth.role || "guest"));
const writableCount = computed(() => clusters.value.filter((c) => !c.readonly).length);

const permissionHint = computed(() => {
  if (auth.role === "admin") return t("dashboard.permissionAdmin");
  if (auth.role === "operator") return t("dashboard.permissionOperator");
  if (auth.role === "readonly" || auth.role === "viewer") return t("dashboard.permissionReadonly");
  return t("dashboard.permissionLimited");
});

const shortcutIcons = {
  clusters: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/><path d="M12 7v4M7.5 17l3-4M16.5 17l-3-4"/></svg>`,
  services: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/></svg>`,
  keys: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>`,
  leases: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 3"/></svg>`,
};

const shortcuts = computed(() => [
  {
    to: "/clusters",
    icon: shortcutIcons.clusters,
    title: t("dashboard.scClusters"),
    desc: t("dashboard.scClustersDesc"),
  },
  {
    to: "/services",
    icon: shortcutIcons.services,
    title: t("dashboard.scServices"),
    desc: t("dashboard.scServicesDesc"),
  },
  {
    to: "/keys",
    icon: shortcutIcons.keys,
    title: t("dashboard.scKeys"),
    desc: t("dashboard.scKeysDesc"),
  },
  {
    to: "/leases",
    icon: shortcutIcons.leases,
    title: t("dashboard.scLeases"),
    desc: t("dashboard.scLeasesDesc"),
  },
]);

const load = async () => {
  loading.value = true;
  error.value = "";
  try {
    clusters.value = (await api.get("/clusters")).data || [];
  } catch (e: any) {
    clusters.value = [];
    error.value = e?.message || t("common.loadFailed");
  } finally {
    loading.value = false;
  }
};

onMounted(load);
</script>

<style scoped>
.env-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 10px;
}

.env-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  padding: 13px 15px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--surface-2);
  transition: border-color 0.2s, background 0.2s;
}

.env-card:hover {
  border-color: var(--border-strong);
  background: rgba(34, 211, 238, 0.04);
}

.env-name {
  font-size: 14px;
  font-weight: 500;
}
</style>
