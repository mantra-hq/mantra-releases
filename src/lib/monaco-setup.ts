/**
 * Monaco Editor Setup - 本地资源加载配置
 *
 * 问题: @monaco-editor/react 默认从 CDN (unpkg.com) 加载 Monaco
 *       在 Tauri 打包环境下，CSP 策略阻止外部资源加载
 *
 * 解决: 配置 Monaco Loader 使用本地打包的 monaco-editor 资源
 *       同时配置 MonacoEnvironment 使 Workers 从本地加载
 *       这样打包后的 AppImage/msi/dmg 也能完全离线运行
 */

import { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";

// 配置 Monaco Workers 使用本地资源
// 参考: https://github.com/vitejs/vite/discussions/1791
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

// 配置 MonacoEnvironment 使用本地 Workers
self.MonacoEnvironment = {
    getWorker(_: unknown, label: string) {
        if (label === "json") {
            return new jsonWorker();
        }
        if (label === "css" || label === "scss" || label === "less") {
            return new cssWorker();
        }
        if (label === "html" || label === "handlebars" || label === "razor") {
            return new htmlWorker();
        }
        if (label === "typescript" || label === "javascript") {
            return new tsWorker();
        }
        return new editorWorker();
    },
};

// 配置 loader 使用本地 Monaco 实例
loader.config({ monaco });

// 预加载 Monaco 避免首次使用时的延迟
export function initMonaco(): Promise<typeof monaco> {
    return loader.init();
}
