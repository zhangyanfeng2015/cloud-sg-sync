/** 供 Vue 面板复用的应用元信息类型（与原 app-meta.ts 一致） */

export interface AppInfo {
  name: string;
  version: string;
  description: string;
  dataDirHint: string;
  activityLogMaxEntries: number;
}

export interface UpdateCheck {
  currentVersion: string;
  latestVersion: string;
  hasUpdate: boolean;
  message: string;
  releaseUrl: string;
}
