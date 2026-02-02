/**
 * OAuth 配置 Sheet
 * Story 12.2: 简单表单 Dialog 改造为 Sheet - Task 2
 *
 * 支持配置远程 MCP 服务的 OAuth 认证：
 * - OAuth 2.0 配置 (Client ID, Secret, URLs, Scopes)
 * - Bearer Token 简化模式
 * - 连接/断开操作
 */

import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  Loader2,
  Link2,
  Unlink,
  Key,
  Shield,
  RefreshCw,
  AlertCircle,
  CheckCircle2,
  Clock,
} from "lucide-react";
import { feedback } from "@/lib/feedback";
import { toast } from "sonner";

/**
 * OAuth 服务状态
 */
export interface OAuthServiceStatus {
  service_id: string;
  status: "disconnected" | "connected" | "expired" | "pending";
  expires_at: string | null;
  scopes: string[];
  last_refreshed: string | null;
}

/**
 * OAuth 配置
 */
export interface OAuthConfig {
  service_id: string;
  client_id: string;
  client_secret?: string;
  authorization_url: string;
  token_url: string;
  revoke_url?: string;
  scopes: string[];
  callback_port: number;
}

interface OAuthConfigSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  serviceId: string;
  serviceName: string;
  onSuccess?: () => void;
}

export function OAuthConfigSheet({
  open,
  onOpenChange,
  serviceId,
  serviceName,
  onSuccess,
}: OAuthConfigSheetProps) {
  const { t } = useTranslation();
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<OAuthServiceStatus | null>(null);

  // 认证类型
  const [authType, setAuthType] = useState<"oauth" | "bearer">("oauth");

  // OAuth 配置
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [authorizationUrl, setAuthorizationUrl] = useState("");
  const [tokenUrl, setTokenUrl] = useState("");
  const [revokeUrl, setRevokeUrl] = useState("");
  const [scopes, setScopes] = useState("");

  // Bearer Token 配置
  const [bearerToken, setBearerToken] = useState("");

  // 加载状态
  useEffect(() => {
    if (open && serviceId) {
      loadStatus();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, serviceId]);

  const loadStatus = async () => {
    try {
      const result = await invoke<OAuthServiceStatus>("oauth_get_status", {
        serviceId,
      });
      setStatus(result);
    } catch (error) {
      console.error("[OAuthConfigSheet] Failed to load status:", error);
    }
  };

  // 启动 OAuth 流程
  const handleConnect = async () => {
    if (!clientId || !authorizationUrl || !tokenUrl) {
      feedback.error(
        t("hub.oauth.error"),
        t("hub.oauth.missingConfig")
      );
      return;
    }

    setIsLoading(true);
    try {
      await invoke("oauth_start_flow", {
        request: {
          service_id: serviceId,
          client_id: clientId,
          client_secret: clientSecret || null,
          authorization_url: authorizationUrl,
          token_url: tokenUrl,
          revoke_url: revokeUrl || null,
          scopes: scopes.split(",").map((s) => s.trim()).filter(Boolean),
          callback_port: 0, // 动态分配
        },
      });

      toast.success(t("hub.oauth.connecting"), {
        description: t("hub.oauth.browserOpened"),
      });

      // 等待一段时间后刷新状态
      setTimeout(loadStatus, 5000);
    } catch (error) {
      console.error("[OAuthConfigSheet] Failed to start OAuth flow:", error);
      feedback.error(
        t("hub.oauth.error"),
        (error as Error).message
      );
    } finally {
      setIsLoading(false);
    }
  };

  // 断开连接
  const handleDisconnect = async () => {
    setIsLoading(true);
    try {
      await invoke("oauth_disconnect", {
        serviceId,
        config: {
          service_id: serviceId,
          client_id: clientId,
          client_secret: clientSecret || null,
          authorization_url: authorizationUrl,
          token_url: tokenUrl,
          revoke_url: revokeUrl || null,
          scopes: scopes.split(",").map((s) => s.trim()).filter(Boolean),
          callback_port: 0,
        },
      });

      toast.success(t("hub.oauth.disconnected"), {
        description: t("hub.oauth.disconnectedDesc"),
      });

      await loadStatus();
      onSuccess?.();
    } catch (error) {
      console.error("[OAuthConfigSheet] Failed to disconnect:", error);
      feedback.error(
        t("hub.oauth.error"),
        (error as Error).message
      );
    } finally {
      setIsLoading(false);
    }
  };

  // 刷新 Token
  const handleRefresh = async () => {
    setIsLoading(true);
    try {
      await invoke("oauth_refresh_token", {
        serviceId,
        config: {
          service_id: serviceId,
          client_id: clientId,
          client_secret: clientSecret || null,
          authorization_url: authorizationUrl,
          token_url: tokenUrl,
          revoke_url: revokeUrl || null,
          scopes: scopes.split(",").map((s) => s.trim()).filter(Boolean),
          callback_port: 0,
        },
      });

      toast.success(t("hub.oauth.refreshed"), {
        description: t("hub.oauth.refreshedDesc"),
      });

      await loadStatus();
    } catch (error) {
      console.error("[OAuthConfigSheet] Failed to refresh token:", error);
      feedback.error(
        t("hub.oauth.error"),
        (error as Error).message
      );
    } finally {
      setIsLoading(false);
    }
  };

  // 渲染状态徽章
  const renderStatusBadge = () => {
    if (!status) return null;

    switch (status.status) {
      case "connected":
        return (
          <Badge variant="default" className="bg-emerald-500/20 text-emerald-400 border-emerald-500/30">
            <CheckCircle2 className="h-3 w-3 mr-1" />
            {t("hub.oauth.statusConnected")}
          </Badge>
        );
      case "expired":
        return (
          <Badge variant="default" className="bg-amber-500/20 text-amber-400 border-amber-500/30">
            <Clock className="h-3 w-3 mr-1" />
            {t("hub.oauth.statusExpired")}
          </Badge>
        );
      case "pending":
        return (
          <Badge variant="default" className="bg-blue-500/20 text-blue-400 border-blue-500/30">
            <Loader2 className="h-3 w-3 mr-1 animate-spin" />
            {t("hub.oauth.statusPending")}
          </Badge>
        );
      default:
        return (
          <Badge variant="secondary">
            <AlertCircle className="h-3 w-3 mr-1" />
            {t("hub.oauth.statusDisconnected")}
          </Badge>
        );
    }
  };

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-full max-w-lg overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            {t("hub.oauth.title", { name: serviceName })}
          </SheetTitle>
          <SheetDescription>
            {t("hub.oauth.description")}
          </SheetDescription>
        </SheetHeader>

        <div className="space-y-4 py-4 px-4">
          {/* 状态显示 */}
          <div className="flex items-center justify-between py-2 px-3 bg-muted/50 rounded-lg">
            <span className="text-sm text-muted-foreground">
              {t("hub.oauth.currentStatus")}
            </span>
            {renderStatusBadge()}
          </div>

          <Tabs value={authType} onValueChange={(v) => setAuthType(v as "oauth" | "bearer")}>
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="oauth">
                <Shield className="h-4 w-4 mr-2" />
                OAuth 2.0
              </TabsTrigger>
              <TabsTrigger value="bearer">
                <Key className="h-4 w-4 mr-2" />
                Bearer Token
              </TabsTrigger>
            </TabsList>

            <TabsContent value="oauth" className="space-y-4 mt-4">
              <div className="grid gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="clientId">{t("hub.oauth.clientId")} *</Label>
                  <Input
                    id="clientId"
                    value={clientId}
                    onChange={(e) => setClientId(e.target.value)}
                    placeholder="your-client-id"
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="clientSecret">{t("hub.oauth.clientSecret")}</Label>
                  <Input
                    id="clientSecret"
                    type="password"
                    value={clientSecret}
                    onChange={(e) => setClientSecret(e.target.value)}
                    placeholder={t("hub.oauth.clientSecretPlaceholder")}
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="authorizationUrl">{t("hub.oauth.authorizationUrl")} *</Label>
                  <Input
                    id="authorizationUrl"
                    value={authorizationUrl}
                    onChange={(e) => setAuthorizationUrl(e.target.value)}
                    placeholder="https://provider.com/oauth/authorize"
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="tokenUrl">{t("hub.oauth.tokenUrl")} *</Label>
                  <Input
                    id="tokenUrl"
                    value={tokenUrl}
                    onChange={(e) => setTokenUrl(e.target.value)}
                    placeholder="https://provider.com/oauth/token"
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="revokeUrl">{t("hub.oauth.revokeUrl")}</Label>
                  <Input
                    id="revokeUrl"
                    value={revokeUrl}
                    onChange={(e) => setRevokeUrl(e.target.value)}
                    placeholder="https://provider.com/oauth/revoke"
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="scopes">{t("hub.oauth.scopes")}</Label>
                  <Input
                    id="scopes"
                    value={scopes}
                    onChange={(e) => setScopes(e.target.value)}
                    placeholder="read, write, admin"
                  />
                  <p className="text-xs text-muted-foreground">
                    {t("hub.oauth.scopesHint")}
                  </p>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="bearer" className="space-y-4 mt-4">
              <div className="grid gap-2">
                <Label htmlFor="bearerToken">{t("hub.oauth.bearerToken")} *</Label>
                <Textarea
                  id="bearerToken"
                  value={bearerToken}
                  onChange={(e) => setBearerToken(e.target.value)}
                  placeholder={t("hub.oauth.bearerTokenPlaceholder")}
                  rows={3}
                />
                <p className="text-xs text-muted-foreground">
                  {t("hub.oauth.bearerTokenHint")}
                </p>
              </div>
            </TabsContent>
          </Tabs>

          {/* Token 信息 */}
          {status?.status === "connected" && status.expires_at && (
            <div className="text-xs text-muted-foreground bg-muted/30 p-3 rounded-lg">
              <div className="flex items-center gap-2">
                <Clock className="h-3 w-3" />
                <span>
                  {t("hub.oauth.expiresAt")}: {new Date(status.expires_at).toLocaleString()}
                </span>
              </div>
              {status.scopes.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {status.scopes.map((scope) => (
                    <Badge key={scope} variant="outline" className="text-xs">
                      {scope}
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <SheetFooter className="gap-2">
          {status?.status === "connected" || status?.status === "expired" ? (
            <>
              <Button
                variant="outline"
                onClick={handleRefresh}
                disabled={isLoading}
              >
                {isLoading ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <RefreshCw className="h-4 w-4 mr-2" />
                )}
                {t("hub.oauth.refresh")}
              </Button>
              <Button
                variant="destructive"
                onClick={handleDisconnect}
                disabled={isLoading}
              >
                {isLoading ? (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                ) : (
                  <Unlink className="h-4 w-4 mr-2" />
                )}
                {t("hub.oauth.disconnect")}
              </Button>
            </>
          ) : (
            <Button
              onClick={handleConnect}
              disabled={isLoading || (authType === "oauth" && (!clientId || !authorizationUrl || !tokenUrl))}
            >
              {isLoading ? (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              ) : (
                <Link2 className="h-4 w-4 mr-2" />
              )}
              {t("hub.oauth.connect")}
            </Button>
          )}
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}

export default OAuthConfigSheet;
