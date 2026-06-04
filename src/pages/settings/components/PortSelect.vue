<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import type { ElSelect } from "element-plus";
import {
  PORT_OPTION_GROUPS,
  isPresetPort,
  type PortOption,
} from "@/lib/port-presets";
import { parsePortInput } from "@/lib/shared";

const model = defineModel<string>({ required: true });

const emit = defineEmits<{
  change: [port: string];
}>();

const selectRef = ref<InstanceType<typeof ElSelect> | null>(null);

const customOption = computed((): PortOption | null => {
  const raw = String(model.value ?? "").trim();
  if (!raw || isPresetPort(raw)) return null;
  const p = parsePortInput(raw);
  return { value: p ?? raw, service: "自定义" };
});

async function onChange(val: string | number | undefined) {
  const raw = String(val ?? "").trim();
  const p = parsePortInput(raw) || raw;
  model.value = p;
  await nextTick();
  emit("change", p);
}

function onVisibleChange(open: boolean) {
  if (!open) {
    selectRef.value?.blur();
  }
}

function optionClass(opt: PortOption) {
  return {
    "port-select__option": true,
    "port-select__option--custom": opt.service === "自定义",
  };
}
</script>

<template>
  <el-select
    ref="selectRef"
    v-model="model"
    class="port-select ep-control"
    size="small"
    filterable
    allow-create
    default-first-option
    clearable
    :reserve-keyword="false"
    placeholder="选择或输入端口"
    teleported
    popper-class="port-select-popper"
    @change="onChange"
    @visible-change="onVisibleChange"
  >
    <el-option-group
      v-for="group in PORT_OPTION_GROUPS"
      :key="group.label"
      :label="group.label"
    >
      <el-option
        v-for="opt in group.options"
        :key="opt.value"
        :label="opt.value"
        :value="opt.value"
      >
        <div :class="optionClass(opt)">
          <span class="port-select__option-port">{{ opt.value }}</span>
          <span class="port-select__option-svc">{{ opt.service }}</span>
        </div>
      </el-option>
    </el-option-group>

    <el-option
      v-if="customOption"
      :key="`custom-${customOption.value}`"
      :label="customOption.value"
      :value="customOption.value"
    >
      <div :class="optionClass(customOption)">
        <span class="port-select__option-port">{{ customOption.value }}</span>
        <span class="port-select__option-svc">{{ customOption.service }}</span>
      </div>
    </el-option>
  </el-select>
</template>

<style scoped lang="scss">
.port-select {
  width: 100%;
}
</style>
