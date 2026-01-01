/**
 * file-icons - 文件类型图标映射
 * Story 2.13: Task 2
 *
 * 根据文件扩展名返回对应的 Lucide 图标组件
 */

import {
    FileCode,
    FileJson,
    FileText,
    FileType,
    File,
    FileImage,
    FileVideo,
    FileAudio,
    FileArchive,
    Braces,
    Hash,
    Database,
    Settings,
    Globe,
    Palette,
    Box,
    FileCheck,
    Lock,
    type LucideIcon,
} from "lucide-react";

/**
 * 扩展名到图标的映射
 */
const extensionIconMap: Record<string, LucideIcon> = {
    // TypeScript / JavaScript
    ts: FileCode,
    tsx: FileCode,
    js: FileCode,
    jsx: FileCode,
    mjs: FileCode,
    cjs: FileCode,

    // Web
    html: Globe,
    htm: Globe,
    css: Palette,
    scss: Palette,
    sass: Palette,
    less: Palette,

    // Data
    json: FileJson,
    yaml: Settings,
    yml: Settings,
    toml: Settings,
    xml: Braces,

    // Rust
    rs: Braces,

    // Go
    go: Braces,

    // Python
    py: Hash,
    pyw: Hash,
    pyx: Hash,

    // Ruby
    rb: Hash,

    // PHP
    php: Hash,

    // Java / Kotlin
    java: FileCode,
    kt: FileCode,
    kts: FileCode,

    // C / C++ / C#
    c: FileCode,
    cpp: FileCode,
    cc: FileCode,
    cxx: FileCode,
    h: FileCode,
    hpp: FileCode,
    cs: FileCode,

    // Swift
    swift: FileCode,

    // Shell
    sh: FileText,
    bash: FileText,
    zsh: FileText,
    fish: FileText,
    ps1: FileText,

    // Markdown / Text
    md: FileType,
    mdx: FileType,
    txt: FileText,
    rtf: FileText,

    // Config
    env: Settings,
    ini: Settings,
    conf: Settings,
    config: Settings,

    // Database
    sql: Database,
    sqlite: Database,
    db: Database,

    // Images
    png: FileImage,
    jpg: FileImage,
    jpeg: FileImage,
    gif: FileImage,
    svg: FileImage,
    ico: FileImage,
    webp: FileImage,
    bmp: FileImage,

    // Video
    mp4: FileVideo,
    webm: FileVideo,
    avi: FileVideo,
    mov: FileVideo,
    mkv: FileVideo,

    // Audio
    mp3: FileAudio,
    wav: FileAudio,
    ogg: FileAudio,
    flac: FileAudio,
    aac: FileAudio,

    // Archives
    zip: FileArchive,
    tar: FileArchive,
    gz: FileArchive,
    rar: FileArchive,
    "7z": FileArchive,

    // Package / Build
    lock: Lock,
    log: FileText,

    // Test
    test: FileCheck,
    spec: FileCheck,

    // 3D / Binary
    wasm: Box,
    bin: Box,
};

/**
 * 特殊文件名到图标的映射
 */
const filenameIconMap: Record<string, LucideIcon> = {
    "package.json": FileJson,
    "tsconfig.json": Settings,
    "vite.config.ts": Settings,
    "vitest.config.ts": FileCheck,
    ".gitignore": Settings,
    ".env": Lock,
    ".env.local": Lock,
    ".env.development": Lock,
    ".env.production": Lock,
    "Dockerfile": Box,
    "Makefile": Settings,
    "Cargo.toml": Settings,
    "Cargo.lock": Lock,
    "pnpm-lock.yaml": Lock,
    "package-lock.json": Lock,
    "yarn.lock": Lock,
    "README.md": FileType,
    "LICENSE": FileText,
};

/**
 * 根据文件路径获取对应的图标组件
 * @param filePath - 文件路径
 * @returns Lucide 图标组件
 */
export function getFileIcon(filePath: string): LucideIcon {
    // 提取文件名
    const fileName = filePath.split("/").pop() || filePath;
    const lowerFileName = fileName.toLowerCase();

    // 检查特殊文件名
    if (filenameIconMap[fileName] || filenameIconMap[lowerFileName]) {
        return filenameIconMap[fileName] || filenameIconMap[lowerFileName];
    }

    // 检查测试文件
    if (fileName.includes(".test.") || fileName.includes(".spec.")) {
        return FileCheck;
    }

    // 提取扩展名
    const ext = fileName.split(".").pop()?.toLowerCase();

    if (ext && extensionIconMap[ext]) {
        return extensionIconMap[ext];
    }

    // 默认图标
    return File;
}

export default getFileIcon;


