# 应用图标

| 文件 | 说明 |
|------|------|
| `source/app-icon.svg` | 主图标（彩色） |
| `source/tray-icon.svg` | 托盘图标（单色剪影） |

重新生成各平台尺寸：

```powershell
pnpm run icons:gen
```

托盘路径配置：`tauri.conf.json` → `trayIcon.iconPath`（`icons/tray.ico`）。
