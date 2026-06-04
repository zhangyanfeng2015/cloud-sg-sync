# 发版说明

版本号须三处一致：`package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`。

## 1. 构建安装包

在项目根目录：

```powershell
pnpm install
pnpm tauri build
```

产物目录：

```text
src-tauri/target/release/bundle/nsis/
```

常见文件名：`Cloud SG Sync_1.0.0_x64-setup.exe`（以实际文件名为准）。

## 2. 打 Git 标签并推送

标签格式必须为 `v1.0.0`（与关于页更新检查解析一致）。

```powershell
git tag -a v1.0.0 -m "release: v1.0.0"
git push origin v1.0.0
git push gitee v1.0.0
```

若标签已存在需重建（仅本地未推送时）：

```powershell
git tag -d v1.0.0
git tag -a v1.0.0 -m "release: v1.0.0"
```

## 3. GitHub Release

1. 打开 https://github.com/zhangyanfeng2015/cloud-sg-sync/releases/new  
2. **Choose a tag**：`v1.0.0`（无则选 “Create new tag” 并填 `v1.0.0`）  
3. **Release title**：`v1.0.0`  
4. 说明可复制 [`release-notes/v1.0.0.md`](release-notes/v1.0.0.md)  
5. 上传 `nsis` 目录下的 `*-setup.exe`  
6. 发布（Publish release）

## 4. Gitee 发行版

1. 打开 https://gitee.com/zhangyanfeng2015/cloud-sg-sync/releases  
2. **创建发行版** → 版本号 `v1.0.0`，关联上述标签（或新建标签）  
3. 说明同上，上传同一安装包  
4. 保存

## 5. 验证

- 关于页「检查更新」应能读到 Release（不再提示“尚未发布”）。  
- 新用户从 Release 页下载安装包可正常安装。

后续版本：复制 `release-notes/v1.0.0.md` 为新文件，递增版本号后重复步骤 1–4。
