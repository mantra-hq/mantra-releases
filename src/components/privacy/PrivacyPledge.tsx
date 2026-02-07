/**
 * PrivacyPledge Component - 隐私宣言组件
 *
 * 非侵入式隐私承诺展示，用于空状态首页底部
 * 让用户在自然操作流中感知 Mantra 的隐私保护策略
 */

import { ShieldCheck, HardDrive, Lock, Eye, CloudOff } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";

const pledgeItems = [
  { icon: HardDrive, key: "localFirst" as const },
  { icon: Lock, key: "encrypted" as const },
  { icon: Eye, key: "reviewBeforeUpload" as const },
  { icon: CloudOff, key: "neverAutoUpload" as const },
] as const;

export function PrivacyPledge() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <div
      data-testid="privacy-pledge"
      className={cn(
        "mt-10 pt-6",
        "border-t border-border/40",
        "max-w-lg mx-auto"
      )}
    >
      {/* 标题行 */}
      <div className="flex items-center justify-center gap-1.5 mb-4">
        <ShieldCheck className="w-4 h-4 text-emerald-500" />
        <span className="text-xs font-medium text-muted-foreground">
          {t("privacy.pledge.title")}
        </span>
      </div>

      {/* 承诺项 */}
      <div className="grid grid-cols-2 gap-x-6 gap-y-3">
        {pledgeItems.map(({ icon: Icon, key }) => (
          <div key={key} className="flex items-start gap-2">
            <Icon className="w-3.5 h-3.5 mt-0.5 text-muted-foreground/70 shrink-0" />
            <div>
              <p className="text-xs font-medium text-muted-foreground">
                {t(`privacy.pledge.${key}`)}
              </p>
              <p className="text-[11px] text-muted-foreground/60 leading-tight">
                {t(`privacy.pledge.${key}Desc`)}
              </p>
            </div>
          </div>
        ))}
      </div>

      {/* 了解更多 */}
      <div className="mt-4 text-center">
        <button
          onClick={() => navigate("/settings/privacy")}
          className={cn(
            "text-[11px] text-muted-foreground/50",
            "hover:text-muted-foreground transition-colors",
            "underline-offset-2 hover:underline"
          )}
        >
          {t("privacy.pledge.learnMore")}
        </button>
      </div>
    </div>
  );
}
