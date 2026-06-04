import "element-plus/theme-chalk/dark/css-vars.css";
import "@/styles/element-plus.scss";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { bindNav, openSettings, showPage } from "@/app/nav";
import { initTheme, normalizeAccent, type ThemeMode } from "@/app/theme";
import { bindHome, probePublicIp, refreshHome } from "@/pages/home";
import { bindSettings } from "@/pages/settings";

window.addEventListener("DOMContentLoaded", async () => {
  bindNav();
  bindHome();
  bindSettings();

  try {
    const { config } = await invoke<{ config: { themeMode: string; themeAccent: string } }>(
      "get_config",
    );
    initTheme(
      (config.themeMode || "dark") as ThemeMode,
      normalizeAccent(config.themeAccent),
    );
  } catch {
    initTheme("dark", "sky");
  }

  showPage("home");
  void (async () => {
    await probePublicIp();
    await refreshHome();
  })();

  setInterval(() => refreshHome(), 5000);
  listen("sync-status", () => refreshHome()).catch(console.error);
  listen("open-settings", () => openSettings().catch(console.error)).catch(
    console.error,
  );
});
