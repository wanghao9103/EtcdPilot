<template>
  <div class="path-prefix-input">
    <div
      class="tags-box"
      :class="{ focused }"
      @click="focusInput"
    >
      <div class="tags-scroll">
        <span v-for="(tag, index) in tags" :key="`${tag}-${index}`" class="tag">
          <span class="tag-text" :title="tag">{{ tag }}</span>
          <button
            type="button"
            class="tag-remove"
            :aria-label="t('services.removePrefix', { path: tag })"
            @click.stop="removeTag(index)"
          >
            ×
          </button>
        </span>
        <input
          ref="inputRef"
          v-model="draft"
          class="tag-input"
          :placeholder="tags.length ? '' : placeholder"
          @focus="focused = true"
          @blur="onBlur"
          @keydown="onKeydown"
          @paste="onPaste"
        />
      </div>
    </div>

    <div class="prefix-footer">
      <p class="field-hint">{{ t("services.pathHint") }}</p>
      <div class="prefix-tools">
        <div v-if="availablePresets.length" class="quick-presets">
          <span class="presets-label">{{ t("services.quickAdd") }}</span>
          <button
            v-for="preset in availablePresets"
            :key="preset"
            type="button"
            class="preset-btn"
            @click="addTag(preset)"
          >
            + {{ preset }}
          </button>
        </div>
        <button
          v-if="tags.length"
          type="button"
          class="ghost btn-sm clear-btn"
          @click="clearAll"
        >
          {{ t("services.clearPrefixes") }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    presets?: string[];
  }>(),
  {
    placeholder: "/services/",
    presets: () => ["/services/", "/apps/", "/registry/"],
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t } = useI18n();
const draft = ref("");
const focused = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

const parsePrefixes = (raw: string) =>
  raw
    .split(/[,;\n]+/)
    .map((s) => s.trim())
    .filter(Boolean);

const tags = computed(() => parsePrefixes(props.modelValue));

const availablePresets = computed(() =>
  props.presets.filter((preset) => !tags.value.includes(preset)),
);

const syncTags = (next: string[]) => {
  emit("update:modelValue", next.join("\n"));
};

const addTag = (raw: string) => {
  const value = raw.trim();
  if (!value || tags.value.includes(value)) return;
  syncTags([...tags.value, value]);
};

const removeTag = (index: number) => {
  syncTags(tags.value.filter((_, i) => i !== index));
};

const clearAll = () => {
  syncTags([]);
  draft.value = "";
  focusInput();
};

const commitDraft = () => {
  const parts = parsePrefixes(draft.value);
  if (!parts.length) return;
  const merged = [...tags.value];
  for (const part of parts) {
    if (!merged.includes(part)) merged.push(part);
  }
  syncTags(merged);
  draft.value = "";
};

const focusInput = () => {
  inputRef.value?.focus();
};

const onBlur = () => {
  focused.value = false;
  if (draft.value.trim()) commitDraft();
};

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === "Enter" || event.key === ",") {
    event.preventDefault();
    commitDraft();
    return;
  }
  if (event.key === "Backspace" && !draft.value && tags.value.length) {
    removeTag(tags.value.length - 1);
  }
};

const onPaste = (event: ClipboardEvent) => {
  const text = event.clipboardData?.getData("text") ?? "";
  if (!/[,;\n]/.test(text)) return;
  event.preventDefault();
  const merged = [...tags.value];
  for (const part of parsePrefixes(text)) {
    if (!merged.includes(part)) merged.push(part);
  }
  syncTags(merged);
  draft.value = "";
};
</script>

<style scoped>
.path-prefix-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.tags-box {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: rgba(0, 0, 0, 0.25);
  transition: border-color 0.2s, box-shadow 0.2s;
  cursor: text;
}

.tags-box.focused {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-dim);
}

.tags-scroll {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  max-height: 132px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(34, 211, 238, 0.35) transparent;
}

.tags-scroll::-webkit-scrollbar {
  width: 6px;
}

.tags-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.tags-scroll::-webkit-scrollbar-thumb {
  background: rgba(34, 211, 238, 0.28);
  border-radius: 999px;
}

.tags-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(34, 211, 238, 0.45);
}

.tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 100%;
  padding: 3px 6px 3px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-family: var(--mono);
  color: var(--primary);
  background: var(--primary-dim);
  border: 1px solid var(--border);
}

.tag-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 220px;
}

.tag-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--muted);
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  flex-shrink: 0;
}

.tag-remove:hover {
  color: var(--text);
  background: rgba(255, 255, 255, 0.08);
}

.tag-input {
  flex: 1 1 120px;
  min-width: 120px;
  border: none !important;
  background: transparent !important;
  box-shadow: none !important;
  padding: 4px 2px !important;
  font-family: var(--mono);
  font-size: 12px;
}

.prefix-footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-hint {
  margin: 0;
  font-size: 11px;
  color: var(--muted);
  font-weight: 400;
  line-height: 1.4;
}

.prefix-tools {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex-wrap: wrap;
}

.quick-presets {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  min-width: 0;
}

.presets-label {
  font-size: 11px;
  color: var(--muted);
  flex-shrink: 0;
}

.preset-btn {
  padding: 3px 8px;
  border-radius: 999px;
  border: 1px dashed var(--border-strong);
  background: rgba(0, 0, 0, 0.15);
  color: var(--muted);
  font-size: 11px;
  font-family: var(--mono);
  cursor: pointer;
  transition: color 0.2s, border-color 0.2s, background 0.2s;
}

.preset-btn:hover {
  color: var(--primary);
  border-color: var(--primary);
  background: var(--primary-dim);
}

.clear-btn {
  margin-left: auto;
  flex-shrink: 0;
}
</style>
