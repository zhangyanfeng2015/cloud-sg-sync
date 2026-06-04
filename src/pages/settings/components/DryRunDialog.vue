<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface SyncPlanAction {
  action: string;
  direction: string;
  portRange: string;
  ipProtocol: string;
  cidr: string;
  description: string;
  detail: string;
}

export interface DryRunResult {
  currentIp: string;
  lastSyncIp?: string | null;
  ipChanged: boolean;
  summary: string;
  actions: SyncPlanAction[];
}

const visible = ref(false);
const loading = ref(false);
const error = ref("");
const result = ref<DryRunResult | null>(null);

const actionMeta: Record<
  string,
  { label: string; tag: "success" | "warning" | "info" | "danger" }
> = {
  revoke: { label: "撤销", tag: "warning" },
  authorize: { label: "授权", tag: "success" },
  skip: { label: "跳过", tag: "info" },
  unchanged: { label: "无需变更", tag: "success" },
};

const directionLabel: Record<string, string> = {
  ingress: "入站",
  egress: "出站",
};

function actionLabel(action: string) {
  return actionMeta[action]?.label ?? action;
}

function actionTagType(action: string) {
  return actionMeta[action]?.tag ?? "info";
}

async function open() {
  visible.value = true;
  loading.value = true;
  error.value = "";
  result.value = null;
  try {
    result.value = await invoke<DryRunResult>("dry_run_sync");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

defineExpose({ open });
</script>

<template>
  <el-dialog
    v-model="visible"
    title="预览同步变更"
    width="780px"
    destroy-on-close
    class="dry-run-dialog"
  >
    <p v-if="loading" class="dry-run-hint">加载中…</p>
    <p v-else-if="error" class="dry-run-error">{{ error }}</p>
    <template v-else-if="result">
      <section class="dry-run-summary">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="当前公网 IP">
            <span class="mono">{{ result.currentIp }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="上次同步 IP">
            <span class="mono">{{ result.lastSyncIp || "（尚未同步）" }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="IP 是否变化" :span="2">
            <el-tag :type="result.ipChanged ? 'warning' : 'success'" size="small">
              {{ result.ipChanged ? "是" : "否" }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
        <p class="dry-run-summary__text">{{ result.summary }}</p>
      </section>

      <p class="dry-run-legend__note">以下为预览结果，不会写入云端；完整规则请在「安全组规则」中查看。</p>

      <section v-if="result.actions.length" class="dry-run-table-wrap">
        <h4 class="dry-run-table-wrap__title">计划</h4>
        <el-table :data="result.actions" size="small" stripe max-height="280">
          <el-table-column label="操作" width="100">
            <template #default="{ row }">
              <el-tag :type="actionTagType(row.action)" size="small">
                {{ actionLabel(row.action) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="说明" min-width="220">
            <template #default="{ row }">
              <span class="dry-run-detail">{{ row.detail }}</span>
            </template>
          </el-table-column>
          <el-table-column label="方向" width="64">
            <template #default="{ row }">
              {{ directionLabel[row.direction] ?? row.direction }}
            </template>
          </el-table-column>
          <el-table-column prop="portRange" label="端口" width="80" />
          <el-table-column prop="cidr" label="CIDR" min-width="130" />
        </el-table>
      </section>
      <p v-else class="dry-run-hint">无端口规则可预览。</p>
    </template>
    <template #footer>
      <el-button @click="visible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.dry-run-hint {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.88rem;
}

.dry-run-error {
  margin: 0;
  color: var(--danger, #e74c3c);
  font-size: 0.88rem;
}

.dry-run-summary {
  margin-bottom: 16px;
}

.dry-run-summary__text {
  margin: 10px 0 0;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--text);
}

.mono {
  font-family: ui-monospace, Consolas, monospace;
  font-size: 0.85rem;
}

.dry-run-legend__note {
  margin: 0 0 12px;
  font-size: 0.78rem;
  color: var(--text-muted);
}

.dry-run-table-wrap__title {
  margin: 0 0 8px;
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text-muted);
}

.dry-run-detail {
  font-size: 0.84rem;
  line-height: 1.4;
}
</style>
