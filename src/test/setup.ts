import "@testing-library/jest-dom/vitest";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import zhCN from "../i18n/locales/zh-CN.json";
import en from "../i18n/locales/en.json";

// Story 2-26: 初始化 i18n 用于测试环境
// 测试环境使用简化配置，默认中文
// 注意: 需要同步初始化以避免测试中的竞态条件
if (!i18n.isInitialized) {
  i18n
    .use(initReactI18next)
    .init({
      resources: {
        "zh-CN": { translation: zhCN },
        en: { translation: en },
      },
      lng: "zh-CN", // 测试环境默认使用中文
      fallbackLng: "zh-CN",
      interpolation: {
        escapeValue: false,
      },
      // 禁用调试输出
      debug: false,
      // 确保同步初始化
      initAsync: false,
    });
}

const shouldIgnoreDialogTitleWarning = (args: unknown[]) =>
  args.some((arg) => {
    if (typeof arg === "string") {
      return arg.includes("DialogContent requires a `DialogTitle`");
    }
    if (arg instanceof Error) {
      return arg.message.includes("DialogContent requires a `DialogTitle`");
    }
    return false;
  });

const originalConsoleError = console.error;
console.error = (...args) => {
  if (shouldIgnoreDialogTitleWarning(args)) {
    return;
  }
  originalConsoleError(...args);
};

const originalConsoleWarn = console.warn;
console.warn = (...args) => {
  if (shouldIgnoreDialogTitleWarning(args)) {
    return;
  }
  originalConsoleWarn(...args);
};

const originalStderrWrite = process.stderr.write.bind(process.stderr);
process.stderr.write = ((chunk: string | Uint8Array, encoding?: BufferEncoding, cb?: (err?: Error) => void) => {
  const text = typeof chunk === "string" ? chunk : Buffer.from(chunk).toString("utf8");
  if (
    text.includes("DialogContent requires a `DialogTitle`") ||
    text.includes("If you want to hide the `DialogTitle`") ||
    text.includes("radix-ui.com/primitives/docs/components/dialog")
  ) {
    if (typeof cb === "function") {
      cb();
    }
    return true;
  }
  return originalStderrWrite(chunk as never, encoding as never, cb as never);
}) as typeof process.stderr.write;
