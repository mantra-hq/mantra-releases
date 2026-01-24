/**
 * i18n Configuration - 国际化配置
 * Story 2-26: 国际化支持
 *
 * 使用 i18next + react-i18next 实现中英文双语支持
 * - 自动检测系统语言
 * - localStorage 持久化语言偏好
 * - 支持简体中文和英文
 */

import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import zhCN from "./locales/zh-CN.json";
import en from "./locales/en.json";

// 语言资源
const resources = {
  "zh-CN": {
    translation: zhCN,
  },
  en: {
    translation: en,
  },
};

if (!i18n.isInitialized) {
  i18n
    // 自动检测用户语言
    .use(LanguageDetector)
    // 传递 i18n 实例给 react-i18next
    .use(initReactI18next)
    // 初始化 i18next
    .init({
      resources,
      fallbackLng: "en", // 默认回退语言
      supportedLngs: ["zh-CN", "zh", "en"], // 支持的语言

      // 语言检测配置
      detection: {
        // 检测顺序：localStorage -> navigator (系统语言)
        order: ["localStorage", "navigator"],
        // localStorage 存储 key
        lookupLocalStorage: "mantra-language",
        // 缓存用户语言选择到 localStorage
        caches: ["localStorage"],
        // 将检测到的语言代码转换为支持的语言
        convertDetectedLanguage: (lng: string) => {
          // zh, zh-CN, zh-Hans, zh-Hans-CN 等都映射到 zh-CN
          if (lng.startsWith("zh")) {
            return "zh-CN";
          }
          // en, en-US, en-GB 等都映射到 en
          if (lng.startsWith("en")) {
            return "en";
          }
          return lng;
        },
      },

      interpolation: {
        escapeValue: false, // React 已经默认安全处理
      },

      // 开发环境输出调试信息
      debug: import.meta.env.DEV,
    });
}

export default i18n;
