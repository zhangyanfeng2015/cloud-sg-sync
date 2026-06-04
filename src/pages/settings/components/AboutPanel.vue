<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { renderChangelogList } from "@/lib/changelog";
import type { AppInfo, UpdateCheck } from "@/lib/meta";

const autoStartWindows = defineModel<boolean>("autoStartWindows", {
  required: true,
});

const appName = ref("—");
const appVersion = ref("—");
const appDesc = ref("—");
const appDataDir = ref("—");
const updateMsg = ref("");
const updateMsgKind = ref<"success" | "error" | "info">("info");
const DEFAULT_RELEASE_URL =
  "https://github.com/zhangyanfeng2015/cloud-sg-sync/releases";
const releaseUrl = ref(DEFAULT_RELEASE_URL);

function setUpdateMsg(text: string, kind: "success" | "error" | "info" = "info") {
  updateMsg.value = text;
  updateMsgKind.value = kind;
}

async function loadAbout() {
  const info = await invoke<AppInfo>("get_app_info");
  appName.value = info.name;
  appVersion.value = `v${info.version}`;
  appDesc.value = info.description;
  appDataDir.value = info.dataDirHint;
}

async function onCheckUpdate() {
  setUpdateMsg("正在检查…", "info");
  try {
    const res = await invoke<UpdateCheck>("check_for_updates");
    appVersion.value = `v${res.currentVersion}`;
    releaseUrl.value = res.releaseUrl;
    setUpdateMsg(res.message, res.hasUpdate ? "success" : "info");
  } catch (e) {
    setUpdateMsg(String(e), "error");
  }
}

async function openRelease() {
  if (releaseUrl.value) await openUrl(releaseUrl.value);
}

let autoStartReady = false;

watch(autoStartWindows, (enabled) => {
  if (!autoStartReady) return;
  void invoke("set_auto_start_windows", { enabled }).catch(console.error);
});

onMounted(() => {
  void loadAbout();
  renderChangelogList("changelogList");
  autoStartReady = true;
});
</script>

<template>
  <div class="about-panel">
    <el-descriptions :column="1" border size="small" class="about-desc">
      <el-descriptions-item label="应用">{{ appName }}</el-descriptions-item>
      <el-descriptions-item label="版本">
        <span class="mono">{{ appVersion }}</span>
      </el-descriptions-item>
      <el-descriptions-item label="说明">{{ appDesc }}</el-descriptions-item>
      <el-descriptions-item label="数据目录">
        <span class="mono kv-path">{{ appDataDir }}</span>
      </el-descriptions-item>
    </el-descriptions>

    <el-divider />

    <div class="about-row">
      <span class="about-row__label">开机自启</span>
      <el-switch v-model="autoStartWindows" size="small" />
    </div>

    <el-divider />

    <div class="settings-block">
      <div class="about-update-head">
        <span class="field-label">更新</span>
        <span class="mono about-version">{{ appVersion }}</span>
      </div>
      <div class="settings-block__actions settings-block__actions--wrap">
        <el-button size="small" @click="onCheckUpdate">检查更新</el-button>
        <el-button size="small" @click="openRelease">发布页</el-button>
      </div>
      <p
        v-if="updateMsg"
        class="inline-msg"
        :class="{
          'msg-success': updateMsgKind === 'success',
          'msg-error': updateMsgKind === 'error',
          'msg-info': updateMsgKind === 'info',
        }"
        role="status"
      >
        {{ updateMsg }}
      </p>
    </div>

    <el-divider />

    <div id="changelogList" class="changelog-list" aria-label="版本更新记录" />
  </div>
</template>

<style scoped lang="scss">
.about-panel {
  padding: 4px 0;
}

.about-desc {
  width: 100%;
}

.kv-path {
  word-break: break-all;
  font-size: 0.78rem;
}

.about-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.about-row__label {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--text);
}

.about-update-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
}

.about-version {
  font-size: 0.85rem;
  color: var(--text-muted);
}
</style>
