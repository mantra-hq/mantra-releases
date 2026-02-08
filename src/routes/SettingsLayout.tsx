/**
 * Settings Page - 设置页面布局容器
 * Story 2-35: Task 2 - 重构为侧边栏 + Outlet 布局
 *
 * 左侧固定 SettingsSidebar + 右侧 Outlet 渲染子路由内容
 */

import { useNavigate } from 'react-router-dom';
import { Outlet } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Settings as SettingsIcon } from 'lucide-react';
import { SettingsSidebar } from '@/components/settings/SettingsSidebar';

export function Settings() {
    const { t } = useTranslation();
    const navigate = useNavigate();

    return (
        <div className="h-screen flex flex-col bg-background">
            {/* Header */}
            <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                <div className="container flex h-14 items-center px-4">
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => navigate(-1)}
                        aria-label={t("common.back")}
                        data-testid="back-button"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div className="flex items-center gap-2 ml-2">
                        <SettingsIcon className="h-5 w-5" />
                        <h1 className="text-lg font-semibold">{t("settings.title")}</h1>
                    </div>
                </div>
            </header>

            {/* Sidebar + Content */}
            <div className="flex flex-1 overflow-hidden">
                <SettingsSidebar />
                <main className="flex-1 overflow-y-auto px-6 py-6 max-w-4xl mx-auto">
                    <Outlet />
                </main>
            </div>
        </div>
    );
}

export default Settings;
