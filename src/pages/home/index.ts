import { invoke } from "@tauri-apps/api/core";
import {
  getRefreshIpButtonApi,
  mountRefreshIpButton,
} from "./refresh-ip";
import {
  el,
  formatDateTime,
  formatInterval,
  setMsg,
  shortId,
  type ActivityEntry,
  type StatusResponse,
} from "@/lib/shared";

function updateMonitorBadge(monitoring: boolean) {
  const badge = el<HTMLElement>("monitorBadge");
  badge.classList.remove("status-badge--active", "status-badge--idle");
  badge.classList.add(monitoring ? "status-badge--active" : "status-badge--idle");
  badge.querySelector(".status-badge-text")!.textContent = monitoring
    ? "监听中"
    : "未监听";
}

function applySyncStatusPill(st: StatusResponse) {
  const pill = el<HTMLElement>("syncStatusPill");
  pill.textContent = st.syncStatusLabel;
  pill.className = "sync-pill";
  pill.classList.add(`sync-pill--${st.syncStatus}`);
}

export function setHomeCurrentIpDisplay(ip: string | null | undefined) {
  const ipEl = el<HTMLElement>("homeCurrentIp");
  if (ip) {
    ipEl.textContent = ip;
    ipEl.classList.add("is-live");
  } else {
    ipEl.textContent = "—";
    ipEl.classList.remove("is-live");
  }
}

export function updateHomePanel(st: StatusResponse) {
  updateMonitorBadge(st.monitoring);

  if (st.currentIp) {
    setHomeCurrentIpDisplay(st.currentIp);
  }

  applySyncStatusPill(st);

  el<HTMLElement>("homeLastIp").textContent = st.state.lastIp ?? "—";
  el<HTMLElement>("homeLastSyncAt").textContent = formatDateTime(
    st.state.lastSyncAt,
  );
  el<HTMLElement>("homeNextCheckAt").textContent = st.monitoring
    ? formatDateTime(st.nextCheckAt)
    : "监听已停止";
  el<HTMLElement>("homePollInterval").textContent = formatInterval(
    st.pollIntervalSecs,
  );
  el<HTMLElement>("homeRegion").textContent = st.regionId || "未配置";
  el<HTMLElement>("homeSecurityGroup").textContent = shortId(
    st.securityGroupId,
  );
  el<HTMLElement>("homeRulesCount").textContent = st.configReady
    ? `${st.rulesCount} 条`
    : "—";

  const configHint = el<HTMLElement>("homeConfigHint");
  const hideConfigHint = st.configReady;
  configHint.classList.toggle("hidden", hideConfigHint);
  configHint.toggleAttribute("hidden", hideConfigHint);

  const errBox = el<HTMLElement>("homeErrorBox");
  const errText = el<HTMLElement>("homeLastError");
  if (st.state.lastError) {
    errBox.classList.remove("hidden");
    errText.textContent = st.state.lastError;
  } else {
    errBox.classList.add("hidden");
    errText.textContent = "";
  }

  const btnToggle = el<HTMLButtonElement>("btnToggleMonitor");
  btnToggle.textContent = st.monitoring ? "停止监听" : "开始监听";
  btnToggle.classList.toggle("btn--danger-ghost", st.monitoring);
}

async function refreshActivityLog() {
  try {
    const entries = await invoke<ActivityEntry[]>("get_activity_log", {
      limit: 50,
    });
    const list = el<HTMLUListElement>("homeActivityLog");
    const empty = el<HTMLElement>("homeActivityEmpty");
    if (!entries.length) {
      list.innerHTML = "";
      empty.classList.remove("hidden");
      return;
    }
    empty.classList.add("hidden");
    list.innerHTML = entries
      .map(
        (e) =>
          `<li class="activity-log__item activity-log__item--${e.level}">` +
          `<time class="activity-log__time">${formatDateTime(e.at)}</time>` +
          `<span class="activity-log__text">${escapeHtml(e.message)}</span>` +
          `</li>`,
      )
      .join("");
  } catch {
    /* ignore */
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export async function refreshHome() {
  try {
    const st = await invoke<StatusResponse>("get_status");
    updateHomePanel(st);
    await refreshActivityLog();
  } catch {
    /* ignore */
  }
}

export async function onHomeSyncNow(force = false) {
  try {
    const res = await invoke<{ message: string; changed: boolean }>("sync_now", {
      force,
    });
    setMsg("homeMsg", res.message, res.changed ? "success" : "info");
    await refreshHome();
    await probePublicIp();
  } catch (e) {
    setMsg("homeMsg", String(e), "error");
    await refreshHome();
  }
}

export async function onForceSync() {
  if (
    !confirm(
      "将按当前公网 IP 重写本工具管理的规则条目，是否继续？",
    )
  ) {
    return;
  }
  await onHomeSyncNow(true);
}

export async function probePublicIp() {
  try {
    const ip = await invoke<string | null>("fetch_public_ip");
    setHomeCurrentIpDisplay(ip ?? undefined);
    return ip;
  } catch {
    return null;
  }
}

export async function onToggleMonitor() {
  try {
    const st = await invoke<StatusResponse>("get_status");
    if (!st.configReady) {
      setMsg("homeMsg", "请先在设置中完成配置", "error");
      return;
    }
    if (st.monitoring) {
      await invoke("stop_monitor");
      setMsg("homeMsg", "已停止监听", "info");
    } else {
      await invoke("start_monitor");
      setMsg("homeMsg", "已开始监听", "success");
    }
    await refreshHome();
  } catch (e) {
    setMsg("homeMsg", String(e), "error");
  }
}

let refreshingIp = false;

export async function onRefreshPublicIp() {
  if (refreshingIp) return;
  refreshingIp = true;
  const refreshBtn = getRefreshIpButtonApi();
  const ipEl = el<HTMLElement>("homeCurrentIp");
  refreshBtn?.setLoading(true);
  ipEl.textContent = "获取中…";
  ipEl.classList.remove("is-live");
  try {
    const ip = await probePublicIp();
    const st = await invoke<StatusResponse>("get_status");
    updateHomePanel(st);
    if (!ip) {
      setMsg("homeMsg", "未能获取公网 IP，请检查网络或探测地址", "error");
    } else {
      setMsg("homeMsg", "", "info");
    }
  } catch (e) {
    ipEl.textContent = "获取失败";
    setMsg("homeMsg", String(e), "error");
  } finally {
    refreshingIp = false;
    refreshBtn?.setLoading(false);
  }
}

export async function onClearActivityLog() {
  try {
    await invoke("clear_activity_log");
    await refreshActivityLog();
  } catch (e) {
    setMsg("homeMsg", String(e), "error");
  }
}

export async function initActivityLogCap() {
  try {
    const info = await invoke<{ activityLogMaxEntries: number }>("get_app_info");
    el<HTMLElement>("homeActivityLogCap").textContent =
      `仅保留最近 ${info.activityLogMaxEntries} 条操作记录`;
  } catch {
    /* 保留 HTML 默认文案 */
  }
}

export function bindHome() {
  void initActivityLogCap();
  mountRefreshIpButton(() => onRefreshPublicIp());
  el<HTMLButtonElement>("btnSync").addEventListener("click", () =>
    onHomeSyncNow(false),
  );
  el<HTMLButtonElement>("btnForceSync").addEventListener("click", () =>
    onForceSync(),
  );
  el<HTMLButtonElement>("btnToggleMonitor").addEventListener("click", () =>
    onToggleMonitor(),
  );
  el<HTMLButtonElement>("btnClearActivityLog").addEventListener("click", () =>
    onClearActivityLog(),
  );
}
