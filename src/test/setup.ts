import "@testing-library/jest-dom/vitest";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import zhCN from "../i18n/locales/zh-CN.json";
import en from "../i18n/locales/en.json";

// Story 2-26: 初始化 i18n 用于测试环境
// 测试环境使用简化配置，默认中文
// 注意: 需要同步初始化以避免测试中的竞态条件
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
