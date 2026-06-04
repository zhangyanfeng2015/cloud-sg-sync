export { PORT_PRESETS } from "./port-presets";
import { POLL_INTERVAL_OPTIONS } from "./poll-intervals";
export {
  POLL_INTERVAL_OPTIONS,
  normalizePollIntervalSecs,
  MIN_POLL_INTERVAL_SECS,
  MAX_POLL_INTERVAL_SECS,
} from "./poll-intervals";

export type RuleDirection = "ingress" | "egress";

export interface Rule {
  port: string;
  protocol: string;
  description: string;
  direction: RuleDirection;
}

export interface AppConfig {
  version: number;
  regionId: string;
  securityGroupId: string;
  pollIntervalSecs: number;
  autoStartWindows: boolean;
  monitoringEnabled: boolean;
  ipProbeUrls: string[];
  rules: Rule[];
  themeMode: string;
  themeAccent: string;
}

export interface AppState {
  lastIp?: string | null;
  lastSyncAt?: string | null;
  lastCheckAt?: string | null;
  lastError?: string | null;
}

export interface ActivityEntry {
  at: string;
  level: string;
  message: string;
}

export interface StatusResponse {
  currentIp?: string | null;
  state: AppState;
  monitoring: boolean;
  pollIntervalSecs: number;
  nextCheckAt?: string | null;
  configReady: boolean;
  regionId: string;
  securityGroupId: string;
  rulesCount: number;
  syncStatus: string;
  syncStatusLabel: string;
}

export function el<T extends HTMLElement>(id: string): T {
  return document.getElementById(id) as T;
}

export function setMsg(
  id: string,
  text: string,
  kind: "success" | "error" | "info" = "info",
) {
  const node = el<HTMLElement>(id);
  node.textContent = text;
  node.classList.remove("msg-success", "msg-error", "msg-info", "error");
  if (text) {
    node.classList.add(`msg-${kind}`);
  }
}

/** 固定格式：2026-11-12 11:11:11 */
export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    const h = String(d.getHours()).padStart(2, "0");
    const min = String(d.getMinutes()).padStart(2, "0");
    const s = String(d.getSeconds()).padStart(2, "0");
    return `${y}-${m}-${day} ${h}:${min}:${s}`;
  } catch {
    return iso;
  }
}

export function formatInterval(secs: number): string {
  const hit = POLL_INTERVAL_OPTIONS.find((o) => o.value === secs);
  if (hit) return hit.label;
  if (secs < 120) return `每 ${secs} 秒`;
  if (secs % 86400 === 0) {
    const d = secs / 86400;
    return d === 1 ? "每 1 天" : `每 ${d} 天`;
  }
  if (secs % 3600 === 0) {
    const h = secs / 3600;
    return h === 1 ? "每 1 小时" : `每 ${h} 小时`;
  }
  if (secs % 60 === 0) return `每 ${secs / 60} 分钟`;
  return `每 ${secs} 秒`;
}

export function shortId(id: string): string {
  if (!id) return "—";
  return id.length > 20 ? `${id.slice(0, 10)}…${id.slice(-6)}` : id;
}

export function defaultRule(
  port: string,
  direction: RuleDirection = "ingress",
): Rule {
  return {
    port,
    protocol: "tcp",
    description: `auto-whitelist:${port}`,
    direction,
  };
}

export function parsePortInput(raw: string): string | null {
  const v = raw.trim();
  if (!/^\d+$/.test(v)) return null;
  const n = Number(v);
  if (n < 1 || n > 65535) return null;
  return String(n);
}
