# 安全策略

## 报告方式

请勿在公开 Issue 中提供 AccessKey、`.agsync` 文件、`secrets.dat` 或完整 `config.json`。

可在 GitHub 仓库 **Security → Advisories** 中私下报告（启用后无需额外配置文件）。

## 使用建议

- 配置与凭证保存在 `%AppData%\CloudSgSync\`，请勿将数据目录打包上传至 Issue。  
- 建议使用 RAM 子账号，并仅授予 README 中列出的 ECS 安全组相关权限，避免长期使用主账号 AccessKey。

## 版本支持

当前维护版本：**1.0.x**（接受安全相关修复评估）。
