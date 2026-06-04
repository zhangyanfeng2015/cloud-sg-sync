import { refreshHome } from "@/pages/home";
import { loadSettingsUi } from "@/pages/settings";
import { resetSettingsNav } from "@/pages/settings/shell";
import { el } from "@/lib/shared";

export type PageId = "home" | "settings";

let currentPage: PageId = "home";

export function getCurrentPage(): PageId {
  return currentPage;
}

export function showPage(page: PageId) {
  currentPage = page;
  document.querySelectorAll<HTMLElement>("[data-page-panel]").forEach((panel) => {
    const id = panel.dataset.pagePanel as PageId;
    panel.classList.toggle("page--hidden", id !== page);
  });
  const settingsBtn = el<HTMLButtonElement>("btnOpenSettings");
  settingsBtn.classList.toggle("btn--settings-active", page === "settings");
  settingsBtn.style.visibility = page === "settings" ? "hidden" : "visible";

  document.body.dataset.page = page;
}

export async function openSettings() {
  resetSettingsNav();
  await loadSettingsUi();
  showPage("settings");
}

export function goHome() {
  showPage("home");
  refreshHome();
}

export function bindNav() {
  el<HTMLButtonElement>("btnOpenSettings").addEventListener("click", () => {
    openSettings().catch(console.error);
  });
  el<HTMLButtonElement>("btnBackHome").addEventListener("click", () => goHome());
}
