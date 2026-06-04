import { invoke } from "@tauri-apps/api/core";
import { refreshHome } from "@/pages/home";
import { el, setMsg, type AppConfig } from "@/lib/shared";
import { getSettingsShell, mountSettingsShell } from "./shell";

function shell() {
  const api = getSettingsShell();
  if (!api) throw new Error("设置页未初始化");
  return api;
}

async function invokeSaveConfig(opts: {
  config: AppConfig;
  accessKeyId?: string | null;
  accessKeySecret?: string | null;
}) {
  await invoke("save_config", {
    payload: {
      config: opts.config,
      accessKeyId: opts.accessKeyId ?? null,
      accessKeySecret: opts.accessKeySecret ?? null,
    },
  });
}

export async function loadSettingsUi() {
  const data = await invoke<{
    config: AppConfig;
    hasSecret: boolean;
    accessKeyId?: string | null;
  }>("get_config");

  shell().loadForm({
    config: data.config,
    hasSecret: data.hasSecret,
    accessKeyId: data.accessKeyId,
  });

  if (data.hasSecret) {
    await shell().refreshRegions(
      data.config.regionId,
      data.config.securityGroupId,
    );
  }
}

export async function onSave() {
  const api = shell();
  const ruleErr = api.validateRules();
  if (ruleErr) {
    setMsg("settingsMsg", ruleErr, "error");
    return;
  }
  if (!api.collectConfig().securityGroupId?.trim()) {
    setMsg("settingsMsg", "请填写或选择安全组", "error");
    return;
  }
  try {
    const { id, secret } = api.readAk();
    await invokeSaveConfig({
      config: api.collectConfig(),
      accessKeyId: id || null,
      accessKeySecret: secret || null,
    });
    setMsg("settingsMsg", "已保存", "success");
  } catch (e) {
    setMsg("settingsMsg", String(e), "error");
  }
}

export function bindSettings() {
  mountSettingsShell(el("settingsShellMount"));
  el<HTMLButtonElement>("btnSave").addEventListener("click", () => onSave());
  window.addEventListener("sync-config-imported", () => {
    void loadSettingsUi();
    void refreshHome();
  });
}
