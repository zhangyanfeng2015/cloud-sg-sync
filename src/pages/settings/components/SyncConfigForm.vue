<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import DryRunDialog from "./DryRunDialog.vue";
import RulesEditor from "./RulesEditor.vue";
import {
  defaultRule,
  normalizePollIntervalSecs,
  parsePortInput,
  POLL_INTERVAL_OPTIONS,
  setMsg,
  type AppConfig,
  type Rule,
  type RuleDirection,
} from "@/lib/shared";

const accessKeyId = ref("");
const accessKeySecret = ref("");
const secretPlaceholder = ref("请输入 AccessKey Secret");

const regionId = ref("");
const securityGroupId = ref("");
const regions = ref<{ regionId: string; localName: string }[]>([]);
const securityGroups = ref<
  { securityGroupId: string; securityGroupName: string }[]
>([]);
const regionDisabled = ref(true);
const sgLoading = ref(false);

const pollIntervalSecs = ref(300);
const monitoringEnabled = ref(true);

const testMsg = ref("");
const testMsgKind = ref<"success" | "error" | "info">("info");

const rulesEditorRef = ref<InstanceType<typeof RulesEditor> | null>(null);
const dryRunRef = ref<InstanceType<typeof DryRunDialog> | null>(null);
const hasStoredSecret = ref(false);

const emit = defineEmits<{ viewSg: [] }>();

function sgOptionLabel(g: { securityGroupId: string; securityGroupName: string }) {
  const name = g.securityGroupName?.trim();
  return name ? `${name} (${g.securityGroupId})` : g.securityGroupId;
}

function setTestMsg(text: string, kind: "success" | "error" | "info" = "info") {
  testMsg.value = text;
  testMsgKind.value = kind;
}

function getRules(): Rule[] {
  return rulesEditorRef.value?.getRules() ?? [];
}

function setRules(rules: Rule[]) {
  const list = rules.length ? rules : [defaultRule("22")];
  rulesEditorRef.value?.setRules(
    list.map((r) => ({
      ...r,
      direction: (r.direction === "egress" ? "egress" : "ingress") as RuleDirection,
    })),
  );
}

function addRule() {
  rulesEditorRef.value?.addRule();
}

async function loadSecurityGroups(selectedSg?: string) {
  if (!regionId.value) return;
  sgLoading.value = true;
  try {
    const groups = await invoke<
      { securityGroupId: string; securityGroupName: string }[]
    >("list_security_groups", { regionId: regionId.value });
    securityGroups.value = groups;
    if (selectedSg) {
      securityGroupId.value = selectedSg;
    } else if (!securityGroupId.value && groups[0]) {
      securityGroupId.value = groups[0].securityGroupId;
    }
  } finally {
    sgLoading.value = false;
  }
}

async function refreshRegions(selectedRegion?: string, selectedSg?: string) {
  const list = await invoke<{ regionId: string; localName: string }[]>(
    "list_regions",
  );
  regions.value = list;
  regionDisabled.value = false;
  if (selectedRegion) {
    regionId.value = selectedRegion;
  } else if (!regionId.value && list[0]) {
    regionId.value = list[0].regionId;
  }
  if (regionId.value) {
    await loadSecurityGroups(selectedSg);
  }
}

function loadForm(data: {
  config: AppConfig;
  accessKeyId?: string | null;
  hasSecret: boolean;
}) {
  const { config, hasSecret } = data;
  hasStoredSecret.value = hasSecret;
  accessKeyId.value = data.accessKeyId || "";
  accessKeySecret.value = "";
  secretPlaceholder.value = hasSecret
    ? "已保存，留空则不修改"
    : "请输入 AccessKey Secret";

  pollIntervalSecs.value = normalizePollIntervalSecs(
    config.pollIntervalSecs || 300,
  );
  monitoringEnabled.value = config.monitoringEnabled;
  regionId.value = config.regionId || "";
  securityGroupId.value = config.securityGroupId || "";
  setRules(config.rules);

  regionDisabled.value = !hasSecret;
  if (!hasSecret) {
    regions.value = [];
    securityGroups.value = [];
  }
}

function readAk() {
  return {
    id: accessKeyId.value.trim(),
    secret: accessKeySecret.value.trim(),
  };
}

/** 同步配置字段（主题、自启由设置页其它区域合并） */
function collectSyncFields(): Pick<
  AppConfig,
  | "version"
  | "regionId"
  | "securityGroupId"
  | "pollIntervalSecs"
  | "monitoringEnabled"
  | "ipProbeUrls"
  | "rules"
> {
  return {
    version: 1,
    regionId: regionId.value,
    securityGroupId: securityGroupId.value.trim(),
    pollIntervalSecs: pollIntervalSecs.value,
    monitoringEnabled: monitoringEnabled.value,
    ipProbeUrls: ["https://api.ipify.org", "https://ifconfig.me/ip"],
    rules: getRules(),
  };
}

async function exportConfig() {
  try {
    const path = await save({
      filters: [{ name: "Cloud SG Sync", extensions: ["agsync"] }],
      defaultPath: "cloud-sg-sync.agsync",
    });
    if (!path) return;
    await invoke("export_config", { path });
    setMsg("settingsMsg", `配置与操作记录已导出到：${path}`, "success");
  } catch (e) {
    setMsg("settingsMsg", String(e), "error");
  }
}

async function importConfig() {
  try {
    const path = await open({
      filters: [{ name: "Cloud SG Sync", extensions: ["agsync"] }],
      multiple: false,
    });
    if (!path || Array.isArray(path)) return;
    await invoke("import_config", { path });
    setMsg("settingsMsg", "配置与操作记录已导入", "success");
    window.dispatchEvent(new CustomEvent("sync-config-imported"));
  } catch (e) {
    setMsg("settingsMsg", String(e), "error");
  }
}

function validateRules(): string | null {
  for (const rule of getRules()) {
    if (!parsePortInput(rule.port)) {
      return "请填写有效的端口（1–65535）";
    }
  }
  return null;
}

async function testConnection() {
  const { id, secret } = readAk();
  if (!id) {
    setTestMsg("请填写 AccessKey ID", "error");
    return;
  }
  if (!secret && !hasStoredSecret.value) {
    setTestMsg("请填写 AccessKey Secret", "error");
    return;
  }
  setTestMsg("测试中…", "info");
  try {
    await invoke("test_connection", {
      accessKeyId: id,
      accessKeySecret: secret || null,
    });
    setTestMsg("连接成功", "success");
    hasStoredSecret.value = true;
    regionDisabled.value = false;
    const saved = await invoke<{ config: AppConfig; hasSecret: boolean }>(
      "get_config",
    );
    if (!saved.hasSecret) {
      await invoke("save_config", {
        payload: {
          config: {
            ...saved.config,
            ...collectSyncFields(),
            regionId: "",
            securityGroupId: "",
          },
          accessKeyId: id,
          accessKeySecret: secret,
        },
      });
    }
    await refreshRegions();
  } catch (e) {
    setTestMsg(String(e), "error");
  }
}

watch(regionId, async (id, prev) => {
  if (id && id !== prev) {
    await loadSecurityGroups();
  }
});

defineExpose({
  loadForm,
  readAk,
  collectSyncFields,
  exportConfig,
  importConfig,
  getRules,
  setRules,
  addRule,
  validateRules,
  refreshRegions,
  loadSecurityGroups,
  testConnection,
  regionId,
  securityGroupId,
});
</script>

<template>
  <div class="sync-config-form">
    <section class="sync-section">
      <h3 class="settings-block__title">凭证</h3>
        <div class="field-grid field-grid--2">
          <div class="ep-field">
            <span class="field-label">AccessKey ID</span>
            <el-input
              v-model="accessKeyId"
              class="ep-control"
              autocomplete="off"
              placeholder="LTAI…"
              clearable
            />
          </div>
          <div class="ep-field">
            <span class="field-label">AccessKey Secret</span>
            <el-input
              v-model="accessKeySecret"
              class="ep-control"
              type="password"
              show-password
              :placeholder="secretPlaceholder"
              autocomplete="new-password"
            />
          </div>
        </div>
        <div class="settings-block__actions settings-block__actions--wrap">
          <el-button size="small" @click="testConnection">测试连接</el-button>
          <el-button size="small" @click="exportConfig">导出配置</el-button>
          <el-button size="small" @click="importConfig">导入配置</el-button>
          <p
            v-if="testMsg"
            class="inline-msg"
            :class="{
              'msg-success': testMsgKind === 'success',
              'msg-error': testMsgKind === 'error',
              'msg-info': testMsgKind === 'info',
            }"
            role="status"
          >
            {{ testMsg }}
          </p>
        </div>
    </section>

    <el-divider />

    <section class="sync-section">
      <h3 class="settings-block__title">地域与安全组</h3>
        <div class="field-grid field-grid--2">
          <div class="ep-field">
            <span class="field-label">地域</span>
            <el-select
              v-model="regionId"
              class="ep-control"
              size="small"
              filterable
              placeholder="请先测试连接"
              :disabled="regionDisabled"
              teleported
              popper-class="sync-config-popper"
            >
              <el-option
                v-for="r in regions"
                :key="r.regionId"
                :label="`${r.localName} (${r.regionId})`"
                :value="r.regionId"
              />
            </el-select>
          </div>
          <div class="ep-field">
            <span class="field-label">安全组</span>
            <div class="sg-field">
              <el-select
                v-model="securityGroupId"
                class="ep-control ep-control--grow"
                size="small"
                filterable
                allow-create
                default-first-option
                :reserve-keyword="false"
                placeholder="选择或输入 sg-xxx"
                :disabled="!regionId"
                :loading="sgLoading"
                teleported
                popper-class="sync-config-popper"
              >
                <el-option
                  v-for="g in securityGroups"
                  :key="g.securityGroupId"
                  :label="sgOptionLabel(g)"
                  :value="g.securityGroupId"
                />
              </el-select>
              <el-button
                size="small"
                :disabled="!regionId"
                :loading="sgLoading"
                @click="loadSecurityGroups()"
              >
                加载列表
              </el-button>
            </div>
          </div>
        </div>
        <p class="sync-sg-link">
          <el-button
            type="primary"
            link
            size="small"
            :disabled="!regionId || !securityGroupId"
            @click="emit('viewSg')"
          >
            查看云端安全组全部规则 →
          </el-button>
        </p>
    </section>

    <el-divider />

    <section class="sync-section">
      <div class="settings-block__row">
        <h3 class="settings-block__title">端口规则</h3>
          <el-button size="small" text type="primary" @click="addRule">
            + 添加端口
          </el-button>
        </div>
      <div class="rules-wrap">
        <RulesEditor ref="rulesEditorRef" />
      </div>
    </section>

    <el-divider />

    <section class="sync-section">
      <div class="settings-block__row">
        <h3 class="settings-block__title">变更预览</h3>
        <el-button size="small" @click="dryRunRef?.open()">预览变更</el-button>
      </div>
      <p class="field-hint">
        预览将探测当前公网 IP，对比上次同步记录与云端规则，并说明各端口的撤销/授权/跳过步骤（不会写入云端）。
      </p>
    </section>

    <el-divider />

    <section class="sync-section">
      <h3 class="settings-block__title">监听</h3>
        <div class="monitor-row">
          <span class="field-label monitor-row__label">轮询间隔</span>
          <el-select
            v-model="pollIntervalSecs"
            class="ep-control monitor-row__select"
            size="small"
            teleported
            popper-class="sync-config-popper"
          >
            <el-option
              v-for="o in POLL_INTERVAL_OPTIONS"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
          <div class="monitor-row__switch">
            <el-switch v-model="monitoringEnabled" size="small" />
            <span class="monitor-switch__label">记住监听状态</span>
          </div>
        </div>
    </section>

    <DryRunDialog ref="dryRunRef" />
  </div>
</template>

<style scoped lang="scss">
.sync-section {
  scroll-margin-top: 72px;
}

.ep-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.ep-control {
  width: 100%;
}

.ep-control--grow {
  flex: 1;
  min-width: 0;
}

.sg-field {
  display: flex;
  gap: 8px;
  align-items: center;
}

.sync-sg-link {
  margin: 8px 0 0;
}

.monitor-row {
  display: grid;
  grid-template-columns: minmax(180px, 280px) auto;
  grid-template-rows: auto auto;
  column-gap: 24px;
  row-gap: 6px;
  align-items: center;
}

.monitor-row__label {
  grid-column: 1;
  grid-row: 1;
}

.monitor-row__select {
  grid-column: 1;
  grid-row: 2;
  width: 100%;
}

.monitor-row__switch {
  grid-column: 2;
  grid-row: 2;
  display: flex;
  align-items: center;
  gap: 10px;
  align-self: center;
}

.monitor-switch__label {
  font-size: 0.88rem;
  color: var(--text-muted);
  white-space: nowrap;
}

.field--grow {
  flex: 1;
  min-width: 180px;
}

@media (max-width: 560px) {
  .monitor-row {
    grid-template-columns: 1fr;
  }

  .monitor-row__switch {
    grid-column: 1;
    grid-row: 3;
  }
}
</style>
