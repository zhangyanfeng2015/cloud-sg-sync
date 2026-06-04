<script setup lang="ts">
import { watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  ACCENT_OPTIONS,
  applyTheme,
  normalizeAccent,
  type ThemeAccent,
  type ThemeMode,
} from "@/app/theme";
import { setMsg } from "@/lib/shared";

const themeMode = defineModel<ThemeMode>("themeMode", { default: "dark" });
const themeAccent = defineModel<ThemeAccent>("themeAccent", { default: "sky" });

let skipPersist = false;
let toastTimer: ReturnType<typeof setTimeout> | null = null;

const MODE_LABELS: Record<ThemeMode, string> = {
  dark: "深色模式",
  light: "浅色模式",
  system: "跟随系统",
};

function accentTitle(id: ThemeAccent): string {
  return ACCENT_OPTIONS.find((a) => a.id === id)?.title ?? id;
}

function showToolbarToast(text: string, kind: "success" | "error" | "info" = "info") {
  if (toastTimer) {
    clearTimeout(toastTimer);
    toastTimer = null;
  }
  setMsg("settingsMsg", text, kind);
  if (kind !== "error" && text) {
    toastTimer = setTimeout(() => {
      setMsg("settingsMsg", "", "info");
      toastTimer = null;
    }, 2200);
  }
}

function loadAppearance(mode: ThemeMode, accent: ThemeAccent) {
  skipPersist = true;
  themeMode.value = mode;
  themeAccent.value = normalizeAccent(accent);
  applyTheme(themeMode.value, themeAccent.value);
  skipPersist = false;
}

async function persistAppearance(kind: "mode" | "accent") {
  if (skipPersist) return;
  try {
    applyTheme(themeMode.value, themeAccent.value);
    await invoke("save_theme", {
      themeMode: themeMode.value,
      themeAccent: themeAccent.value,
    });
    const text =
      kind === "mode"
        ? `已切换为${MODE_LABELS[themeMode.value]}`
        : `强调色已切换为${accentTitle(themeAccent.value)}`;
    showToolbarToast(text, "success");
  } catch (e) {
    showToolbarToast(String(e), "error");
  }
}

watch(themeMode, () => void persistAppearance("mode"));
watch(themeAccent, () => void persistAppearance("accent"));

defineExpose({
  loadAppearance,
  getThemeMode: () => themeMode.value,
  getThemeAccent: () => themeAccent.value,
});
</script>

<template>
  <div class="appearance-panel">
    <div class="settings-block">
      <span class="field-label">外观模式</span>
      <el-radio-group v-model="themeMode" class="segmented-radio-group" size="small">
        <el-radio-button value="dark">深色</el-radio-button>
        <el-radio-button value="light">浅色</el-radio-button>
        <el-radio-button value="system">跟随系统</el-radio-button>
      </el-radio-group>
    </div>

    <el-divider />

    <div class="settings-block">
      <span class="field-label">强调色</span>
      <el-radio-group
        v-model="themeAccent"
        class="segmented-radio-group accent-radio-group"
        size="small"
      >
        <el-radio-button
          v-for="a in ACCENT_OPTIONS"
          :key="a.id"
          :value="a.id"
          :title="a.title"
        >
          <span class="accent-radio__dot" :class="`accent-radio__dot--${a.id}`" />
        </el-radio-button>
      </el-radio-group>
    </div>
  </div>
</template>

<style scoped lang="scss">
.appearance-panel {
  padding: 4px 0;
}

/* 独立分段按钮：取消 EP 组内 -1px 叠边，避免选中项左侧「双边框」 */
.segmented-radio-group {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 8px;
}

.segmented-radio-group :deep(.el-radio-button) {
  margin-left: 0 !important;
}

.segmented-radio-group :deep(.el-radio-button__inner) {
  padding: 6px 14px;
  border-radius: 8px !important;
  border: 1px solid var(--border) !important;
  border-left: 1px solid var(--border) !important;
  margin-left: 0 !important;
  box-shadow: none !important;
  background: var(--bg-input) !important;
  color: var(--text-muted) !important;
  transition:
    background 0.15s,
    border-color 0.15s,
    color 0.15s;
}

.segmented-radio-group :deep(.el-radio-button__inner:hover) {
  border-color: color-mix(in srgb, var(--accent) 40%, var(--border)) !important;
  background: var(--bg-hover) !important;
  color: var(--text) !important;
}

.segmented-radio-group :deep(.el-radio-button.is-active .el-radio-button__inner) {
  background: var(--accent-subtle) !important;
  border-color: var(--accent) !important;
  color: var(--accent) !important;
  box-shadow: none !important;
  z-index: 1;
}

.accent-radio-group {
  gap: 10px;
}

.accent-radio-group :deep(.el-radio-button__inner) {
  padding: 7px 11px;
}

.accent-radio__dot {
  display: block;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  box-shadow: inset 0 0 0 1px color-mix(in srgb, #000 14%, transparent);
}

.segmented-radio-group :deep(.el-radio-button.is-active) .accent-radio__dot {
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, #000 10%, transparent),
    0 0 0 2px color-mix(in srgb, var(--accent) 55%, transparent);
}

.accent-radio__dot--sky {
  background: #4d7cff;
}
.accent-radio__dot--cyan {
  background: #06b6d4;
}
.accent-radio__dot--emerald {
  background: #10b981;
}
.accent-radio__dot--lime {
  background: #84cc16;
}
.accent-radio__dot--amber {
  background: #eab308;
}
.accent-radio__dot--coral {
  background: #f43f5e;
}
.accent-radio__dot--grape {
  background: #a855f7;
}
.accent-radio__dot--graphite {
  background: #64748b;
}
</style>
