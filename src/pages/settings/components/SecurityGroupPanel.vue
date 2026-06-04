<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Refresh } from "@element-plus/icons-vue";
import type { SecurityGroupDetail, SgPermissionRow } from "@/types/security-group";

const loading = ref(false);
const detail = ref<SecurityGroupDetail | null>(null);
const errorMsg = ref("");
const directionFilter = ref<"all" | "ingress" | "egress">("all");

const filteredPermissions = computed(() => {
  const rows = detail.value?.permissions ?? [];
  if (directionFilter.value === "all") return rows;
  return rows.filter(
    (r) => r.direction.toLowerCase() === directionFilter.value,
  );
});

function directionLabel(d: string) {
  return d.toLowerCase() === "egress" ? "出站" : "入站";
}

function cidrForRow(row: SgPermissionRow) {
  if (row.direction.toLowerCase() === "egress") {
    return row.destCidrIp || "—";
  }
  return row.sourceCidrIp || "—";
}

function cidrColumnLabel() {
  return directionFilter.value === "egress" ? "目标地址" : "源地址";
}

async function fetchDetail() {
  loading.value = true;
  errorMsg.value = "";
  try {
    detail.value = await invoke<SecurityGroupDetail>("get_security_group_detail");
  } catch (e) {
    detail.value = null;
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void fetchDetail();
});

defineExpose({ refresh: fetchDetail });
</script>

<template>
  <div class="sg-panel">
    <div class="settings-block__row sg-panel__head">
      <div>
        <h3 class="settings-block__title">云端安全组规则</h3>
        <p class="sg-panel__hint">
          从阿里云拉取当前配置的安全组全部入站 / 出站规则（非仅本工具同步项）。
        </p>
      </div>
      <el-button size="small" :loading="loading" @click="fetchDetail">
        <el-icon class="sg-panel__btn-icon"><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <el-alert
      v-if="errorMsg"
      type="error"
      :title="errorMsg"
      show-icon
      :closable="false"
      class="sg-panel__alert"
    />

    <template v-if="detail && !errorMsg">
      <el-descriptions :column="2" border size="small" class="sg-panel__desc">
        <el-descriptions-item label="地域">{{ detail.regionId }}</el-descriptions-item>
        <el-descriptions-item label="安全组 ID">
          <span class="mono">{{ detail.securityGroupId }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="名称">{{ detail.securityGroupName || "—" }}</el-descriptions-item>
        <el-descriptions-item label="VPC">{{ detail.vpcId || "—" }}</el-descriptions-item>
        <el-descriptions-item label="备注" :span="2">
          {{ detail.innerDescription || "—" }}
        </el-descriptions-item>
        <el-descriptions-item label="规则条数" :span="2">
          {{ detail.permissions.length }} 条
        </el-descriptions-item>
      </el-descriptions>

      <el-tabs v-model="directionFilter" class="sg-panel__tabs">
        <el-tab-pane label="全部" name="all" />
        <el-tab-pane label="入站" name="ingress" />
        <el-tab-pane label="出站" name="egress" />
      </el-tabs>

      <div class="sg-panel__table-wrap">
      <el-table
        :data="filteredPermissions"
        size="small"
        stripe
        border
        class="sg-panel__table"
        empty-text="暂无规则"
        max-height="420"
      >
        <el-table-column label="方向" width="72" align="center">
          <template #default="{ row }">
            <el-tag
              size="small"
              :type="row.direction.toLowerCase() === 'egress' ? 'warning' : 'success'"
              effect="plain"
            >
              {{ directionLabel(row.direction) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="policy" label="策略" width="72" align="center" />
        <el-table-column prop="ipProtocol" label="协议" width="80" align="center" />
        <el-table-column prop="portRange" label="端口" width="110" />
        <el-table-column :label="cidrColumnLabel()" min-width="180">
          <template #default="{ row }">
            <span class="mono">{{ cidrForRow(row) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="description" label="描述" min-width="160" show-overflow-tooltip />
        <el-table-column prop="nicType" label="网卡" width="88" align="center" />
      </el-table>
      </div>
    </template>

    <el-empty
      v-else-if="!loading && !errorMsg"
      description="暂无数据，请先完成同步配置并保存"
    />
  </div>
</template>

<style scoped lang="scss">
.sg-panel {
  padding: 4px 0;
}

.sg-panel__head {
  align-items: flex-start;
  margin-bottom: 14px;
}

.sg-panel__hint {
  margin: 6px 0 0;
  font-size: 0.78rem;
  color: var(--text-dim);
  line-height: 1.45;
}

.sg-panel__btn-icon {
  margin-right: 4px;
}

.sg-panel__alert {
  margin-bottom: 12px;
}

.sg-panel__desc {
  width: 100%;
  margin-bottom: 12px;
}

.sg-panel__tabs {
  margin-bottom: 8px;
}

.sg-panel__tabs :deep(.el-tabs__header) {
  margin-bottom: 0;
}

.sg-panel__table-wrap {
  width: 100%;
  overflow-x: auto;
}

.sg-panel__table {
  width: 100%;
  min-width: 720px;
}

.mono {
  font-family: var(--font-mono);
  font-size: 0.82rem;
  word-break: break-all;
}
</style>
