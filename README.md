# Cloud SG Sync（安全组同步）

Windows 桌面应用：监测本机公网 IPv4 变化，并按配置将**单个**阿里云 ECS 安全组的入站/出站规则更新为当前地址（`ip/32`）。

- GitHub：https://github.com/zhangyanfeng2015/cloud-sg-sync  
- Gitee：https://gitee.com/zhangyanfeng2015/cloud-sg-sync  

## 许可与协作

采用 [MIT License](./LICENSE)，可自由使用、修改与再分发（须保留版权与许可声明）。本仓库**不接受**外部 Pull Request，说明见 [CONTRIBUTING.md](./CONTRIBUTING.md)。安全问题见 [SECURITY.md](./SECURITY.md)。

## 环境要求

- Windows 10 / 11  
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)  
- Rust stable、Node.js 18+、pnpm  

## 阿里云 RAM（最小权限）

```json
{
  "Version": "1",
  "Statement": [{
    "Effect": "Allow",
    "Action": [
      "ecs:DescribeRegions",
      "ecs:DescribeSecurityGroups",
      "ecs:DescribeSecurityGroupAttribute",
      "ecs:AuthorizeSecurityGroup",
      "ecs:RevokeSecurityGroup"
    ],
    "Resource": "*"
  }]
}
```

## 目录结构

```text
.
├── index.html              # 页面骨架（首页 / 设置）
├── package.json
├── src/                    # 前端（Vite + Vue 3）
│   ├── main.ts
│   ├── app/                # 导航、主题
│   ├── lib/                # 共用逻辑、changelog
│   ├── assets/
│   ├── types/
│   ├── styles/
│   └── pages/
│       ├── home/           # 首页状态、同步、IP 刷新
│       └── settings/       # 设置壳与子组件
├── src-tauri/              # Tauri 2 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── icons/              # 应用图标（源文件在 icons/source/）
│   └── src/
│       ├── main.rs
│       ├── lib.rs          # 入口、invoke 注册
│       ├── app/            # 配置、凭证、备份、日志
│       ├── sync/           # IP 探测、同步引擎、轮询
│       ├── cloud/aliyun/   # ECS 安全组 API
│       ├── commands/       # 暴露给前端的 command
│       ├── platform/       # 托盘、开机自启
│       └── services/       # 版本更新检查
└── scripts/                # 图标生成等脚本
```

## 开发与构建

克隆仓库后，在**项目根目录**执行：

```powershell
pnpm install
pnpm tauri dev
pnpm tauri build
```

Release 安装包路径：`src-tauri/target/release/bundle/nsis/`。若 Rust 构建缓存异常，可执行 `pnpm run cargo:clean` 后重新构建。

首次发版或打 GitHub/Gitee Release 的步骤见 [RELEASE.md](./RELEASE.md)（标签 `v1.0.0` + 上传安装包）。

应用图标：编辑 `src-tauri/icons/source/*.svg`，执行 `pnpm run icons:gen` 重新生成各尺寸资源。

## 本地数据

数据目录：`%AppData%\CloudSgSync\`

| 文件 | 说明 |
|------|------|
| `config.json` | 地域、安全组、端口规则、轮询间隔、主题 |
| `state.json` | 上次同步 IP、检查/同步时间、错误信息 |
| `secrets.dat` | AccessKey（Windows 本机加密存储） |
| `activity-log.json` | 操作记录（最多 50 条） |

导出的 `.agsync` 可能包含敏感信息，请勿提交至版本库。

## 功能说明

- 轮询间隔支持 1 分钟至 7 天；开启监听后，首页每 5 秒刷新状态，**不会**周期性请求外网 IP 探测服务，需手动刷新或启动时探测。  
- **立即同步**：公网 IP 与上次记录一致时不写入 ECS；**强制同步**在 IP 未变时仍按当前公网 IP 更新本工具维护的规则。  
- **预览变更**（Dry-run）仅计算计划步骤，不修改云端。  
- 关闭主窗口后最小化至系统托盘；未实现系统通知。  
- 开机自启可在关于页配置（`tauri-plugin-autostart`）。
