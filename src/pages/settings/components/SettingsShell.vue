<script setup lang="ts">
import { ref } from "vue";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import { Setting, Brush, InfoFilled, Tickets } from "@element-plus/icons-vue";
import SyncConfigForm from "./SyncConfigForm.vue";
import SecurityGroupPanel from "./SecurityGroupPanel.vue";
import AppearancePanel from "./AppearancePanel.vue";
import AboutPanel from "./AboutPanel.vue";
import { normalizeAccent, type ThemeAccent, type ThemeMode } from "@/app/theme";
import type { AppConfig } from "@/lib/shared";
import type { SettingsPanelId } from "@/types/settings";

const locale = zhCn;
const activePanel = ref<SettingsPanelId>("sync");
const scrollRef = ref<HTMLElement | null>(null);
const autoStartWindows = ref(false);
const themeMode = ref<ThemeMode>("dark");
const themeAccent = ref<ThemeAccent>("sky");

const syncFormRef = ref<InstanceType<typeof SyncConfigForm> | null>(null);
const appearanceRef = ref<InstanceType<typeof AppearancePanel> | null>(null);

function onMenuSelect(index: string) {
  activePanel.value = index as SettingsPanelId;
  emit("panelChange", activePanel.value);
}

const emit = defineEmits<{
  panelChange: [panel: SettingsPanelId];
}>();

function resetToSync() {
  activePanel.value = "sync";
  emit("panelChange", "sync");
}

function navigateToSg() {
  activePanel.value = "sg";
  emit("panelChange", "sg");
}

function loadForm(data: {
  config: AppConfig;
  accessKeyId?: string | null;
  hasSecret: boolean;
}) {
  const { config, hasSecret } = data;
  syncFormRef.value?.loadForm({
    config,
    hasSecret,
    accessKeyId: data.accessKeyId,
  });
  const mode = (config.themeMode || "dark") as ThemeMode;
  const accent = normalizeAccent(config.themeAccent);
  themeMode.value = mode;
  themeAccent.value = accent;
  appearanceRef.value?.loadAppearance(mode, accent);
  autoStartWindows.value = config.autoStartWindows;
}

function collectSyncFields() {
  return syncFormRef.value!.collectSyncFields();
}

function collectConfig(): AppConfig {
  const sync = collectSyncFields();
  return {
    ...sync,
    autoStartWindows: autoStartWindows.value,
    themeMode: themeMode.value,
    themeAccent: themeAccent.value,
  };
}

defineExpose({
  resetToSync,
  navigateToSg,
  loadForm,
  collectSyncFields,
  collectConfig,
  readAk: () => syncFormRef.value!.readAk(),
  validateRules: () => syncFormRef.value!.validateRules(),
  refreshRegions: (r?: string, sg?: string) =>
    syncFormRef.value!.refreshRegions(r, sg),
});
</script>

<template>
  <el-config-provider :locale="locale">
    <div class="settings-shell ep-form-root">
      <aside class="settings-shell__aside">
        <el-menu
          :key="activePanel"
          :default-active="activePanel"
          class="settings-shell__menu"
          @select="onMenuSelect"
        >
          <el-menu-item index="sync">
            <el-icon><Setting /></el-icon>
            <span>同步配置</span>
          </el-menu-item>
          <el-menu-item index="sg">
            <el-icon><Tickets /></el-icon>
            <span>安全组规则</span>
          </el-menu-item>
          <el-menu-item index="appearance">
            <el-icon><Brush /></el-icon>
            <span>外观</span>
          </el-menu-item>
          <el-menu-item index="about">
            <el-icon><InfoFilled /></el-icon>
            <span>关于</span>
          </el-menu-item>
        </el-menu>
      </aside>

      <div class="settings-shell__main">
        <div ref="scrollRef" class="settings-shell__scroll">
          <div class="card settings-card">
            <!-- v-show：切换菜单时不销毁表单，避免凭证等地域配置丢失 -->
            <SyncConfigForm
              v-show="activePanel === 'sync'"
              ref="syncFormRef"
              class="settings-panel"
              :class="{ 'settings-panel--hidden': activePanel !== 'sync' }"
              @view-sg="navigateToSg"
            />
            <SecurityGroupPanel
              v-if="activePanel === 'sg'"
              class="settings-panel"
            />
            <AppearancePanel
              v-else-if="activePanel === 'appearance'"
              ref="appearanceRef"
              class="settings-panel"
              v-model:theme-mode="themeMode"
              v-model:theme-accent="themeAccent"
            />
            <AboutPanel
              v-else-if="activePanel === 'about'"
              class="settings-panel"
              v-model:auto-start-windows="autoStartWindows"
            />
          </div>
        </div>
      </div>
    </div>
  </el-config-provider>
</template>

<style scoped lang="scss">
.settings-shell {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 168px 1fr;
  gap: 18px;
  align-items: stretch;
}

.settings-shell__aside {
  min-height: 0;
}

.settings-shell__menu {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elevated);
  padding: 6px;
}

.settings-shell__menu :deep(.el-menu-item) {
  border-radius: var(--radius-sm);
  margin-bottom: 2px;
  height: 40px;
}

.settings-shell__menu :deep(.el-menu-item.is-active) {
  background: var(--accent-subtle);
  color: var(--text);
  font-weight: 600;
}

.settings-shell__main {
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.settings-shell__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-right: 6px;
  scrollbar-width: thin;
  scrollbar-color: var(--scrollbar) transparent;
}

.settings-panel--hidden {
  display: none;
}

@media (max-width: 720px) {
  .settings-shell {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
  }

  .settings-shell__menu {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
  }

  .settings-shell__menu :deep(.el-menu-item) {
    flex: 1;
    min-width: 96px;
    justify-content: center;
  }
}
</style>
