<template>
  <li class="tree-node">
    <div class="tree-row" :class="{ selected: selectedPath === node.path }" @click="onRowClick">
      <button
        v-if="node.expandable"
        type="button"
        class="toggle"
        @click.stop="emit('toggle', node.path)"
      >
        {{ expanded ? "▼" : "▶" }}
      </button>
      <span v-else class="toggle spacer"></span>
      <span class="node-key mono">{{ node.key }}</span>
      <span class="node-type">{{ node.type }}</span>
      <span v-if="!node.expandable" class="node-preview mono">{{ previewValue(node.value) }}</span>
    </div>
    <ul v-if="node.expandable && expanded" class="tree-children">
      <JsonTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :selected-path="selectedPath"
        :expanded-paths="expandedPaths"
        @select="emit('select', $event)"
        @toggle="emit('toggle', $event)"
      />
    </ul>
  </li>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { TreeNodeItem } from "./JsonTreeView.vue";
import { previewValue } from "../utils/jsonPath";

const props = defineProps<{
  node: TreeNodeItem;
  selectedPath: string;
  expandedPaths: Set<string>;
}>();

const emit = defineEmits<{
  select: [path: string];
  toggle: [path: string];
}>();

const expanded = computed(() => props.expandedPaths.has(props.node.path));

const onRowClick = () => {
  emit("select", props.node.path);
};
</script>

<style scoped>
.tree-node {
  list-style: none;
}

.tree-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.tree-row:hover {
  background: rgba(34, 211, 238, 0.06);
}

.tree-row.selected {
  background: var(--primary-dim);
  box-shadow: inset 0 0 0 1px var(--border-strong);
}

.toggle {
  width: 18px;
  height: 18px;
  border: none;
  background: transparent;
  color: var(--muted);
  font-size: 10px;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
}

.toggle.spacer {
  display: inline-block;
}

.node-key {
  color: var(--primary);
  font-weight: 500;
}

.node-type {
  font-size: 10px;
  color: var(--muted);
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.05);
  text-transform: uppercase;
}

.node-preview {
  color: var(--muted);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.tree-children {
  margin: 0;
  padding-left: 18px;
}
</style>
