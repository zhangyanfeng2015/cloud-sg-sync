<script setup lang="ts">
import { ref } from "vue";
import { Delete } from "@element-plus/icons-vue";
import PortSelect from "./PortSelect.vue";
import { defaultRule, type Rule, type RuleDirection } from "@/lib/shared";

type RuleRow = Rule & { _id: number };

let idSeq = 0;
function toRow(rule: Rule): RuleRow {
  return {
    port: String(rule.port ?? "").trim(),
    protocol: rule.protocol || "tcp",
    description: rule.description,
    direction: rule.direction === "egress" ? "egress" : "ingress",
    _id: ++idSeq,
  };
}

const rules = ref<RuleRow[]>([toRow(defaultRule("22"))]);

function getRules(): Rule[] {
  return rules.value.map(({ _id: _unused, ...r }) => ({
    port: String(r.port).trim(),
    protocol: "tcp",
    description: r.description.trim() || `auto-whitelist:${r.port}`,
    direction: (r.direction === "egress" ? "egress" : "ingress") as RuleDirection,
  }));
}

function setRules(list: Rule[]) {
  const arr = list.length ? list : [defaultRule("22")];
  rules.value = arr.map(toRow);
}

function addRule() {
  rules.value.push(toRow(defaultRule("22")));
}

function removeRule(index: number) {
  if (rules.value.length > 1) {
    rules.value.splice(index, 1);
  }
}

function onPortChange(row: RuleRow, port: string) {
  if (!port) return;
  if (row.description.startsWith("auto-whitelist:")) {
    row.description = `auto-whitelist:${port}`;
  }
}

defineExpose({ getRules, setRules, addRule });
</script>

<template>
  <div class="rules-editor">
    <div class="rules-editor__head">
      <span class="rules-editor__col rules-editor__col--dir">方向</span>
      <span class="rules-editor__col rules-editor__col--port">端口</span>
      <span class="rules-editor__col rules-editor__col--desc">描述</span>
      <span class="rules-editor__col rules-editor__col--act" aria-hidden="true"></span>
    </div>
    <div
      v-for="(row, index) in rules"
      :key="row._id"
      class="rules-editor__row"
    >
      <div class="rules-editor__col rules-editor__col--dir">
        <el-select
          v-model="row.direction"
          size="small"
          class="rules-editor__dir-select ep-control"
          teleported
          popper-class="rules-editor-popper"
        >
          <el-option label="入站" value="ingress" />
          <el-option label="出站" value="egress" />
        </el-select>
      </div>
      <div class="rules-editor__col rules-editor__col--port">
        <PortSelect
          v-model="row.port"
          @change="(p) => onPortChange(row, p)"
        />
      </div>
      <div class="rules-editor__col rules-editor__col--desc">
        <el-input
          v-model="row.description"
          size="small"
          clearable
          class="ep-control"
        />
      </div>
      <div class="rules-editor__col rules-editor__col--act">
        <el-button
          type="danger"
          :icon="Delete"
          circle
          size="small"
          title="删除"
          :disabled="rules.length <= 1"
          @click="removeRule(index)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.rules-editor {
  width: 100%;
}

.rules-editor__head,
.rules-editor__row {
  display: grid;
  grid-template-columns: 76px minmax(128px, 156px) 1fr 36px;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
}

.rules-editor__head {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-muted);
  background: var(--table-head);
}

.rules-editor__row {
  border-top: 1px solid var(--border);

  &:hover {
    background: var(--bg-hover);
  }
}

.rules-editor__dir-select {
  width: 76px;
  max-width: 76px;
}

.rules-editor__col--act {
  display: flex;
  justify-content: center;
}
</style>
