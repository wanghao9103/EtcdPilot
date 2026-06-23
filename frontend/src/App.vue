<template>
  <div class="app-shell" :class="{ 'is-login': isLoginPage }">
    <div class="bg-grid" aria-hidden="true"></div>
    <header v-if="!isLoginPage" class="topbar">
      <div class="brand-wrap">
        <BrandIcon size="sm" />
        <h1 class="brand">EtcdPilot</h1>
        <span class="brand-tag">{{ t("app.console") }}</span>
      </div>
      <div class="topbar-nav-scroll">
        <nav class="topbar-nav" :aria-label="t('nav.primary')">
          <router-link v-for="item in navItems" :key="item.to" :to="item.to" class="nav-link">
            <span class="nav-icon" aria-hidden="true" v-html="item.icon"></span>
            <span class="nav-text">{{ item.label }}</span>
          </router-link>
        </nav>
      </div>
      <div class="topbar-actions">
        <LanguageSwitcher />
        <template v-if="auth.loggedIn">
          <div ref="userMenuRef" class="user-menu">
            <button
              type="button"
              class="user-trigger"
              :aria-expanded="userMenuOpen"
              aria-haspopup="true"
              @click="userMenuOpen = !userMenuOpen"
            >
              <span class="user-avatar">{{ userInitial }}</span>
              <span class="user-trigger-text">
                <span class="user-name">{{ auth.username }}</span>
                <span class="user-role-badge">{{ roleLabel }}</span>
              </span>
              <svg class="user-chevron" :class="{ open: userMenuOpen }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M6 9l6 6 6-6" />
              </svg>
            </button>
            <div v-if="userMenuOpen" class="user-dropdown" role="menu">
              <div class="dropdown-header">
                <span class="user-avatar lg">{{ userInitial }}</span>
                <div>
                  <p class="dropdown-greeting">{{ greeting }}</p>
                  <p class="dropdown-role">{{ roleLabel }}</p>
                </div>
              </div>
              <div class="dropdown-divider"></div>
              <button type="button" class="dropdown-item danger" role="menuitem" @click="logout">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" width="16" height="16">
                  <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9" />
                </svg>
                {{ t("nav.logout") }}
              </button>
            </div>
          </div>
        </template>
        <router-link v-else to="/login" class="nav-link nav-login">
          <span>{{ t("nav.login") }}</span>
        </router-link>
      </div>
    </header>
    <LanguageSwitcher v-if="isLoginPage" class="login-lang-switch" />
    <main class="content">
      <router-view v-slot="{ Component }">
        <transition name="page" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useAuthStore } from "./stores/auth";
import { useRoute, useRouter } from "vue-router";
import { roleLabelOf } from "./composables/useRoleLabel";
import { buildGreeting } from "./composables/useGreeting";
import BrandIcon from "./components/BrandIcon.vue";
import LanguageSwitcher from "./components/LanguageSwitcher.vue";

const { t } = useI18n();
const auth = useAuthStore();
const route = useRoute();
const router = useRouter();
const isLoginPage = computed(() => route.path === "/login");
const userMenuOpen = ref(false);
const userMenuRef = ref<HTMLElement | null>(null);

const roleLabel = computed(() => roleLabelOf(t, auth.role));
const userInitial = computed(() => (auth.username?.[0] || "?").toUpperCase());
const greeting = computed(() => buildGreeting(t, auth.username));

const navIcons = {
  overview: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>`,
  clusters: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/><path d="M12 7v4M7.5 17l3-4M16.5 17l-3-4"/></svg>`,
  keys: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>`,
  services: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/></svg>`,
  leases: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 3"/></svg>`,
  audits: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6M16 13H8M16 17H8M10 9H8"/></svg>`,
};

const navItems = computed(() => [
  { to: "/", label: t("nav.overview"), icon: navIcons.overview },
  { to: "/clusters", label: t("nav.clusters"), icon: navIcons.clusters },
  { to: "/keys", label: t("nav.keys"), icon: navIcons.keys },
  { to: "/services", label: t("nav.services"), icon: navIcons.services },
  { to: "/leases", label: t("nav.leases"), icon: navIcons.leases },
  { to: "/audits", label: t("nav.audits"), icon: navIcons.audits },
]);

const onDocClick = (e: MouseEvent) => {
  if (!userMenuRef.value?.contains(e.target as Node)) {
    userMenuOpen.value = false;
  }
};

onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));

const logout = async () => {
  userMenuOpen.value = false;
  await auth.logout();
  router.push("/login");
};
</script>

<style>
:root {
  --page-bg: #060a12;
  --page-bg-2: #0b1120;
  --card-bg: rgba(12, 20, 36, 0.82);
  --card-bg-solid: #0c1424;
  --surface-2: rgba(0, 0, 0, 0.22);
  --surface-3: rgba(0, 0, 0, 0.35);
  --border: rgba(56, 189, 248, 0.12);
  --border-strong: rgba(56, 189, 248, 0.26);
  --text: #eef4ff;
  --muted: #8b9cb8;
  --primary: #22d3ee;
  --primary-dim: rgba(34, 211, 238, 0.1);
  --primary-glow: rgba(34, 211, 238, 0.32);
  --primary-foreground: #041018;
  --accent: #818cf8;
  --danger: #f87171;
  --danger-dim: rgba(248, 113, 113, 0.1);
  --success: #4ade80;
  --success-dim: rgba(74, 222, 128, 0.1);
  --warning: #fbbf24;
  --warning-dim: rgba(251, 191, 36, 0.1);
  --code-text: #a5f3fc;
  --radius: 14px;
  --radius-sm: 8px;
  --radius-xs: 6px;
  --shadow: 0 12px 40px rgba(0, 0, 0, 0.42);
  --shadow-sm: 0 4px 16px rgba(0, 0, 0, 0.28);
  --font: "Inter", "PingFang SC", "Microsoft YaHei", sans-serif;
  --mono: "JetBrains Mono", Consolas, Menlo, monospace;
  --content-max: 1360px;
  --topbar-h: 58px;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  width: 100%;
  min-height: 100%;
  font-family: var(--font);
  background: var(--page-bg);
  color: var(--text);
  -webkit-font-smoothing: antialiased;
  scrollbar-width: thin;
  scrollbar-color: rgba(34, 211, 238, 0.28) transparent;
}

html::-webkit-scrollbar,
body::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

html::-webkit-scrollbar-thumb,
body::-webkit-scrollbar-thumb {
  background: rgba(34, 211, 238, 0.22);
  border-radius: 999px;
}

html::-webkit-scrollbar-thumb:hover,
body::-webkit-scrollbar-thumb:hover {
  background: rgba(34, 211, 238, 0.38);
}

::selection {
  background: rgba(34, 211, 238, 0.28);
  color: var(--text);
}

.app-shell {
  position: relative;
  min-height: 100vh;
}

.bg-grid {
  position: fixed;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(ellipse 90% 55% at 50% -15%, rgba(34, 211, 238, 0.09), transparent 55%),
    radial-gradient(ellipse 50% 40% at 100% 0%, rgba(129, 140, 248, 0.06), transparent 50%),
    linear-gradient(var(--page-bg) 0%, var(--page-bg-2) 100%);
  z-index: 0;
}

.bg-grid::before {
  content: "";
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(56, 189, 248, 0.035) 1px, transparent 1px),
    linear-gradient(90deg, rgba(56, 189, 248, 0.035) 1px, transparent 1px);
  background-size: 56px 56px;
  mask-image: radial-gradient(ellipse 75% 65% at 50% 25%, black 15%, transparent 78%);
}

.bg-grid::after {
  content: "";
  position: absolute;
  inset: 0;
  background: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.03'/%3E%3C/svg%3E");
  opacity: 0.4;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 0 22px;
  height: var(--topbar-h);
  background: rgba(6, 10, 18, 0.78);
  backdrop-filter: blur(20px) saturate(1.2);
  border-bottom: 1px solid var(--border);
  box-shadow: 0 1px 0 rgba(34, 211, 238, 0.06);
}

.topbar-nav-scroll {
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  scrollbar-width: none;
  mask-image: linear-gradient(90deg, transparent, black 12px, black calc(100% - 12px), transparent);
}

.topbar-nav-scroll::-webkit-scrollbar {
  display: none;
}

.topbar-nav {
  display: flex;
  align-items: center;
  gap: 3px;
  flex-wrap: nowrap;
  justify-content: center;
  padding: 0 8px;
  min-width: min-content;
}

.brand-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.brand {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.02em;
  background: linear-gradient(135deg, #e8f0ff 30%, var(--primary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.brand-tag {
  font-size: 11px;
  color: var(--muted);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 1px 6px;
  letter-spacing: 0.08em;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  margin-left: auto;
}

.user-menu {
  position: relative;
}

.user-trigger {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 10px 4px 4px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.04);
  color: var(--text);
  cursor: pointer;
  font-family: var(--font);
  transition: border-color 0.2s, background 0.2s;
}

.user-trigger:hover,
.user-trigger[aria-expanded="true"] {
  border-color: var(--border-strong);
  background: rgba(255, 255, 255, 0.07);
}

.user-trigger-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
  line-height: 1.2;
  text-align: left;
}

.user-chevron {
  width: 14px;
  height: 14px;
  color: var(--muted);
  transition: transform 0.2s;
  flex-shrink: 0;
}

.user-chevron.open {
  transform: rotate(180deg);
}

.user-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 220px;
  padding: 8px;
  border-radius: var(--radius);
  border: 1px solid var(--border-strong);
  background: var(--card-bg-solid);
  box-shadow: var(--shadow);
  z-index: 30;
}

.dropdown-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 10px;
}

.dropdown-greeting {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.dropdown-role {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--muted);
}

.dropdown-divider {
  height: 1px;
  background: var(--border);
  margin: 6px 0;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text);
  font-size: 13px;
  font-family: var(--font);
  cursor: pointer;
  transition: background 0.15s;
}

.dropdown-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.dropdown-item.danger {
  color: var(--danger);
}

.dropdown-item.danger:hover {
  background: var(--danger-dim);
}

.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 11px;
  border-radius: var(--radius-sm);
  color: var(--muted);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  transition: color 0.2s, background 0.2s, box-shadow 0.2s, transform 0.15s;
}

.nav-link:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.05);
}

.nav-link.router-link-active,
.nav-link.router-link-exact-active {
  color: var(--primary);
  background: var(--primary-dim);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}

.nav-text {
  line-height: 1.2;
}

.nav-icon {
  display: flex;
  width: 15px;
  height: 15px;
}

.nav-icon svg {
  width: 100%;
  height: 100%;
}

.user-avatar {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary-dim), rgba(129, 140, 248, 0.2));
  border: 1px solid var(--border-strong);
  color: var(--primary);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.user-avatar.lg {
  width: 36px;
  height: 36px;
  font-size: 14px;
}

.user-name {
  color: var(--text);
  font-weight: 600;
  font-size: 13px;
}

.user-role-badge {
  font-size: 10px;
  color: var(--primary);
  background: var(--primary-dim);
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 500;
}

.content {
  position: relative;
  z-index: 1;
  max-width: var(--content-max);
  margin: 0 auto;
  padding: 26px 22px 48px;
}

.is-login .content {
  max-width: none;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  position: relative;
}

.login-lang-switch {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 30;
}

.page-enter-active,
.page-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 2px;
}

button:focus:not(:focus-visible),
a:focus:not(:focus-visible),
input:focus:not(:focus-visible),
select:focus:not(:focus-visible),
textarea:focus:not(:focus-visible) {
  outline: none;
}

/* Buttons */
.ghost,
button {
  appearance: none;
  border: 1px solid var(--border-strong);
  background: rgba(255, 255, 255, 0.04);
  color: var(--text);
  border-radius: var(--radius-sm);
  padding: 8px 14px;
  font-size: 13px;
  font-family: var(--font);
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s, border-color 0.2s, box-shadow 0.2s, transform 0.1s;
}

.ghost:hover,
button:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  border-color: var(--primary);
}

button:active:not(:disabled) {
  transform: scale(0.98);
}

button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

button.primary {
  border-color: transparent;
  background: linear-gradient(135deg, #22d3ee, #06b6d4);
  color: var(--primary-foreground);
  box-shadow: 0 0 16px rgba(34, 211, 238, 0.25);
}

button.primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #67e8f9, #22d3ee);
  box-shadow: 0 0 24px rgba(34, 211, 238, 0.4);
}

button.danger {
  border-color: rgba(248, 113, 113, 0.4);
  background: var(--danger-dim);
  color: var(--danger);
}

button.danger:hover:not(:disabled) {
  background: rgba(248, 113, 113, 0.2);
  border-color: var(--danger);
}

.btn-sm {
  padding: 5px 10px;
  font-size: 12px;
}

.btn-icon {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.topbar .ghost {
  border-color: var(--border);
  background: transparent;
  color: var(--muted);
}

.topbar .ghost:hover {
  color: var(--text);
  border-color: var(--border-strong);
}

/* Form controls */
input,
select,
textarea {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 9px 12px;
  font-size: 13px;
  font-family: var(--font);
  background: rgba(0, 0, 0, 0.25);
  color: var(--text);
  transition: border-color 0.2s, box-shadow 0.2s;
  width: 100%;
}

input::placeholder,
textarea::placeholder {
  color: rgba(139, 156, 184, 0.6);
}

input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-dim);
}

select {
  cursor: pointer;
}

select option {
  background: var(--card-bg-solid);
}

/* Panels & cards */
.panel {
  position: relative;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 22px;
  backdrop-filter: blur(14px);
  box-shadow: var(--shadow);
}

.panel::before {
  content: "";
  position: absolute;
  top: 0;
  left: 24px;
  right: 24px;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(34, 211, 238, 0.45), transparent);
  pointer-events: none;
}

.panel h2 {
  margin: 0;
  font-size: 21px;
  font-weight: 650;
  letter-spacing: -0.02em;
}

.section-head {
  margin: 0 0 18px;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.section-head .section-desc {
  margin: 4px 0 0;
  font-size: 13px;
}

.toolbar {
  margin-bottom: 16px;
  display: flex;
  gap: 10px;
  align-items: flex-end;
  flex-wrap: wrap;
  padding: 14px;
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border);
}

.toolbar-title {
  width: 100%;
  margin: 0 0 4px;
  font-size: 12px;
  font-weight: 600;
  color: var(--primary);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.toolbar label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.toolbar .grow {
  min-width: 200px;
  flex: 1;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 14px;
}

.card {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.15s;
}

.card:hover {
  border-color: var(--border-strong);
}

.card h3 {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

/* Shared filters & lists */
.filter-bar {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-wrap: wrap;
  padding: 14px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--surface-2);
  margin-bottom: 16px;
}

.filter-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.filter-item.grow {
  flex: 1;
  min-width: 200px;
}

.summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.list-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.search-input {
  max-width: 280px;
  min-width: 180px;
}

.edit-panel {
  padding: 16px 18px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.12);
  margin-bottom: 16px;
}

.panel-label {
  margin: 0 0 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--primary);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.edit-grid {
  display: grid;
  gap: 12px;
}

.edit-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.content-field {
  min-width: 0;
}

.edit-field-label {
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}

.edit-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 14px;
}

.interactive-card {
  padding: 14px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--surface-2);
  transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
}

.interactive-card:hover {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-sm);
}

.interactive-card.active {
  border-color: var(--primary);
  background: var(--primary-dim);
  box-shadow: 0 0 0 1px rgba(34, 211, 238, 0.15);
}

.section-block {
  margin-top: 24px;
}

.block-title {
  margin: 0 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.stat-hint {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--muted);
  line-height: 1.4;
}

.status-dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.online {
  background: var(--success);
  box-shadow: 0 0 10px rgba(74, 222, 128, 0.55);
}

.status-dot.offline {
  background: var(--danger);
  box-shadow: 0 0 8px rgba(248, 113, 113, 0.35);
}

.status-dot.unknown {
  background: var(--muted);
}

.tech-inline,
.tech-details {
  margin-top: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-xs);
  overflow: hidden;
  background: rgba(0, 0, 0, 0.12);
}

.tech-inline {
  border: none;
  background: transparent;
  border-radius: 0;
}

.tech-inline summary,
.tech-details summary {
  padding: 8px 0;
  cursor: pointer;
  font-size: 11px;
  color: var(--muted);
  user-select: none;
  list-style: none;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: color 0.15s;
}

.tech-details summary {
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.15);
  font-size: 12px;
}

.tech-inline summary::before,
.tech-details summary::before {
  content: "▶";
  font-size: 9px;
  transition: transform 0.2s;
  opacity: 0.7;
}

.tech-inline[open] summary::before,
.tech-details[open] summary::before {
  transform: rotate(90deg);
}

.tech-inline summary:hover,
.tech-details summary:hover {
  color: var(--text);
}

.tech-inline-body,
.tech-body {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 11px;
  color: var(--muted);
}

.tech-inline-body {
  margin-top: 6px;
  padding: 0 2px 4px;
}

.tech-body {
  padding: 12px 14px;
  gap: 10px;
}

.tech-inline code,
.tech-body code,
.tech-row code {
  color: var(--code-text);
  word-break: break-all;
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

/* Tables */
.table-wrap {
  overflow-x: auto;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.data-table th,
.data-table td {
  border-bottom: 1px solid var(--border);
  padding: 10px 12px;
  text-align: left;
  vertical-align: middle;
}

.data-table th {
  color: var(--muted);
  font-weight: 600;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  background: rgba(0, 0, 0, 0.25);
  white-space: nowrap;
}

.data-table tbody tr {
  transition: background 0.15s;
}

.data-table tbody tr:hover {
  background: rgba(34, 211, 238, 0.04);
}

.data-table tbody tr:last-child td {
  border-bottom: none;
}

.data-table tbody tr.clickable {
  cursor: pointer;
}

.data-table tbody tr.row-active {
  background: var(--primary-dim);
}

/* Typography helpers */
.hint {
  color: var(--muted);
  font-size: 13px;
  line-height: 1.5;
}

.mono {
  font-family: var(--mono);
  font-size: 12px;
}

/* Messages */
.message {
  margin: 0 0 14px;
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.message.error {
  color: var(--danger);
  background: var(--danger-dim);
  border: 1px solid rgba(248, 113, 113, 0.25);
}

.message.ok {
  color: var(--success);
  background: var(--success-dim);
  border: 1px solid rgba(74, 222, 128, 0.25);
}

/* Badges */
.badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid transparent;
}

.badge.ok {
  color: var(--success);
  background: var(--success-dim);
  border-color: rgba(74, 222, 128, 0.3);
}

.badge.error {
  color: var(--danger);
  background: var(--danger-dim);
  border-color: rgba(248, 113, 113, 0.3);
}

.badge.neutral {
  color: var(--muted);
  background: rgba(255, 255, 255, 0.04);
  border-color: var(--border);
}

.badge.warn {
  color: var(--warning);
  background: var(--warning-dim);
  border-color: rgba(251, 191, 36, 0.3);
}

/* Empty state */
.empty-state {
  text-align: center;
  padding: 48px 24px;
  color: var(--muted);
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.12);
}

.empty-state .empty-icon {
  font-size: 36px;
  margin-bottom: 12px;
  opacity: 0.45;
  filter: grayscale(0.2);
}

.empty-state p {
  margin: 0;
  font-size: 14px;
  line-height: 1.5;
}

/* Spinner */
.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.15);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  vertical-align: middle;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Code blocks */
.pre-block {
  margin: 12px 0 0;
  white-space: pre-wrap;
  word-break: break-word;
  background: rgba(0, 0, 0, 0.35);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 12px;
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.6;
  color: #a5f3fc;
  overflow-x: auto;
}

/* Stat cards */
.stat-card {
  position: relative;
  overflow: hidden;
  padding: 18px 16px;
}

.stat-card::before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, var(--primary), var(--accent));
  opacity: 0.75;
}

.stat-card::after {
  content: "";
  position: absolute;
  right: -20px;
  top: -20px;
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: radial-gradient(circle, var(--primary-dim), transparent 70%);
  pointer-events: none;
}

.stat-value {
  margin: 12px 0 0;
  font-size: 34px;
  font-weight: 700;
  color: var(--text);
  line-height: 1;
  letter-spacing: -0.02em;
}

.stat-value.text {
  font-size: 19px;
  font-weight: 600;
  letter-spacing: 0;
}

/* Quick links */
.quick-links {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 10px;
  margin-top: 4px;
}

.quick-link {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--text);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  transition: border-color 0.2s, background 0.2s, transform 0.15s, box-shadow 0.2s;
}

.quick-link:hover {
  border-color: var(--primary);
  background: var(--primary-dim);
  transform: translateY(-2px);
  box-shadow: var(--shadow-sm);
}

.quick-link .ql-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-xs);
  background: var(--primary-dim);
  border: 1px solid var(--border);
  color: var(--primary);
  flex-shrink: 0;
}

.quick-link .ql-icon svg {
  width: 18px;
  height: 18px;
}

.quick-link strong {
  display: block;
  font-size: 14px;
  margin-bottom: 3px;
  font-weight: 600;
}

.ql-desc {
  font-size: 12px;
  color: var(--muted);
  font-weight: 400;
  line-height: 1.4;
}

/* Collapsible detail */
.detail-block {
  margin-top: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.detail-block summary {
  padding: 10px 14px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: var(--muted);
  background: rgba(0, 0, 0, 0.2);
  user-select: none;
  list-style: none;
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-block summary::before {
  content: "▶";
  font-size: 10px;
  transition: transform 0.2s;
}

.detail-block[open] summary::before {
  transform: rotate(90deg);
}

.detail-block summary:hover {
  color: var(--text);
}

.detail-block .detail-body {
  padding: 12px 14px;
}

@media (max-width: 900px) {
  .topbar {
    flex-wrap: wrap;
    height: auto;
    min-height: var(--topbar-h);
    padding: 10px 14px;
    gap: 8px;
  }

  .topbar-nav-scroll {
    width: 100%;
    order: 3;
    mask-image: linear-gradient(90deg, transparent, black 8px, black calc(100% - 8px), transparent);
  }

  .topbar-nav {
    justify-content: flex-start;
    padding: 2px 4px 4px;
  }

  .topbar-actions {
    margin-left: auto;
  }

  .user-trigger-text {
    display: none;
  }

  .brand-tag {
    display: none;
  }

  .content {
    padding: 16px 14px 36px;
  }

  .panel {
    padding: 18px 16px;
  }
}
</style>
