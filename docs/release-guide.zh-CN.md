# 发布操作指南

[English](./release-guide.md) | **中文**

本文档说明如何发布 Mantra Client 的新版本。

---

## 发布流程概览

```
代码就绪 → 创建 Tag → 自动构建 → 私有仓库 Release → 公开仓库 Release
                         │                │                  │
                    (自动触发)        (自动创建)          (自动同步)
```

---

## 方式一：完整发布（推荐）

通过 Git Tag 触发完整构建和发布流程。

### 步骤

```bash
# 1. 确保代码已提交并推送
git add .
git commit -m "chore: prepare release v0.1.0"
git push

# 2. 创建版本标签
git tag v0.1.0

# 3. 推送标签触发构建
git push origin v0.1.0
```

### 发生了什么？

1. **构建** - 4 个平台并行构建（约 15-20 分钟）
   - macOS (Apple Silicon)
   - macOS (Intel)
   - Windows
   - Linux

2. **私有仓库发布** - 自动创建 GitHub Release

3. **公开仓库发布** - 自动同步到 `gonewx/mantra-releases`

### 查看进度

前往 [Actions](../../actions) 页面查看构建状态。

---

## 方式二：仅同步到公开仓库

如果私有仓库已有 Release，但公开仓库发布失败或需要重新发布。

### 步骤

1. 前往 [Actions](../../actions) 页面
2. 左侧选择 **"Publish to Public Repository"**
3. 点击 **"Run workflow"**
4. 输入版本号（如 `v0.1.0-alpha.1`）
5. 点击绿色 **"Run workflow"** 按钮

> ⚠️ 前提：私有仓库必须已存在该版本的 Release

---

## 方式三：手动触发完整构建

无需创建 Tag，适用于测试构建流程。

### 步骤

1. 前往 [Actions](../../actions) 页面
2. 左侧选择 **"Release"**
3. 点击 **"Run workflow"**
4. 输入版本号（如 `v0.1.0-test`）
5. 点击绿色 **"Run workflow"** 按钮

> ⚠️ 手动触发不会自动创建 Release，仅生成构建产物

---

## 版本号规范

| 类型 | 格式 | 示例 | 说明 |
|------|------|------|------|
| 正式版 | `vX.Y.Z` | `v1.0.0` | 稳定发布 |
| Alpha | `vX.Y.Z-alpha.N` | `v0.1.0-alpha.1` | 早期预览 |
| Beta | `vX.Y.Z-beta.N` | `v1.0.0-beta.1` | 公测版本 |
| RC | `vX.Y.Z-rc.N` | `v1.0.0-rc.1` | 发布候选 |

带有 `-` 的版本会自动标记为 **Pre-release**。

---

## 构建产物命名

| 平台 | 文件名格式 |
|------|-----------|
| macOS (Apple Silicon) | `Mantra_v0.1.0_macos-arm64.dmg` |
| macOS (Intel) | `Mantra_v0.1.0_macos-x64.dmg` |
| Windows (MSI) | `Mantra_v0.1.0_windows-x64.msi` |
| Windows (EXE) | `Mantra_v0.1.0_windows-x64.exe` |
| Linux (AppImage) | `Mantra_v0.1.0_linux-x64.AppImage` |
| Linux (Deb) | `Mantra_v0.1.0_linux-x64.deb` |

---

## 常见问题

### 构建失败怎么办？

1. 查看 Actions 日志定位错误
2. 修复代码问题
3. 删除失败的 Tag：`git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`
4. 重新创建 Tag 并推送

### 公开仓库发布失败？

使用 **"Publish to Public Repository"** 工作流重新同步。

### 如何撤回一个版本？

1. 删除公开仓库的 Release（手动在 GitHub 上操作）
2. 删除私有仓库的 Release
3. 删除 Git Tag：`git push origin :refs/tags/v0.1.0`

### macOS 显示"无法验证开发者"？

由于 Mantra 目前未进行 Apple 代码签名，首次打开需要手动授权：

1. **右键打开**：在访达中按住 Control 键点击应用 → 选择"打开" → 再次点击"打开"
2. **系统设置**：系统设置 → 隐私与安全性 → 找到 Mantra 被阻止的提示 → 点击"仍要打开"
3. **命令行**：`xattr -cr /Applications/Mantra.app`

---

## 相关文件

| 文件 | 说明 |
|------|------|
| `.github/workflows/release.yml` | 主发布工作流 |
| `.github/workflows/publish-public.yml` | 公开仓库同步工作流 |
| `docs/cross-repo-release-setup.zh-CN.md` | 跨仓库发布配置指南 |
