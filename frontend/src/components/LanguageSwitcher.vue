<template>
  <div class="lang-switch" :title="t('lang.switch')">
    <button
      type="button"
      class="lang-btn"
      :class="{ active: locale === 'zh-CN' }"
      @click="switchTo('zh-CN')"
    >
      {{ t("lang.zh") }}
    </button>
    <button
      type="button"
      class="lang-btn"
      :class="{ active: locale === 'en-US' }"
      @click="switchTo('en-US')"
    >
      {{ t("lang.en") }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { setAppLocale, type AppLocale } from "../i18n";

const { t, locale } = useI18n();

const switchTo = (next: AppLocale) => {
  if (locale.value !== next) {
    setAppLocale(next);
  }
};
</script>

<style scoped>
.lang-switch {
  display: inline-flex;
  padding: 3px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--surface-2);
}

.lang-btn {
  appearance: none;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 11px;
  font-weight: 600;
  padding: 5px 11px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  font-family: var(--font);
  letter-spacing: 0.02em;
  transition: color 0.15s, background 0.15s;
}

.lang-btn:hover {
  color: var(--text);
}

.lang-btn.active {
  color: var(--primary);
  background: var(--primary-dim);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}
</style>
