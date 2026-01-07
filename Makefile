# Mantra Client 构建与发布管理
# ========================================

SHELL := /bin/bash
.DEFAULT_GOAL := help

# ----------------------------------------
# 变量定义
# ----------------------------------------

# 项目根目录
PROJECT_ROOT := $(shell pwd)

# 版本信息
VERSION := $(shell cat VERSION 2>/dev/null || echo "0.0.0")
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
BUILD_TIME := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# 目录定义
TAURI_DIR := src-tauri
DIST_DIR := dist
RELEASE_DIR := release

# 颜色定义
CYAN := \033[36m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

# ----------------------------------------
# 帮助信息
# ----------------------------------------

.PHONY: help
help: ## 显示帮助信息
	@echo ""
	@echo "$(CYAN)Mantra Client 构建与发布管理$(RESET)"
	@echo "$(CYAN)========================================$(RESET)"
	@echo "当前版本: $(GREEN)$(VERSION)$(RESET)"
	@echo "Git 提交: $(GIT_COMMIT)"
	@echo "Git 分支: $(GIT_BRANCH)"
	@echo ""
	@echo "$(YELLOW)可用命令:$(RESET)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; section=""} \
		/^## / { section=substr($$0, 4); printf "\n$(CYAN)%s$(RESET)\n", section } \
		/^[a-zA-Z0-9_-]+:.*##/ { printf "  $(GREEN)%-20s$(RESET) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo ""

# ----------------------------------------
## 开发命令
# ----------------------------------------

.PHONY: dev
dev: ## 启动开发服务器 (前端 + Tauri)
	@echo "$(CYAN)启动开发服务器...$(RESET)"
	pnpm tauri dev

.PHONY: dev-web
dev-web: ## 仅启动前端开发服务器
	@echo "$(CYAN)启动前端开发服务器...$(RESET)"
	pnpm dev

.PHONY: install
install: ## 安装所有依赖
	@echo "$(CYAN)安装项目依赖...$(RESET)"
	pnpm install
	cd $(TAURI_DIR) && cargo fetch
	@echo "$(GREEN)依赖安装完成$(RESET)"

.PHONY: clean
clean: ## 清理构建产物
	@echo "$(CYAN)清理构建产物...$(RESET)"
	rm -rf $(DIST_DIR)
	rm -rf $(RELEASE_DIR)
	cd $(TAURI_DIR) && cargo clean
	@echo "$(GREEN)清理完成$(RESET)"

# ----------------------------------------
## 构建命令
# ----------------------------------------

.PHONY: build
build: build-web build-tauri ## 完整构建 (前端 + Tauri)

.PHONY: build-web
build-web: ## 构建前端
	@echo "$(CYAN)构建前端...$(RESET)"
	pnpm build
	@echo "$(GREEN)前端构建完成$(RESET)"

.PHONY: build-tauri
build-tauri: ## 构建 Tauri 应用 (所有平台格式)
	@echo "$(CYAN)构建 Tauri 应用...$(RESET)"
	pnpm tauri build
	@echo "$(GREEN)Tauri 构建完成$(RESET)"

.PHONY: build-debug
build-debug: ## 构建 Debug 版本
	@echo "$(CYAN)构建 Debug 版本...$(RESET)"
	pnpm tauri build --debug
	@echo "$(GREEN)Debug 构建完成$(RESET)"

# ----------------------------------------
## 平台特定构建
# ----------------------------------------

.PHONY: build-linux
build-linux: ## 构建 Linux 版本 (deb + rpm + AppImage)
	@echo "$(CYAN)构建 Linux 版本...$(RESET)"
	pnpm tauri build --bundles deb,rpm,appimage
	@echo "$(GREEN)Linux 构建完成$(RESET)"

.PHONY: build-deb
build-deb: ## 构建 Linux DEB 包
	@echo "$(CYAN)构建 Linux DEB 包...$(RESET)"
	pnpm tauri build --bundles deb
	@echo "$(GREEN)DEB 构建完成$(RESET)"

.PHONY: build-rpm
build-rpm: ## 构建 Linux RPM 包
	@echo "$(CYAN)构建 Linux RPM 包...$(RESET)"
	pnpm tauri build --bundles rpm
	@echo "$(GREEN)RPM 构建完成$(RESET)"

.PHONY: build-appimage
build-appimage: ## 构建 Linux AppImage
	@echo "$(CYAN)构建 Linux AppImage...$(RESET)"
	pnpm tauri build --bundles appimage
	@echo "$(GREEN)AppImage 构建完成$(RESET)"

.PHONY: build-macos
build-macos: ## 构建 macOS 版本 (dmg + app)
	@echo "$(CYAN)构建 macOS 版本...$(RESET)"
	pnpm tauri build --bundles dmg,app
	@echo "$(GREEN)macOS 构建完成$(RESET)"

.PHONY: build-dmg
build-dmg: ## 构建 macOS DMG
	@echo "$(CYAN)构建 macOS DMG...$(RESET)"
	pnpm tauri build --bundles dmg
	@echo "$(GREEN)DMG 构建完成$(RESET)"

.PHONY: build-windows
build-windows: ## 构建 Windows 版本 (msi + nsis)
	@echo "$(CYAN)构建 Windows 版本...$(RESET)"
	pnpm tauri build --bundles msi,nsis
	@echo "$(GREEN)Windows 构建完成$(RESET)"

.PHONY: build-msi
build-msi: ## 构建 Windows MSI 安装包
	@echo "$(CYAN)构建 Windows MSI...$(RESET)"
	pnpm tauri build --bundles msi
	@echo "$(GREEN)MSI 构建完成$(RESET)"

.PHONY: build-nsis
build-nsis: ## 构建 Windows NSIS 安装包
	@echo "$(CYAN)构建 Windows NSIS...$(RESET)"
	pnpm tauri build --bundles nsis
	@echo "$(GREEN)NSIS 构建完成$(RESET)"

# ----------------------------------------
## 版本管理
# ----------------------------------------

.PHONY: version
version: ## 显示当前版本信息
	@./scripts/version.sh show

.PHONY: version-set
version-set: ## 设置版本号 (用法: make version-set V=1.0.0)
ifndef V
	@echo "$(RED)错误: 请指定版本号$(RESET)"
	@echo "用法: make version-set V=1.0.0"
	@exit 1
endif
	@./scripts/version.sh set $(V)

.PHONY: version-bump-patch
version-bump-patch: ## 增加补丁版本号 (0.1.0 -> 0.1.1)
	@./scripts/version.sh bump patch

.PHONY: version-bump-minor
version-bump-minor: ## 增加次版本号 (0.1.0 -> 0.2.0)
	@./scripts/version.sh bump minor

.PHONY: version-bump-major
version-bump-major: ## 增加主版本号 (0.1.0 -> 1.0.0)
	@./scripts/version.sh bump major

.PHONY: version-sync
version-sync: ## 同步版本号到所有配置文件
	@./scripts/version.sh sync

# ----------------------------------------
## 发布流程
# ----------------------------------------

.PHONY: release
release: pre-release build collect-artifacts post-release ## 执行完整发布流程

.PHONY: pre-release
pre-release: ## 发布前检查
	@echo "$(CYAN)执行发布前检查...$(RESET)"
	@echo "版本: $(VERSION)"
	@echo "分支: $(GIT_BRANCH)"
	@echo ""
	@# 检查是否有未提交的更改
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "$(YELLOW)警告: 存在未提交的更改$(RESET)"; \
		git status --short; \
	fi
	@# 检查版本号是否同步
	@./scripts/version.sh sync
	@echo "$(GREEN)发布前检查完成$(RESET)"

.PHONY: collect-artifacts
collect-artifacts: ## 收集构建产物到 release 目录
	@echo "$(CYAN)收集构建产物...$(RESET)"
	@mkdir -p $(RELEASE_DIR)/$(VERSION)
	@# 收集 Linux 产物
	@if [ -d "$(TAURI_DIR)/target/release/bundle/deb" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/deb/*.deb $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@if [ -d "$(TAURI_DIR)/target/release/bundle/rpm" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/rpm/*.rpm $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@if [ -d "$(TAURI_DIR)/target/release/bundle/appimage" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/appimage/*.AppImage $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@# 收集 macOS 产物
	@if [ -d "$(TAURI_DIR)/target/release/bundle/dmg" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/dmg/*.dmg $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@# 收集 Windows 产物
	@if [ -d "$(TAURI_DIR)/target/release/bundle/msi" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/msi/*.msi $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@if [ -d "$(TAURI_DIR)/target/release/bundle/nsis" ]; then \
		cp -r $(TAURI_DIR)/target/release/bundle/nsis/*.exe $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || true; \
	fi
	@echo "$(GREEN)构建产物已收集到: $(RELEASE_DIR)/$(VERSION)/$(RESET)"
	@ls -la $(RELEASE_DIR)/$(VERSION)/ 2>/dev/null || echo "$(YELLOW)暂无构建产物$(RESET)"

.PHONY: post-release
post-release: ## 发布后操作
	@echo "$(CYAN)发布后操作...$(RESET)"
	@echo "$(GREEN)版本 $(VERSION) 发布完成!$(RESET)"
	@echo ""
	@echo "下一步操作建议:"
	@echo "  1. 创建 Git 标签: git tag -a v$(VERSION) -m 'Release $(VERSION)'"
	@echo "  2. 推送标签: git push origin v$(VERSION)"
	@echo "  3. 在 GitHub 创建 Release"

.PHONY: release-linux
release-linux: pre-release build-linux collect-artifacts ## 发布 Linux 版本

.PHONY: release-macos
release-macos: pre-release build-macos collect-artifacts ## 发布 macOS 版本

.PHONY: release-windows
release-windows: pre-release build-windows collect-artifacts ## 发布 Windows 版本

# ----------------------------------------
## Git 标签管理
# ----------------------------------------

.PHONY: tag
tag: ## 创建版本标签 (用法: make tag 或 make tag V=1.0.0)
	@version=$${V:-$(VERSION)}; \
	echo "$(CYAN)创建标签 v$$version...$(RESET)"; \
	git tag -a "v$$version" -m "Release $$version"; \
	echo "$(GREEN)标签 v$$version 创建成功$(RESET)"; \
	echo "推送标签: git push origin v$$version"

.PHONY: tag-push
tag-push: ## 推送当前版本标签到远程
	@echo "$(CYAN)推送标签 v$(VERSION) 到远程...$(RESET)"
	@git push origin "v$(VERSION)"
	@echo "$(GREEN)标签推送成功$(RESET)"

.PHONY: tag-delete
tag-delete: ## 删除版本标签 (用法: make tag-delete V=1.0.0)
ifndef V
	@echo "$(RED)错误: 请指定版本号$(RESET)"
	@echo "用法: make tag-delete V=1.0.0"
	@exit 1
endif
	@echo "$(CYAN)删除标签 v$(V)...$(RESET)"
	@git tag -d "v$(V)" 2>/dev/null || true
	@git push origin --delete "v$(V)" 2>/dev/null || true
	@echo "$(GREEN)标签 v$(V) 已删除$(RESET)"

# ----------------------------------------
## 测试与质量
# ----------------------------------------

.PHONY: test
test: test-web test-rust ## 运行所有测试

.PHONY: test-web
test-web: ## 运行前端测试
	@echo "$(CYAN)运行前端测试...$(RESET)"
	pnpm test:run

.PHONY: test-rust
test-rust: ## 运行 Rust 测试
	@echo "$(CYAN)运行 Rust 测试...$(RESET)"
	cd $(TAURI_DIR) && cargo test

.PHONY: lint
lint: ## 运行代码检查
	@echo "$(CYAN)运行代码检查...$(RESET)"
	pnpm lint
	cd $(TAURI_DIR) && cargo clippy

.PHONY: format
format: ## 格式化代码
	@echo "$(CYAN)格式化代码...$(RESET)"
	pnpm exec prettier --write .
	cd $(TAURI_DIR) && cargo fmt
	@echo "$(GREEN)代码格式化完成$(RESET)"

# ----------------------------------------
## 实用工具
# ----------------------------------------

.PHONY: info
info: ## 显示项目信息
	@echo ""
	@echo "$(CYAN)项目信息$(RESET)"
	@echo "=========================================="
	@echo "项目名称:   Mantra 心法 (客户端)"
	@echo "版本:       $(VERSION)"
	@echo "Git 提交:   $(GIT_COMMIT)"
	@echo "Git 分支:   $(GIT_BRANCH)"
	@echo "构建时间:   $(BUILD_TIME)"
	@echo ""
	@echo "$(CYAN)目录结构$(RESET)"
	@echo "项目根目录: $(PROJECT_ROOT)"
	@echo "Tauri:      $(TAURI_DIR)"
	@echo "发布目录:   $(RELEASE_DIR)"
	@echo ""

.PHONY: check-deps
check-deps: ## 检查依赖版本
	@echo "$(CYAN)检查依赖版本...$(RESET)"
	@echo ""
	@echo "Node.js: $$(node --version 2>/dev/null || echo '未安装')"
	@echo "pnpm:    $$(pnpm --version 2>/dev/null || echo '未安装')"
	@echo "Rust:    $$(rustc --version 2>/dev/null || echo '未安装')"
	@echo "Cargo:   $$(cargo --version 2>/dev/null || echo '未安装')"
	@echo "Tauri:   $$(pnpm tauri --version 2>/dev/null || echo '未安装')"
	@echo ""

.PHONY: update-deps
update-deps: ## 更新项目依赖
	@echo "$(CYAN)更新项目依赖...$(RESET)"
	pnpm update
	cd $(TAURI_DIR) && cargo update
	@echo "$(GREEN)依赖更新完成$(RESET)"
