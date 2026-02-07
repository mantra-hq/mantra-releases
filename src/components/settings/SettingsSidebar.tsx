/**
 * SettingsSidebar - 设置页面侧边栏导航组件
 * Story 2-35: Task 1
 *
 * VS Code 风格侧边栏，分为 3 个分组：通用、开发环境、隐私与安全
 */

import { NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import {
    Globe,
    HelpCircle,
    Server,
    FolderCog,
    KeyRound,
    ShieldCheck,
    FileEdit,
    FlaskConical,
    FileWarning,
} from 'lucide-react';
import { cn } from '@/lib/utils';

interface NavItem {
    labelKey: string;
    icon: React.ComponentType<{ className?: string }>;
}

interface NavGroup {
    labelKey: string;
    path: string;
    items: NavItem[];
}

const NAV_GROUPS: NavGroup[] = [
    {
        labelKey: 'settings.nav.general',
        path: '/settings/general',
        items: [
            { labelKey: 'settings.nav.language', icon: Globe },
            { labelKey: 'settings.nav.helpAndSupport', icon: HelpCircle },
        ],
    },
    {
        labelKey: 'settings.nav.development',
        path: '/settings/development',
        items: [
            { labelKey: 'settings.nav.localServer', icon: Server },
            { labelKey: 'settings.nav.toolPaths', icon: FolderCog },
            { labelKey: 'settings.nav.envVariables', icon: KeyRound },
        ],
    },
    {
        labelKey: 'settings.nav.privacy',
        path: '/settings/privacy',
        items: [
            { labelKey: 'settings.nav.systemRules', icon: ShieldCheck },
            { labelKey: 'settings.nav.customRules', icon: FileEdit },
            { labelKey: 'settings.nav.ruleTest', icon: FlaskConical },
            { labelKey: 'settings.nav.privacyRecords', icon: FileWarning },
        ],
    },
];

export function SettingsSidebar() {
    const { t } = useTranslation();

    return (
        <nav
            className="w-[200px] shrink-0 border-r bg-zinc-900/50 overflow-y-auto"
            data-testid="settings-sidebar"
        >
            <div className="py-4 space-y-4">
                {NAV_GROUPS.map((group) => (
                    <div key={group.path}>
                        {/* Group title — clickable NavLink */}
                        <NavLink
                            to={group.path}
                            replace
                            className={({ isActive }) =>
                                cn(
                                    'flex items-center px-4 py-2 text-sm font-medium transition-colors',
                                    isActive
                                        ? 'text-foreground border-l-2 border-blue-500 bg-blue-500/5'
                                        : 'text-muted-foreground hover:text-foreground border-l-2 border-transparent'
                                )
                            }
                            data-testid={`settings-nav-${group.path.split('/').pop()}`}
                        >
                            {t(group.labelKey)}
                        </NavLink>

                        {/* Sub-items (display-only labels showing group contents) */}
                        <div className="mt-1 space-y-0.5">
                            {group.items.map((item) => {
                                const Icon = item.icon;
                                return (
                                    <div
                                        key={item.labelKey}
                                        className="flex items-center gap-2 pl-6 pr-4 py-1.5 text-xs text-muted-foreground"
                                    >
                                        <Icon className="h-3.5 w-3.5" />
                                        {t(item.labelKey)}
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                ))}
            </div>
        </nav>
    );
}
