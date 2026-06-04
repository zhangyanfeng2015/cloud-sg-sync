export interface PortOption {
  value: string;
  service: string;
}

export interface PortOptionGroup {
  label: string;
  options: PortOption[];
}

/** 分组常用端口（下拉展示「端口 + 服务名」） */
export const PORT_OPTION_GROUPS: PortOptionGroup[] = [
  {
    label: "远程与终端",
    options: [
      { value: "22", service: "SSH" },
      { value: "3389", service: "RDP" },
    ],
  },
  {
    label: "Web 与代理",
    options: [
      { value: "80", service: "HTTP" },
      { value: "443", service: "HTTPS" },
      { value: "8080", service: "HTTP 备用" },
      { value: "8443", service: "HTTPS 备用" },
    ],
  },
  {
    label: "数据库与缓存",
    options: [
      { value: "3306", service: "MySQL" },
      { value: "5432", service: "PostgreSQL" },
      { value: "6379", service: "Redis" },
      { value: "27017", service: "MongoDB" },
    ],
  },
];

export const PORT_PRESETS: string[] = PORT_OPTION_GROUPS.flatMap((g) =>
  g.options.map((o) => o.value),
);

const presetSet = new Set(PORT_PRESETS);

export function isPresetPort(port: string): boolean {
  return presetSet.has(port.trim());
}

export function portOptionLabel(opt: PortOption): string {
  return `${opt.value} · ${opt.service}`;
}

/** 自动完成下拉项 */
export interface PortSuggestion {
  value: string;
  service: string;
  group: string;
}

export function buildPortSuggestions(): PortSuggestion[] {
  const list: PortSuggestion[] = [];
  for (const g of PORT_OPTION_GROUPS) {
    for (const opt of g.options) {
      list.push({
        value: opt.value,
        service: opt.service,
        group: g.label,
      });
    }
  }
  return list;
}
