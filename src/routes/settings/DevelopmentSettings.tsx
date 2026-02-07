/**
 * DevelopmentSettings - 开发环境设置页面
 * Story 2-35: Task 3.2
 *
 * 包含 LocalServerConfig + ToolConfigPathManager + EnvVariableManager
 */

import { LocalServerConfig } from '@/components/settings/LocalServerConfig';
import { ToolConfigPathManager } from '@/components/settings/ToolConfigPathManager';
import { EnvVariableManager } from '@/components/hub/EnvVariableManager';

export function DevelopmentSettings() {
    return (
        <div className="space-y-8">
            {/* 本地 API Server 端口配置 */}
            <section className="rounded-lg border bg-card p-4">
                <LocalServerConfig />
            </section>

            {/* 工具配置路径管理 */}
            <section className="rounded-lg border bg-card p-4">
                <ToolConfigPathManager />
            </section>

            {/* 环境变量管理 */}
            <section className="rounded-lg border bg-card p-4">
                <EnvVariableManager />
            </section>
        </div>
    );
}

export default DevelopmentSettings;
