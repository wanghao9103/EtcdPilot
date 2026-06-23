<template>
  <div class="json-form" @click.stop>
    <nav class="breadcrumb">
      <button type="button" class="crumb" @click="goTo([])">{{ t("keys.formRoot") }}</button>
      <template v-for="(segment, index) in currentPath" :key="index">
        <span class="sep">/</span>
        <button type="button" class="crumb" @click="goTo(currentPath.slice(0, index + 1))">
          {{ segment }}
        </button>
      </template>
    </nav>

    <div v-if="currentType === 'object'" class="form-body">
      <div v-for="field in objectFields" :key="field.key" class="form-row">
        <span class="field-key mono">{{ field.key }}</span>
        <span class="field-type">{{ field.type }}</span>
        <template v-if="field.isPrimitive">
          <input
            v-if="field.type === 'string'"
            :value="String(field.value ?? '')"
            class="field-input"
            @change="updateField(field.key, ($event.target as HTMLInputElement).value)"
          />
          <input
            v-else-if="field.type === 'number'"
            type="number"
            :value="Number(field.value ?? 0)"
            class="field-input"
            @change="updateField(field.key, Number(($event.target as HTMLInputElement).value))"
          />
          <select
            v-else-if="field.type === 'boolean'"
            :value="String(field.value)"
            class="field-input"
            @change="updateField(field.key, ($event.target as HTMLSelectElement).value === 'true')"
          >
            <option value="true">true</option>
            <option value="false">false</option>
          </select>
          <span v-else class="null-val">null</span>
        </template>
        <button v-else type="button" class="btn-sm" @click="goTo([...currentPath, field.key])">
          {{ t("keys.formDrillIn") }}
        </button>
      </div>
      <div class="add-row">
        <input v-model="newFieldKey" :placeholder="t('keys.formNewKey')" class="field-input" />
        <select v-model="newFieldType" class="type-select">
          <option value="string">{{ t("keys.typeString") }}</option>
          <option value="number">{{ t("keys.typeNumber") }}</option>
          <option value="boolean">{{ t("keys.typeBoolean") }}</option>
          <option value="null">{{ t("keys.typeNull") }}</option>
          <option value="object">{{ t("keys.typeObject") }}</option>
          <option value="array">{{ t("keys.typeArray") }}</option>
        </select>
        <button type="button" class="btn-sm primary" :disabled="!newFieldKey.trim()" @click="addField">
          {{ t("keys.formAddField") }}
        </button>
      </div>
    </div>

    <div v-else-if="currentType === 'array'" class="form-body">
      <div v-for="(item, index) in arrayItems" :key="index" class="form-row">
        <span class="field-key mono">[{{ index }}]</span>
        <span class="field-type">{{ item.type }}</span>
        <template v-if="item.isPrimitive">
          <input
            v-if="item.type === 'string'"
            :value="String(item.value ?? '')"
            class="field-input"
            @change="updateArrayItem(index, ($event.target as HTMLInputElement).value)"
          />
          <input
            v-else-if="item.type === 'number'"
            type="number"
            :value="Number(item.value ?? 0)"
            class="field-input"
            @change="updateArrayItem(index, Number(($event.target as HTMLInputElement).value))"
          />
          <select
            v-else-if="item.type === 'boolean'"
            :value="String(item.value)"
            class="field-input"
            @change="updateArrayItem(index, ($event.target as HTMLSelectElement).value === 'true')"
          >
            <option value="true">true</option>
            <option value="false">false</option>
          </select>
          <span v-else class="null-val">null</span>
        </template>
        <button v-else type="button" class="btn-sm" @click="goTo([...currentPath, String(index)])">
          {{ t("keys.formDrillIn") }}
        </button>
        <button type="button" class="btn-sm danger" @click="removeArrayItem(index)">{{ t("common.delete") }}</button>
      </div>
      <div class="add-row">
        <select v-model="newItemType" class="type-select">
          <option value="string">{{ t("keys.typeString") }}</option>
          <option value="number">{{ t("keys.typeNumber") }}</option>
          <option value="boolean">{{ t("keys.typeBoolean") }}</option>
          <option value="null">{{ t("keys.typeNull") }}</option>
          <option value="object">{{ t("keys.typeObject") }}</option>
          <option value="array">{{ t("keys.typeArray") }}</option>
        </select>
        <button type="button" class="btn-sm primary" @click="addArrayItem">{{ t("keys.formAddItem") }}</button>
      </div>
    </div>

    <p v-else class="form-hint">{{ t("keys.formPrimitiveHint") }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getValueAtPath, setValueAtPath, valueType } from "../utils/jsonPath";

const props = defineProps<{ data: unknown }>();
const emit = defineEmits<{ update: [value: unknown] }>();

const { t } = useI18n();
const currentPath = ref<string[]>([]);
const newFieldKey = ref("");
const newFieldType = ref("string");
const newItemType = ref("string");

const currentValue = computed(() => getValueAtPath(props.data, currentPath.value));
const currentType = computed(() => valueType(currentValue.value));

const defaultForType = (type: string) => {
  if (type === "string") return "";
  if (type === "number") return 0;
  if (type === "boolean") return false;
  if (type === "object") return {};
  if (type === "array") return [];
  return null;
};

const objectFields = computed(() => {
  const obj = currentValue.value;
  if (!obj || typeof obj !== "object" || Array.isArray(obj)) return [];
  return Object.entries(obj as Record<string, unknown>).map(([key, value]) => {
    const type = valueType(value);
    return {
      key,
      value,
      type,
      isPrimitive: type === "string" || type === "number" || type === "boolean" || type === "null",
    };
  });
});

const arrayItems = computed(() => {
  const arr = currentValue.value;
  if (!Array.isArray(arr)) return [];
  return arr.map((value, index) => {
    const type = valueType(value);
    return {
      index,
      value,
      type,
      isPrimitive: type === "string" || type === "number" || type === "boolean" || type === "null",
    };
  });
});

const goTo = (path: string[]) => {
  currentPath.value = path;
};

const patch = (path: string[], value: unknown) => {
  emit("update", setValueAtPath(props.data, path, value));
};

const updateField = (key: string, value: unknown) => {
  const path = [...currentPath.value, key];
  patch(path, value);
};

const updateArrayItem = (index: number, value: unknown) => {
  const path = [...currentPath.value, String(index)];
  patch(path, value);
};

const addField = () => {
  const key = newFieldKey.value.trim();
  if (!key) return;
  patch([...currentPath.value, key], defaultForType(newFieldType.value));
  newFieldKey.value = "";
};

const addArrayItem = () => {
  const arr = Array.isArray(currentValue.value) ? [...currentValue.value] : [];
  arr.push(defaultForType(newItemType.value));
  patch(currentPath.value, arr);
};

const removeArrayItem = (index: number) => {
  const arr = Array.isArray(currentValue.value) ? [...currentValue.value] : [];
  arr.splice(index, 1);
  patch(currentPath.value, arr);
};
</script>

<style scoped>
.json-form {
  padding: 12px;
  max-height: 360px;
  overflow: auto;
}

.breadcrumb {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 12px;
}

.crumb {
  border: none;
  background: transparent;
  color: var(--primary);
  cursor: pointer;
  font-family: var(--mono);
  font-size: 12px;
  padding: 2px 4px;
  border-radius: 4px;
}

.crumb:hover {
  background: var(--primary-dim);
}

.sep {
  color: var(--muted);
}

.form-body {
  display: grid;
  gap: 8px;
}

.form-row {
  display: grid;
  grid-template-columns: minmax(100px, 140px) 70px 1fr auto;
  gap: 8px;
  align-items: center;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: rgba(0, 0, 0, 0.2);
}

.field-key {
  font-size: 12px;
  color: var(--primary);
  word-break: break-all;
}

.field-type {
  font-size: 10px;
  color: var(--muted);
  text-transform: uppercase;
}

.field-input {
  width: 100%;
  min-width: 0;
}

.null-val {
  color: var(--muted);
  font-family: var(--mono);
  font-size: 12px;
}

.add-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px dashed var(--border);
}

.type-select {
  width: 120px;
}

.form-hint {
  margin: 0;
  font-size: 12px;
  color: var(--muted);
}
</style>
