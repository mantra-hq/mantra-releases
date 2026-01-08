# Cross-Repository Release Configuration Guide

**English** | [中文](./cross-repo-release-setup.zh-CN.md)

This guide explains how to configure GitHub Actions in a private repository to publish releases to a public repository.

## Architecture Overview

```
┌─────────────────────────┐         ┌─────────────────────────┐
│   Private Repository    │         │   Public Repository     │
│   mantra-client         │         │   mantra-releases       │
│                         │         │                         │
│  ┌───────────────────┐  │         │  ┌───────────────────┐  │
│  │ .github/workflows │  │  push   │  │     Releases      │  │
│  │   release.yml     │──┼────────►│  │  ├─ v1.0.0        │  │
│  └───────────────────┘  │ binary  │  │  ├─ v1.1.0        │  │
│                         │         │  │  └─ v1.2.0        │  │
│  Source Code (Private)  │         │  └───────────────────┘  │
└─────────────────────────┘         │                         │
                                    │  README.md (Download)   │
                                    └─────────────────────────┘
```

## Configuration Steps

### 1. Create Public Release Repository

1. Create a new repository on GitHub, e.g., `gonewx/mantra-releases`
2. Set it to **Public**
3. Add README.md (use `docs/PUBLIC_RELEASE_REPO_README.md` template)
4. Optional: Add LICENSE file

### 2. Create Personal Access Token (PAT)

1. Go to GitHub Settings → Developer settings → Personal access tokens → Fine-grained tokens
2. Click "Generate new token"
3. Configure:
   - **Token name**: `mantra-release-publisher`
   - **Expiration**: Set as needed (recommend 90 days with reminder)
   - **Repository access**: Select "Only select repositories", then choose the public release repo
   - **Permissions**:
     - **Contents**: Read and write (for creating releases)
     - **Metadata**: Read-only (required)
4. Generate and copy the token

### 3. Configure Private Repository Secrets and Variables

In private repository Settings → Secrets and variables → Actions:

#### Repository Secrets

| Secret Name | Description |
|-------------|-------------|
| `PUBLIC_REPO_TOKEN` | PAT created in previous step |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri app signing key (optional) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signing key password (optional) |

#### Repository Variables

| Variable Name | Example Value | Description |
|---------------|---------------|-------------|
| `PUBLIC_RELEASE_REPO` | `gonewx/mantra-releases` | Public repo in owner/repo format |

### 4. Generate Tauri Signing Key (Optional but Recommended)

```bash
# Install tauri-cli (if not installed)
cargo install tauri-cli

# Generate signing key
cargo tauri signer generate -w ~/.tauri/mantra.key

# Output:
# - Private key: ~/.tauri/mantra.key
# - Public key: ~/.tauri/mantra.key.pub
```

Set private key content as `TAURI_SIGNING_PRIVATE_KEY` and password as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

## Triggering Releases

### Option 1: Via Git Tag

```bash
# Create and push version tag
git tag v1.0.0
git push origin v1.0.0
```

### Option 2: Manual Trigger

1. Go to Actions page
2. Select "Release" workflow
3. Click "Run workflow"
4. Enter version number (e.g., `v1.0.0`)
5. Click "Run workflow"

## Version Naming Convention

- Stable: `v1.0.0`, `v1.1.0`, `v2.0.0`
- Pre-release: `v1.0.0-beta.1`, `v1.0.0-rc.1`
- Development: `v1.0.0-alpha.1`

The workflow automatically determines pre-release status based on version number.

## Troubleshooting

### Insufficient Token Permissions

Error: `Resource not accessible by integration`

Solution: Ensure PAT has Contents Read and write permission for the public repo.

### Repository Not Found

Error: `Not Found`

Solution: Check `PUBLIC_RELEASE_REPO` variable format (should be `owner/repo`).

### Build Failures

1. Check Rust toolchain version
2. Ensure pnpm dependencies are correctly installed
3. Linux builds require system dependencies (configured in workflow)

## Security Considerations

1. **PAT Expiration Reminder**: Set calendar reminder to update before expiration
2. **Least Privilege Principle**: Only grant necessary permissions to PAT
3. **Secret Rotation**: Regularly rotate tokens and signing keys
4. **Audit Logs**: Periodically review Actions run logs

## Related Files

- `.github/workflows/release.yml` - Main release workflow
- `docs/PUBLIC_RELEASE_REPO_README.md` - Public repo README template (English)
- `docs/PUBLIC_RELEASE_REPO_README.zh-CN.md` - Public repo README template (中文)
- `docs/cross-repo-release-setup.zh-CN.md` - This guide (中文)
