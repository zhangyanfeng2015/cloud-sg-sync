import type { AppConfig } from "@/lib/shared";

export type SettingsPanelId = "sync" | "sg" | "appearance" | "about";

export type SyncConfigFields = Pick<
  AppConfig,
  | "version"
  | "regionId"
  | "securityGroupId"
  | "pollIntervalSecs"
  | "monitoringEnabled"
  | "ipProbeUrls"
  | "rules"
>;

export interface SettingsShellApi {
  resetToSync: () => void;
  navigateToSg: () => void;
  loadForm: (data: {
    config: AppConfig;
    accessKeyId?: string | null;
    hasSecret: boolean;
  }) => void;
  collectSyncFields: () => SyncConfigFields;
  collectConfig: () => AppConfig;
  readAk: () => { id: string; secret: string };
  validateRules: () => string | null;
  refreshRegions: (region?: string, sg?: string) => Promise<void>;
}
