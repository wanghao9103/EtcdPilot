<template>
  <div class="json-tree" @click.stop>
    <ul class="tree-root">
      <JsonTreeNode
        v-for="node in rootNodes"
        :key="node.path"
        :node="node"
        :selected-path="selectedPath"
        :expanded-paths="expanded"
        @select="onSelect"
        @toggle="onToggle"
      />
    </ul>

    <div v-if="selectedPath && selectedIsPrimitive" class="tree-editor">
      <p class="tree-editor-title">
        {{ t("keys.editField") }}: <code class="mono">{{ selectedPath }}</code>
      </p>
      <div class="tree-editor-row">
        <select v-model="editType" class="type-select" @change="onTypeChange">
          <option value="string">{{ t("keys.typeString") }}</option>
          <option value="number">{{ t("keys.typeNumber") }}</option>
          <option value="boolean">{{ t("keys.typeBoolean") }}</option>
          <option value="null">{{ t("keys.typeNull") }}</option>
        </select>
        <input v-if="editType === 'string'" v-model="editString" class="grow" @change="applyPrimitive" />
        <input
          v-else-if="editType === 'number'"
          v-model.number="editNumber"
          type="number"
          class="grow"
          @change="applyPrimitive"
        />
        <select v-else-if="editType === 'boolean'" v-model="editBoolean" class="grow" @change="applyPrimitive">
          <option :value="true">true</option>
          <option :value="false">false</option>
        </select>
        <span v-else class="null-label">null</span>
      </div>
    </div>
    <p v-else-if="selectedPath" class="tree-hint">{{ t("keys.treeComplexHint") }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import JsonTreeNode from "./JsonTreeNode.vue";
import { getValueAtPath, setValueAtPath, valueType } from "../utils/jsonPath";

export interface TreeNodeItem {
  key: string;
  path: string;
  type: string;
  value: unknown;
  children: TreeNodeItem[];
  expandable: boolean;
}

const props = defineProps<{ data: unknown }>();
const emit = defineEmits<{ update: [value: unknown] }>();

const { t } = useI18n();
const expanded = ref<Set<string>>(new Set());
const selectedPath = ref("");

const buildNodes = (value: unknown, parentPath: string, key: string): TreeNodeItem => {
  const path = parentPath ? `${parentPath}.${key}` : key;
  const type = valueType(value);
  const expandable = type === "object" || type === "array";
  let children: TreeNodeItem[] = [];

  if (Array.isArray(value)) {
    children = value.map((item, index) => buildNodes(item, path, String(index)));
  } else if (value && typeof value === "object") {
    children = Object.entries(value as Record<string, unknown>).map(([childKey, childVal]) =>
      buildNodes(childVal, path, childKey),
    );
  }

  return { key, path, type, value, children, expandable };
};

const rootNodes = computed(() => {
  if (Array.isArray(props.data)) {
    return props.data.map((item, index) => buildNodes(item, "", String(index)));
  }
  if (props.data && typeof props.data === "object") {
    return Object.entries(props.data as Record<string, unknown>).map(([key, val]) => buildNodes(val, "", key));
  }
  return [];
});

watch(
  () => props.data,
  (data) => {
    if (expanded.value.size) return;
    if (Array.isArray(data)) {
      expanded.value = new Set(data.map((_, i) => String(i)));
    } else if (data && typeof data === "object") {
      expanded.value = new Set(Object.keys(data as object));
    }
  },
  { immediate: true },
);

const selectedValue = computed(() => {
  if (!selectedPath.value) return undefined;
  return getValueAtPath(props.data, selectedPath.value.split("."));
});

const selectedIsPrimitive = computed(() => {
  const type = valueType(selectedValue.value);
  return type === "string" || type === "number" || type === "boolean" || type === "null";
});

const editType = ref("string");
const editString = ref("");
const editNumber = ref(0);
const editBoolean = ref(true);
const syncingSelection = ref(false);

watch(selectedValue, (val) => {
  syncingSelection.value = true;
  const type = valueType(val);
  if (type === "string" || type === "number" || type === "boolean" || type === "null") {
    editType.value = type;
  }
  if (type === "string") editString.value = String(val ?? "");
  if (type === "number") editNumber.value = Number(val ?? 0);
  if (type === "boolean") editBoolean.value = Boolean(val);
  nextTick(() => {
    syncingSelection.value = false;
  });
});

const onSelect = (path: string) => {
  selectedPath.value = path;
};

const onToggle = (path: string) => {
  const next = new Set(expanded.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  expanded.value = next;
};

const onTypeChange = () => {
  if (syncingSelection.value) return;
  if (editType.value === "null") applyPrimitive();
};

const applyPrimitive = () => {
  if (syncingSelection.value || !selectedPath.value) return;
  let nextValue: unknown = null;
  if (editType.value === "string") nextValue = editString.value;
  else if (editType.value === "number") nextValue = editNumber.value;
  else if (editType.value === "boolean") nextValue = editBoolean.value;
  emit("update", setValueAtPath(props.data, selectedPath.value.split("."), nextValue));
};
</script>

<style scoped>
.json-tree {
  padding: 10px;
  max-height: 360px;
  overflow: auto;
  font-size: 13px;
}

.tree-root {
  margin: 0;
  padding: 0;
}

.tree-editor {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.tree-editor-title {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--muted);
}

.tree-editor-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.type-select {
  width: 110px;
  flex-shrink: 0;
}

.grow {
  flex: 1;
}

.null-label {
  color: var(--muted);
  font-family: var(--mono);
}

.tree-hint {
  margin: 12px 0 0;
  font-size: 12px;
  color: var(--muted);
}
</style>
