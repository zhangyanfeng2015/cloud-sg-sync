/** 轮询间隔选项（秒），与后端 MIN/MAX 一致 */
export const POLL_INTERVAL_OPTIONS: { label: string; value: number }[] = [
  { label: "每 1 分钟", value: 60 },
  { label: "每 5 分钟", value: 300 },
  { label: "每 10 分钟", value: 600 },
  { label: "每 15 分钟", value: 900 },
  { label: "每 30 分钟", value: 1800 },
  { label: "每 1 小时", value: 3600 },
  { label: "每 6 小时", value: 21600 },
  { label: "每 12 小时", value: 43200 },
  { label: "每 1 天", value: 86400 },
  { label: "每 3 天", value: 259200 },
  { label: "每 7 天", value: 604800 },
];

export const MIN_POLL_INTERVAL_SECS = 60;
export const MAX_POLL_INTERVAL_SECS = 7 * 24 * 3600;

/** 将秒数规范到可选值；不在列表时取不超过上限的最接近合法值 */
export function normalizePollIntervalSecs(secs: number): number {
  const n = Math.floor(secs);
  if (n <= MIN_POLL_INTERVAL_SECS) return MIN_POLL_INTERVAL_SECS;
  if (n >= MAX_POLL_INTERVAL_SECS) return MAX_POLL_INTERVAL_SECS;
  const hit = POLL_INTERVAL_OPTIONS.find((o) => o.value === n);
  if (hit) return hit.value;
  const sorted = [...POLL_INTERVAL_OPTIONS].map((o) => o.value).sort((a, b) => a - b);
  let best = sorted[0];
  for (const v of sorted) {
    if (v <= n) best = v;
    else break;
  }
  return best;
}
