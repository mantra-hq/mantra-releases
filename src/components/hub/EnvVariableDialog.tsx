/**
 * 环境变量添加/编辑对话框
 * Story 11.4: 环境变量管理 - Task 4.4
 *
 * 支持添加新变量和编辑现有变量
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { AlertCircle, Loader2, Sparkles } from "lucide-react";
import {
  setEnvVariable,
  validateEnvVarNameSync,
  getEnvVariableDecrypted,
  type EnvVariable,
} from "@/lib/env-variable-ipc";
import { feedback } from "@/lib/feedback";

interface EnvVariableDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** 编辑模式时传入现有变量，添加模式时为 null */
  editVariable: EnvVariable | null;
  onSuccess: () => void;
}

export function EnvVariableDialog({
  open,
  onOpenChange,
  editVariable,
  onSuccess,
}: EnvVariableDialogProps) {
  const { t } = useTranslation();
  const isEditMode = editVariable !== null;

  const [name, setName] = useState("");
  const [value, setValue] = useState("");
  const [description, setDescription] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isLoadingValue, setIsLoadingValue] = useState(false);
  const [nameError, setNameError] = useState<string | null>(null);
  const [nameSuggestion, setNameSuggestion] = useState<string | null>(null);

  // 初始化表单
  useEffect(() => {
    if (open) {
      if (editVariable) {
        setName(editVariable.name);
        setDescription(editVariable.description || "");
        // 编辑模式下加载解密后的值
        setIsLoadingValue(true);
        getEnvVariableDecrypted(editVariable.name)
          .then((decrypted) => {
            setValue(decrypted || "");
          })
          .catch((error) => {
            console.error("[EnvVariableDialog] Failed to load value:", error);
            setValue("");
          })
          .finally(() => {
            setIsLoadingValue(false);
          });
      } else {
        setName("");
        setValue("");
        setDescription("");
      }
      setNameError(null);
      setNameSuggestion(null);
    }
  }, [open, editVariable]);

  // 变量名校验
  const handleNameChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newName = e.target.value;
    setName(newName);

    if (newName.trim() === "") {
      setNameError(null);
      setNameSuggestion(null);
      return;
    }

    const validation = validateEnvVarNameSync(newName);
    if (validation.is_valid) {
      setNameError(null);
      setNameSuggestion(null);
    } else {
      setNameError(validation.error_message);
      setNameSuggestion(validation.suggestion);
    }
  }, []);

  // 应用建议的变量名
  const handleApplySuggestion = useCallback(() => {
    if (nameSuggestion) {
      setName(nameSuggestion);
      setNameError(null);
      setNameSuggestion(null);
    }
  }, [nameSuggestion]);

  // 保存
  const handleSave = useCallback(async () => {
    // 最终校验
    const validation = validateEnvVarNameSync(name);
    if (!validation.is_valid) {
      setNameError(validation.error_message);
      setNameSuggestion(validation.suggestion);
      return;
    }

    if (value.trim() === "") {
      feedback.error(t("hub.envVariables.valueRequired"));
      return;
    }

    setIsSaving(true);
    try {
      await setEnvVariable(name, value, description || undefined);
      feedback.saved(
        isEditMode
          ? t("hub.envVariables.updateSuccess")
          : t("hub.envVariables.addSuccess")
      );
      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error("[EnvVariableDialog] Failed to save:", error);
      feedback.error(
        isEditMode
          ? t("hub.envVariables.updateError")
          : t("hub.envVariables.addError")
      );
    } finally {
      setIsSaving(false);
    }
  }, [name, value, description, isEditMode, onSuccess, onOpenChange, t]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>
            {isEditMode
              ? t("hub.envVariables.editTitle")
              : t("hub.envVariables.addTitle")}
          </DialogTitle>
          <DialogDescription>
            {isEditMode
              ? t("hub.envVariables.editDescription")
              : t("hub.envVariables.addDescription")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* 变量名 */}
          <div className="space-y-2">
            <Label htmlFor="env-name">{t("hub.envVariables.nameLabel")}</Label>
            <Input
              id="env-name"
              value={name}
              onChange={handleNameChange}
              placeholder="OPENAI_API_KEY"
              disabled={isEditMode}
              className={nameError ? "border-destructive" : ""}
              data-testid="env-variable-name-input"
            />
            {nameError && (
              <div className="flex items-center gap-2 text-sm text-destructive">
                <AlertCircle className="h-4 w-4 flex-shrink-0" />
                <span>{nameError}</span>
              </div>
            )}
            {nameSuggestion && (
              <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">
                  {t("hub.envVariables.suggestion")}:
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleApplySuggestion}
                  className="h-7 text-xs"
                >
                  <Sparkles className="h-3 w-3 mr-1" />
                  {nameSuggestion}
                </Button>
              </div>
            )}
          </div>

          {/* 变量值 */}
          <div className="space-y-2">
            <Label htmlFor="env-value">{t("hub.envVariables.valueLabel")}</Label>
            {isLoadingValue ? (
              <div className="flex items-center justify-center h-20 border rounded-md">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              </div>
            ) : (
              <Textarea
                id="env-value"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                placeholder="sk-..."
                className="font-mono text-sm"
                rows={3}
                data-testid="env-variable-value-input"
              />
            )}
            <p className="text-xs text-muted-foreground">
              {t("hub.envVariables.valueHint")}
            </p>
          </div>

          {/* 描述 */}
          <div className="space-y-2">
            <Label htmlFor="env-description">
              {t("hub.envVariables.descriptionLabel")}
              <span className="text-muted-foreground ml-1">
                ({t("common.optional")})
              </span>
            </Label>
            <Input
              id="env-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t("hub.envVariables.descriptionPlaceholder")}
              data-testid="env-variable-description-input"
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSaving}
          >
            {t("common.cancel")}
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving || isLoadingValue || !name.trim() || !value.trim()}
            data-testid="env-variable-save-button"
          >
            {isSaving && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {isEditMode ? t("common.save") : t("common.add")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default EnvVariableDialog;
