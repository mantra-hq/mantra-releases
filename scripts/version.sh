#!/bin/bash
# 版本管理脚本
# 用于同步更新项目中所有版本号

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION_FILE="$PROJECT_ROOT/VERSION"
PACKAGE_JSON="$PROJECT_ROOT/package.json"
TAURI_CONF="$PROJECT_ROOT/src-tauri/tauri.conf.json"
CARGO_TOML="$PROJECT_ROOT/src-tauri/Cargo.toml"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 获取当前版本
get_current_version() {
    if [ -f "$VERSION_FILE" ]; then
        cat "$VERSION_FILE" | tr -d '\n'
    else
        echo "0.0.0"
    fi
}

# 验证版本号格式
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        print_error "无效的版本号格式: $version"
        print_info "版本号格式: MAJOR.MINOR.PATCH 或 MAJOR.MINOR.PATCH-PRERELEASE"
        print_info "示例: 1.0.0, 1.0.0-alpha.1, 1.0.0-beta.2"
        exit 1
    fi
}

# 解析版本号
parse_version() {
    local version=$1
    local base_version="${version%%-*}"
    IFS='.' read -r major minor patch <<< "$base_version"
    echo "$major $minor $patch"
}

# 增加版本号
bump_version() {
    local current=$1
    local bump_type=$2

    local base_version="${current%%-*}"
    IFS='.' read -r major minor patch <<< "$base_version"

    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            print_error "未知的版本类型: $bump_type (可用: major, minor, patch)"
            exit 1
            ;;
    esac

    echo "$major.$minor.$patch"
}

# 更新 package.json 版本
update_package_json() {
    local file=$1
    local version=$2

    if [ -f "$file" ]; then
        # 使用 node 来更新 JSON，确保格式正确
        node -e "
            const fs = require('fs');
            const pkg = JSON.parse(fs.readFileSync('$file', 'utf8'));
            pkg.version = '$version';
            fs.writeFileSync('$file', JSON.stringify(pkg, null, 2) + '\n');
        "
        print_success "已更新: $file"
    else
        print_warning "文件不存在: $file"
    fi
}

# 更新 tauri.conf.json 版本
update_tauri_conf() {
    local file=$1
    local version=$2

    if [ -f "$file" ]; then
        node -e "
            const fs = require('fs');
            const conf = JSON.parse(fs.readFileSync('$file', 'utf8'));
            conf.version = '$version';
            fs.writeFileSync('$file', JSON.stringify(conf, null, 2) + '\n');
        "
        print_success "已更新: $file"
    else
        print_warning "文件不存在: $file"
    fi
}

# 更新 Cargo.toml 版本
update_cargo_toml() {
    local file=$1
    local version=$2

    if [ -f "$file" ]; then
        # 使用 sed 更新版本行
        sed -i "s/^version = \".*\"/version = \"$version\"/" "$file"
        print_success "已更新: $file"
    else
        print_warning "文件不存在: $file"
    fi
}

# 同步所有版本
sync_versions() {
    local version=$1

    print_info "正在同步版本号到 $version ..."

    # 更新 VERSION 文件
    echo "$version" > "$VERSION_FILE"
    print_success "已更新: VERSION"

    # 更新所有配置文件
    update_package_json "$PACKAGE_JSON" "$version"
    update_tauri_conf "$TAURI_CONF" "$version"
    update_cargo_toml "$CARGO_TOML" "$version"

    print_success "所有版本已同步到 $version"
}

# 显示当前版本
show_version() {
    local current=$(get_current_version)
    echo -e "\n${BLUE}========== 版本信息 ==========${NC}"
    echo -e "当前版本: ${GREEN}$current${NC}\n"

    echo "各文件版本:"

    if [ -f "$PACKAGE_JSON" ]; then
        local pkg_ver=$(node -e "console.log(require('$PACKAGE_JSON').version)")
        echo "  - package.json:    $pkg_ver"
    fi

    if [ -f "$TAURI_CONF" ]; then
        local tauri_ver=$(node -e "console.log(require('$TAURI_CONF').version)")
        echo "  - tauri.conf.json: $tauri_ver"
    fi

    if [ -f "$CARGO_TOML" ]; then
        local cargo_ver=$(grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')
        echo "  - Cargo.toml:      $cargo_ver"
    fi

    echo -e "${BLUE}==============================${NC}\n"
}

# 帮助信息
show_help() {
    echo "
版本管理脚本

用法:
    $0 <command> [options]

命令:
    show                显示当前版本信息
    set <version>       设置指定版本号 (如: 1.0.0)
    bump <type>         增加版本号
                        type: major | minor | patch
    sync                同步 VERSION 文件中的版本到所有配置文件

示例:
    $0 show                 # 显示当前版本
    $0 set 1.0.0            # 设置版本为 1.0.0
    $0 set 1.0.0-beta.1     # 设置预发布版本
    $0 bump patch           # 0.1.0 -> 0.1.1
    $0 bump minor           # 0.1.0 -> 0.2.0
    $0 bump major           # 0.1.0 -> 1.0.0
    $0 sync                 # 同步版本到所有文件
"
}

# 主函数
main() {
    local command=${1:-show}

    case $command in
        show)
            show_version
            ;;
        set)
            local new_version=$2
            if [ -z "$new_version" ]; then
                print_error "请指定版本号"
                show_help
                exit 1
            fi
            validate_version "$new_version"
            sync_versions "$new_version"
            ;;
        bump)
            local bump_type=${2:-patch}
            local current=$(get_current_version)
            local new_version=$(bump_version "$current" "$bump_type")
            print_info "版本变更: $current -> $new_version"
            sync_versions "$new_version"
            ;;
        sync)
            local current=$(get_current_version)
            sync_versions "$current"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            print_error "未知命令: $command"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
