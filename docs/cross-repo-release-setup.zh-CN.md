# 跨仓库发布配置指南

[English](./cross-repo-release-setup.md) | **中文**

本指南说明如何配置私有仓库的 GitHub Actions 将 Release 发布到公开仓库。

## 架构概览

```
┌─────────────────────────┐         ┌─────────────────────────┐
│   私有仓库 (Private)     │         │   公开仓库 (Public)      │
│   mantra-client         │         │   mantra-releases       │
│                         │         │                         │
│  ┌───────────────────┐  │         │  ┌───────────────────┐  │
│  │ .github/workflows │  │  push   │  │     Releases      │  │
│  │   release.yml     │──┼────────►│  │  ├─ v1.0.0        │  │
│  └───────────────────┘  │ binary  │  │  ├─ v1.1.0        │  │
│                         │         │  │  └─ v1.2.0        │  │
│  源代码 (不公开)         │         │  └───────────────────┘  │
└─────────────────────────┘         │                         │
                                    │  README.md (下载说明)    │
                                    └─────────────────────────┘
```

## 配置步骤

### 1. 创建公开发布仓库

1. 在 GitHub 上创建新仓库，例如 `gonewx/mantra-releases`
2. 设置为 **Public**
3. 添加 README.md（可使用 `.github/PUBLIC_RELEASE_REPO_README.md` 模板）
4. 可选：添加 LICENSE 文件

### 2. 创建 Personal Access Token (PAT)

> ⚠️ **注意**: PAT 在**个人账户设置**中创建，不是仓库设置！

1. 点击 GitHub 右上角你的**头像** → **Settings**（账户设置）
2. 左侧菜单滚动到底部，点击 **Developer settings**
3. 选择 **Personal access tokens** → **Fine-grained tokens**
4. 点击 **Generate new token**
5. 配置：
   - **Token name**: `mantra-release-publisher`
   - **Expiration**: 根据需要设置（建议 90 天，并设置提醒更新）
   - **Repository access**: 选择 "Only select repositories"，然后选择公开发布仓库 `gonewx/mantra-releases`
   - **Permissions**（在 Repository permissions 下）:
     - **Contents**: Read and write（用于创建 Release）
     - **Metadata**: Read-only（必需）
6. 点击 **Generate token** 并复制 Token

💡 **快捷链接**: https://github.com/settings/tokens?type=beta

### 3. 配置私有仓库的 Secrets 和 Variables

在私有仓库 Settings → Secrets and variables → Actions 中配置：

#### Repository secrets

| Secret 名称 | 说明 |
|------------|------|
| `PUBLIC_REPO_TOKEN` | 上一步创建的 PAT |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 应用签名私钥（可选） |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 签名私钥密码（可选） |

#### Repository variables

| Variable 名称 | 值示例 | 说明 |
|--------------|--------|------|
| `PUBLIC_RELEASE_REPO` | `gonewx/mantra-releases` | 公开仓库的 owner/repo 格式 |

### 4. 生成 Tauri 签名密钥（可选但推荐）

```bash
# 安装 tauri-cli（如果未安装）
cargo install tauri-cli

# 生成签名密钥
cargo tauri signer generate -w ~/.tauri/mantra.key

# 输出：
# - Private key: ~/.tauri/mantra.key
# - Public key: ~/.tauri/mantra.key.pub
```

将私钥内容设置为 `TAURI_SIGNING_PRIVATE_KEY`，密码设置为 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。

## 触发发布

### 方式 1：通过 Git Tag 触发

```bash
# 创建并推送版本标签
git tag v1.0.0
git push origin v1.0.0
```

### 方式 2：手动触发

1. 前往 Actions 页面
2. 选择 "Build and Release" 工作流
3. 点击 "Run workflow"
4. 输入版本号（如 `v1.0.0`）
5. 点击 "Run workflow"

## 版本命名规范

- 正式版：`v1.0.0`、`v1.1.0`、`v2.0.0`
- 预发布版：`v1.0.0-beta.1`、`v1.0.0-rc.1`
- 开发版：`v1.0.0-alpha.1`

工作流会根据版本号自动判断是否为预发布版本。

## 故障排查

### Token 权限不足

错误信息：`Resource not accessible by integration`

解决：确保 PAT 对公开仓库有 Contents 的 Read and write 权限。

### 仓库不存在

错误信息：`Not Found`

解决：检查 `PUBLIC_RELEASE_REPO` 变量的格式是否正确（`owner/repo`）。

### 构建失败

1. 检查 Rust 工具链版本
2. 确保 pnpm 依赖正确安装
3. Linux 构建需要系统依赖（已在工作流中配置）

## 安全注意事项

1. **PAT 过期提醒**：设置日历提醒，在 Token 过期前更新
2. **最小权限原则**：PAT 只授予必要的仓库和权限
3. **Secret 轮换**：定期更换 Token 和签名密钥
4. **审计日志**：定期检查 Actions 运行日志

## 相关文件

- `.github/workflows/release.yml` - 主发布工作流
- `docs/PUBLIC_RELEASE_REPO_README.md` - 公开仓库 README 模板 (English)
- `docs/PUBLIC_RELEASE_REPO_README.zh-CN.md` - 公开仓库 README 模板 (中文)
- `docs/cross-repo-release-setup.md` - 本配置指南 (English)
