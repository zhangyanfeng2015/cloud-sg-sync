import { createApp, h, ref, type App } from "vue";

import SettingsShell from "./components/SettingsShell.vue";
import type { SettingsPanelId, SettingsShellApi } from "@/types/settings";

export type { SettingsPanelId, SettingsShellApi, SyncConfigFields } from "@/types/settings";

let app: App | null = null;
let shell: SettingsShellApi | null = null;

function updateToolbarSave(panel: SettingsPanelId) {
  const group = document.getElementById("toolbarSaveGroup");
  if (group) group.hidden = panel !== "sync";
}

export function mountSettingsShell(container: HTMLElement) {
  if (app) return;

  const root = document.createElement("div");
  root.className = "settings-shell-mount";
  container.appendChild(root);

  const shellRef = ref<SettingsShellApi | null>(null);

  app = createApp({
    render: () =>
      h(SettingsShell, {
        ref: (el: unknown) => {
          shellRef.value = el as SettingsShellApi;
          shell = shellRef.value;
        },
        onPanelChange: (panel: SettingsPanelId) => updateToolbarSave(panel),
      }),
  });
  app.mount(root);
  updateToolbarSave("sync");
}

export function getSettingsShell(): SettingsShellApi | null {
  return shell;
}

export function resetSettingsNav() {
  shell?.resetToSync();
  updateToolbarSave("sync");
}
