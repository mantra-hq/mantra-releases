/**
 * LanguageSwitcher Component - 语言切换组件
 * Story 2-26: Task 4
 *
 * 用于切换应用语言（简体中文/English）
 * - 下拉选择器样式
 * - 自动持久化到 localStorage
 */

import { useTranslation } from "react-i18next";
import { Globe } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const languages = [
  { value: "zh-CN", label: "简体中文" },
  { value: "en", label: "English" },
] as const;

type LanguageCode = (typeof languages)[number]["value"];

export function LanguageSwitcher() {
  const { t, i18n } = useTranslation();

  const handleLanguageChange = (value: LanguageCode) => {
    i18n.changeLanguage(value);
  };

  return (
    <div className="flex items-center justify-between py-3">
      <div className="flex items-center gap-2">
        <Globe className="h-4 w-4 text-muted-foreground" />
        <span className="text-sm font-medium">{t("language.title")}</span>
      </div>
      <Select
        value={i18n.language}
        onValueChange={handleLanguageChange}
      >
        <SelectTrigger className="w-[140px]" data-testid="language-select">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {languages.map((lang) => (
            <SelectItem key={lang.value} value={lang.value}>
              {lang.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
