<template>
  <div class="value-editor" :class="{ invalid: isJsonMode && !validation.valid }" @click.stop>
    <div class="editor-toolbar">
      <span class="status-badge" :class="statusClass">{{ statusLabel }}</span>
      <div class="toolbar-actions">
        <div v-if="showViewTabs" class="view-tabs" @click.stop>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: viewTab === 'code' }"
            @click="setViewTab('code')"
          >
            {{ t("keys.viewCode") }}
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: viewTab === 'tree' }"
            @click="setViewTab('tree')"
          >
            {{ t("keys.viewTree") }}
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: viewTab === 'form' }"
            @click="setViewTab('form')"
          >
            {{ t("keys.viewForm") }}
          </button>
        </div>
        <button v-if="isJsonMode && viewTab === 'code'" type="button" class="btn-sm ghost" @click="formatContent">
          {{ t("keys.formatJson") }}
        </button>
        <button v-if="isJsonMode && viewTab === 'code'" type="button" class="btn-sm ghost" @click="minifyContent">
          {{ t("keys.minifyJson") }}
        </button>
        <button type="button" class="btn-sm ghost" @click="toggleMode">
          {{ isJsonMode ? t("keys.switchPlainText") : t("keys.switchJson") }}
        </button>
      </div>
    </div>

    <template v-if="isJsonMode">
      <div class="json-panels">
        <div class="json-cm-host" :inert="viewTab !== 'code' ? true : undefined">
          <Codemirror
            v-show="viewTab === 'code'"
            :model-value="modelValue"
            :placeholder="placeholder"
            :extensions="extensions"
            :style="{ height: editorHeight }"
            class="json-cm"
            @update:model-value="onCodeInput"
          />
        </div>
        <JsonTreeView
          v-if="structuredData !== null"
          v-show="viewTab === 'tree'"
          :data="structuredData"
          @update="onStructuredUpdate"
        />
        <JsonFormView
          v-if="structuredData !== null"
          v-show="viewTab === 'form'"
          :data="structuredData"
          @update="onStructuredUpdate"
        />
      </div>
    </template>
    <textarea
      v-else
      :value="modelValue"
      :placeholder="placeholder"
      :rows="plainRows"
      class="plain-textarea"
      @input="onPlainInput"
    ></textarea>

    <p v-if="isJsonMode && !validation.valid && validation.error" class="validation-error">
      {{ t("keys.jsonError") }}：{{ validation.error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Codemirror } from "vue-codemirror";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter, lintGutter } from "@codemirror/lint";
import { EditorView } from "@codemirror/view";
import { oneDark } from "@codemirror/theme-one-dark";
import JsonTreeView from "./JsonTreeView.vue";
import JsonFormView from "./JsonFormView.vue";
import { formatJson, looksLikeJson, minifyJson, validateJson } from "../utils/json";
import { isStructuredJson, parseJsonSafe, stringifyJson } from "../utils/jsonPath";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    editorHeight?: string;
    plainRows?: number;
  }>(),
  {
    placeholder: "",
    editorHeight: "220px",
    plainRows: 6,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  validation: [payload: { valid: boolean; isJsonMode: boolean; error?: string }];
}>();

const { t } = useI18n();
const mode = ref<"auto" | "json" | "text">("auto");
const viewTab = ref<"code" | "tree" | "form">("code");
const structuredData = ref<unknown>(null);
const syncingFromStructured = ref(false);

const isJsonMode = computed(() => {
  if (mode.value === "json") return true;
  if (mode.value === "text") return false;
  return looksLikeJson(props.modelValue);
});

const validation = computed(() => {
  if (!isJsonMode.value) return { valid: true };
  return validateJson(props.modelValue);
});

const hasStructuredData = computed(() => structuredData.value !== null && validation.value.valid);

const showViewTabs = computed(
  () => isJsonMode.value && (hasStructuredData.value || viewTab.value !== "code"),
);

const setViewTab = (tab: "code" | "tree" | "form") => {
  viewTab.value = tab;
};

const jsonTextEqual = (a: string, b: string) => {
  if (a === b) return true;
  try {
    return JSON.stringify(JSON.parse(a)) === JSON.stringify(JSON.parse(b));
  } catch {
    return false;
  }
};

const statusClass = computed(() => {
  if (!isJsonMode.value) return "plain";
  return validation.value.valid ? "valid" : "invalid";
});

const statusLabel = computed(() => {
  if (!isJsonMode.value) return t("keys.plainText");
  return validation.value.valid ? t("keys.jsonValid") : t("keys.jsonInvalid");
});

const extensions = shallowRef([
  json(),
  linter(jsonParseLinter()),
  lintGutter(),
  oneDark,
  EditorView.lineWrapping,
  EditorView.theme({
    "&": { backgroundColor: "rgba(0,0,0,0.35)", borderRadius: "8px" },
    ".cm-scroller": { fontFamily: "var(--mono)" },
    ".cm-gutters": { backgroundColor: "rgba(0,0,0,0.45)", border: "none" },
  }),
]);

const syncStructuredFromModel = (raw: string) => {
  if (!validateJson(raw).valid) {
    structuredData.value = null;
    return;
  }
  const parsed = parseJsonSafe(raw);
  structuredData.value = isStructuredJson(parsed) ? parsed : null;
};

const emitValidation = () => {
  emit("validation", {
    valid: validation.value.valid,
    isJsonMode: isJsonMode.value,
    error: validation.value.error,
  });
};

watch(validation, emitValidation, { immediate: true });
watch(isJsonMode, emitValidation);

watch(
  () => props.modelValue,
  (val, oldVal) => {
    if (syncingFromStructured.value) return;
    if (val === oldVal || (oldVal !== undefined && jsonTextEqual(val, oldVal))) return;

    if (mode.value !== "text" && looksLikeJson(val)) {
      mode.value = "json";
      syncStructuredFromModel(val);

      if (viewTab.value === "code") {
        const result = formatJson(val);
        if (result.ok && result.value !== val) {
          emit("update:modelValue", result.value);
        }
      }
      return;
    }

    if (mode.value !== "text" && viewTab.value === "code") {
      structuredData.value = null;
    }
  },
  { immediate: true },
);

const onCodeInput = (value: string) => {
  syncStructuredFromModel(value);
  emit("update:modelValue", value);
};

const onPlainInput = (event: Event) => {
  structuredData.value = null;
  emit("update:modelValue", (event.target as HTMLTextAreaElement).value);
};

const onStructuredUpdate = (value: unknown) => {
  structuredData.value = value;
  const next = stringifyJson(value);
  if (next === props.modelValue || jsonTextEqual(next, props.modelValue)) return;

  syncingFromStructured.value = true;
  emit("update:modelValue", next);
  queueMicrotask(() => {
    syncingFromStructured.value = false;
  });
};

const formatContent = () => {
  const result = formatJson(props.modelValue);
  if (result.ok) {
    emit("update:modelValue", result.value);
    mode.value = "json";
    syncStructuredFromModel(result.value);
  }
};

const minifyContent = () => {
  const result = minifyJson(props.modelValue);
  if (result.ok) {
    emit("update:modelValue", result.value);
    mode.value = "json";
    syncStructuredFromModel(result.value);
  }
};

const toggleMode = () => {
  if (isJsonMode.value) {
    mode.value = "text";
    structuredData.value = null;
    viewTab.value = "code";
    return;
  }
  mode.value = "json";
  if (looksLikeJson(props.modelValue)) {
    formatContent();
  }
};

const validate = () => {
  if (!isJsonMode.value) return { valid: true };
  return validateJson(props.modelValue);
};

const resetMode = () => {
  mode.value = "auto";
  viewTab.value = "code";
  structuredData.value = null;
};

const prepareForContent = () => {
  mode.value = "auto";
};

defineExpose({ validate, resetMode, prepareForContent, formatContent });
</script>

<style scoped>
.value-editor {
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: rgba(0, 0, 0, 0.25);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.value-editor.invalid {
  border-color: rgba(248, 113, 113, 0.5);
  box-shadow: 0 0 0 2px var(--danger-dim);
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.2);
  flex-wrap: wrap;
}

.toolbar-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  align-items: center;
}

.view-tabs {
  display: inline-flex;
  padding: 2px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.25);
  margin-right: 4px;
}

.tab-btn {
  appearance: none;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-family: var(--font);
}

.tab-btn.active {
  color: var(--primary);
  background: var(--primary-dim);
}

.tab-btn:hover:not(.active) {
  color: var(--text);
}

.status-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid var(--border);
}

.status-badge.valid {
  color: var(--success);
  background: var(--success-dim);
  border-color: rgba(74, 222, 128, 0.3);
}

.status-badge.invalid {
  color: var(--danger);
  background: var(--danger-dim);
  border-color: rgba(248, 113, 113, 0.3);
}

.status-badge.plain {
  color: var(--muted);
  background: rgba(255, 255, 255, 0.04);
}

.json-cm {
  font-size: 13px;
}

.json-panels {
  min-height: 120px;
}

.json-cm-host[inert] {
  display: none;
}

.json-cm :deep(.cm-editor) {
  outline: none;
}

.json-cm :deep(.cm-editor.cm-focused) {
  outline: none;
}

.plain-textarea {
  width: 100%;
  border: none;
  border-radius: 0;
  background: rgba(0, 0, 0, 0.35);
  padding: 12px;
  font-family: var(--mono);
  font-size: 12px;
  line-height: 1.6;
  resize: vertical;
  min-height: 120px;
  color: var(--text);
}

.plain-textarea:focus {
  outline: none;
  box-shadow: none;
}

.validation-error {
  margin: 0;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--danger);
  background: var(--danger-dim);
  border-top: 1px solid rgba(248, 113, 113, 0.25);
  word-break: break-word;
}
</style>
