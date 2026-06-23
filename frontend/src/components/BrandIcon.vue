<template>
  <span class="brand-icon" :class="sizeClass" aria-hidden="true">
    <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="32" height="32" rx="8" fill="#070b14" />
      <circle cx="16" cy="16" r="4" fill="#22d3ee" />
      <path stroke="#22d3ee" stroke-width="1.6" stroke-linecap="round" d="M16 5v4M16 23v4M5 16h4M23 16h4" />
      <path
        stroke="#22d3ee"
        stroke-width="1.2"
        stroke-linecap="round"
        opacity="0.7"
        d="M8.5 8.5l2.8 2.8M20.7 20.7l2.8 2.8M8.5 23.5l2.8-2.8M20.7 11.3l2.8-2.8"
      />
    </svg>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{ size?: "sm" | "md" | "lg" }>(), { size: "md" });

const sizeClass = computed(() => `size-${props.size}`);
</script>

<style scoped>
.brand-icon {
  display: inline-flex;
  flex-shrink: 0;
  filter: drop-shadow(0 0 8px rgba(34, 211, 238, 0.35));
}

.brand-icon svg {
  width: 100%;
  height: 100%;
}

.size-sm {
  width: 28px;
  height: 28px;
}

.size-md {
  width: 40px;
  height: 40px;
}

.size-lg {
  width: 56px;
  height: 56px;
}
</style>
