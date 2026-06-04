export type ThemeMode = "dark" | "light" | "system";

export type ThemeAccent =
  | "sky"
  | "cyan"
  | "emerald"
  | "lime"
  | "amber"
  | "coral"
  | "grape"
  | "graphite";

export const ACCENT_OPTIONS: { id: ThemeAccent; title: string }[] = [
  { id: "sky", title: "晴空蓝" },
  { id: "cyan", title: "天青" },
  { id: "emerald", title: "翠绿" },
  { id: "lime", title: "草绿" },
  { id: "amber", title: "琥珀" },
  { id: "coral", title: "珊瑚" },
  { id: "grape", title: "葡萄紫" },
  { id: "graphite", title: "石墨" },
];

const ACCENT_SET = new Set<ThemeAccent>(ACCENT_OPTIONS.map((a) => a.id));

const LEGACY_ACCENT_MAP: Record<string, ThemeAccent> = {
  ocean: "cyan",
  teal: "emerald",
  sunset: "amber",
  rose: "coral",
  violet: "grape",
  indigo: "sky",
  slate: "graphite",
};

let systemMq: MediaQueryList | null = null;
let systemListener: (() => void) | null = null;
let cachedAccent: ThemeAccent = "sky";

export function normalizeAccent(value: string | undefined | null): ThemeAccent {
  if (value && ACCENT_SET.has(value as ThemeAccent)) {
    return value as ThemeAccent;
  }
  if (value && LEGACY_ACCENT_MAP[value]) {
    return LEGACY_ACCENT_MAP[value];
  }
  return "sky";
}

function resolveMode(mode: ThemeMode): "dark" | "light" {
  if (mode === "system") {
    return window.matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  }
  return mode;
}

export function applyTheme(mode: ThemeMode, accent: ThemeAccent) {
  const root = document.documentElement;
  const resolved = resolveMode(mode);
  const safeAccent = normalizeAccent(accent);
  root.dataset.theme = resolved;
  root.dataset.accent = safeAccent;
  root.classList.toggle("dark", resolved === "dark");
  cachedAccent = safeAccent;
}

function bindSystemListener(mode: ThemeMode) {
  if (systemListener && systemMq) {
    systemMq.removeEventListener("change", systemListener);
  }
  systemListener = null;
  systemMq = null;

  if (mode !== "system") return;

  systemMq = window.matchMedia("(prefers-color-scheme: light)");
  systemListener = () => applyTheme("system", cachedAccent);
  systemMq.addEventListener("change", systemListener);
}

function syncModeUi(mode: ThemeMode) {
  const sel = document.getElementById("themeMode") as HTMLSelectElement | null;
  if (sel) sel.value = mode;
  document
    .querySelectorAll<HTMLInputElement>('input[name="themeModeRadio"]')
    .forEach((inp) => {
      inp.checked = inp.value === mode;
    });
}

export function initTheme(mode: ThemeMode, accent: ThemeAccent) {
  applyTheme(mode, normalizeAccent(accent));
  bindSystemListener(mode);
  syncModeUi(mode);
}

export function bindThemeControls(
  onChange: (mode: ThemeMode, accent: ThemeAccent) => void,
) {
  const emitFromMode = (mode: ThemeMode) => {
    const accent = normalizeAccent(
      (
        document.querySelector(
          'input[name="themeAccent"]:checked',
        ) as HTMLInputElement
      )?.value,
    );
    syncModeUi(mode);
    initTheme(mode, accent);
    onChange(mode, accent);
  };

  document
    .querySelectorAll<HTMLInputElement>('input[name="themeModeRadio"]')
    .forEach((inp) => {
      inp.addEventListener("change", () => {
        if (inp.checked) emitFromMode(inp.value as ThemeMode);
      });
    });

  const modeSel = document.getElementById("themeMode") as HTMLSelectElement | null;
  modeSel?.addEventListener("change", () => emitFromMode(modeSel.value as ThemeMode));

  document
    .querySelectorAll<HTMLInputElement>('input[name="themeAccent"]')
    .forEach((inp) => {
      inp.addEventListener("change", () => {
        if (!inp.checked) return;
        const mode = (modeSel?.value ||
          document.querySelector('input[name="themeModeRadio"]:checked')
            ?.getAttribute("value") ||
          "dark") as ThemeMode;
        const accent = normalizeAccent(inp.value);
        initTheme(mode, accent);
        onChange(mode, accent);
      });
    });
}

export function setThemeUi(mode: ThemeMode, accent: ThemeAccent) {
  syncModeUi(mode);
  const safe = normalizeAccent(accent);
  const inp = document.querySelector(
    `input[name="themeAccent"][value="${safe}"]`,
  ) as HTMLInputElement | null;
  if (inp) inp.checked = true;
  initTheme(mode, safe);
}
