# Release Guide

**English** | [中文](./release-guide.zh-CN.md)

This document explains how to release new versions of Mantra Client.

---

## Release Flow Overview

```
Code Ready → Create Tag → Auto Build → Private Repo Release → Public Repo Release
                              │                │                    │
                         (auto trigger)   (auto create)        (auto sync)
```

---

## Option 1: Full Release (Recommended)

Trigger a complete build and release process via Git Tag.

### Steps

```bash
# 1. Ensure code is committed and pushed
git add .
git commit -m "chore: prepare release v0.1.0"
git push

# 2. Create version tag
git tag v0.1.0

# 3. Push tag to trigger build
git push origin v0.1.0
```

### What Happens?

1. **Build** - 4 platforms build in parallel (~15-20 minutes)
   - macOS (Apple Silicon)
   - macOS (Intel)
   - Windows
   - Linux

2. **Private Repo Release** - Auto-creates GitHub Release

3. **Public Repo Release** - Auto-syncs to `gonewx/mantra-releases`

### Monitor Progress

Go to [Actions](../../actions) page to view build status.

---

## Option 2: Sync to Public Repo Only

If private repo already has a Release, but public repo publishing failed or needs re-publishing.

### Steps

1. Go to [Actions](../../actions) page
2. Select **"Publish to Public Repository"** on the left
3. Click **"Run workflow"**
4. Enter version number (e.g., `v0.1.0-alpha.1`)
5. Click green **"Run workflow"** button

> ⚠️ Prerequisite: Private repo must already have a Release for this version

---

## Option 3: Manual Build Trigger

Trigger build without creating a Tag, useful for testing the build process.

### Steps

1. Go to [Actions](../../actions) page
2. Select **"Release"** on the left
3. Click **"Run workflow"**
4. Enter version number (e.g., `v0.1.0-test`)
5. Click green **"Run workflow"** button

> ⚠️ Manual trigger won't auto-create Release, only generates build artifacts

---

## Version Numbering

| Type | Format | Example | Description |
|------|--------|---------|-------------|
| Stable | `vX.Y.Z` | `v1.0.0` | Stable release |
| Alpha | `vX.Y.Z-alpha.N` | `v0.1.0-alpha.1` | Early preview |
| Beta | `vX.Y.Z-beta.N` | `v1.0.0-beta.1` | Public beta |
| RC | `vX.Y.Z-rc.N` | `v1.0.0-rc.1` | Release candidate |

Versions with `-` are automatically marked as **Pre-release**.

---

## Build Artifacts Naming

| Platform | File Name Format |
|----------|------------------|
| macOS (Apple Silicon) | `Mantra_v0.1.0_macos-arm64.dmg` |
| macOS (Intel) | `Mantra_v0.1.0_macos-x64.dmg` |
| Windows (MSI) | `Mantra_v0.1.0_windows-x64.msi` |
| Windows (EXE) | `Mantra_v0.1.0_windows-x64.exe` |
| Linux (AppImage) | `Mantra_v0.1.0_linux-x64.AppImage` |
| Linux (Deb) | `Mantra_v0.1.0_linux-x64.deb` |

---

## Troubleshooting

### Build Failed?

1. Check Actions logs to identify the error
2. Fix code issues
3. Delete failed tag: `git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`
4. Re-create tag and push

### Public Repo Publishing Failed?

Use **"Publish to Public Repository"** workflow to re-sync.

### How to Withdraw a Version?

1. Delete the Release from public repo (manually on GitHub)
2. Delete the Release from private repo
3. Delete Git Tag: `git push origin :refs/tags/v0.1.0`

### macOS Shows "Unverified Developer" Warning?

Since Mantra is not currently Apple code-signed, first launch requires manual authorization:

1. **Right-click open**: Control-click the app in Finder → Select "Open" → Click "Open" again
2. **System Settings**: System Settings → Privacy & Security → Find the blocked message → Click "Open Anyway"
3. **Terminal**: `xattr -cr /Applications/Mantra.app`

---

## Related Files

| File | Description |
|------|-------------|
| `.github/workflows/release.yml` | Main release workflow |
| `.github/workflows/publish-public.yml` | Public repo sync workflow |
| `docs/cross-repo-release-setup.md` | Cross-repo release setup guide |
