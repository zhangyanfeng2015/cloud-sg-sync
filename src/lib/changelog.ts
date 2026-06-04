export interface ChangelogEntry {
  version: string;
  date: string;
  items: string[];
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: "1.0.0",
    date: "2026-06",
    items: [
      "支持阿里云 ECS 安全组按公网 IPv4 自动更新入站/出站规则",
      "系统托盘、配置导入导出（.agsync）、同步预览（Dry-run）、强制同步",
      "深色/浅色主题与多种强调色",
    ],
  },
];

export function renderChangelogList(containerId: string) {
  const root = document.getElementById(containerId);
  if (!root) return;
  root.innerHTML = CHANGELOG.map(
    (e) => `
    <article class="changelog-entry">
      <header class="changelog-entry__head">
        <h3 class="changelog-entry__ver">v${e.version}</h3>
        <time class="changelog-entry__date" datetime="${e.date}">${e.date}</time>
      </header>
      <ul class="changelog-entry__list">
        ${e.items.map((item) => `<li>${escapeHtml(item)}</li>`).join("")}
      </ul>
    </article>
  `,
  ).join("");
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
