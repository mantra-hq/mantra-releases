/**
 * log-actions - 日志记录辅助函数
 * Story 2.28: 运行日志复制
 *
 * 提供在关键操作处记录日志的便捷函数
 * 与 useLogStore 配合使用
 */

import { useLogStore } from "@/stores";

/**
 * 获取 store 的 getState - 可在 React 外部调用
 */
const getLogStore = () => useLogStore.getState();

/**
 * 日志记录工具
 */
export const appLog = {
    /**
     * 记录导入开始
     */
    importStart: (source: string, fileCount: number) => {
        getLogStore().info(
            "Import started",
            `Source: ${source}, Files: ${fileCount}`
        );
    },

    /**
     * 记录导入文件成功
     */
    importFileSuccess: (filePath: string, projectName: string) => {
        getLogStore().info(
            "File imported",
            `${filePath} → ${projectName}`
        );
    },

    /**
     * 记录导入文件失败
     */
    importFileError: (filePath: string, error: string) => {
        getLogStore().error(
            "Import failed",
            `${filePath}: ${error}`
        );
    },

    /**
     * 记录导入完成
     */
    importComplete: (successCount: number, failCount: number) => {
        const level = failCount > 0 ? "warn" : "info";
        getLogStore().addLog(
            level,
            "Import completed",
            `Success: ${successCount}, Failed: ${failCount}`
        );
    },

    /**
     * 记录同步开始
     */
    syncStart: (projectName: string) => {
        getLogStore().info("Sync started", projectName);
    },

    /**
     * 记录同步结果
     */
    syncComplete: (
        projectName: string,
        newSessions: number,
        updatedSessions: number
    ) => {
        getLogStore().info(
            "Sync completed",
            `${projectName}: New sessions: ${newSessions}, Updated: ${updatedSessions}`
        );
    },

    /**
     * 记录同步错误
     */
    syncError: (projectName: string, error: string) => {
        getLogStore().error("Sync failed", `${projectName}: ${error}`);
    },

    /**
     * 记录项目移除
     */
    projectRemoved: (projectName: string) => {
        getLogStore().info("Project removed", projectName);
    },

    /**
     * 记录项目重命名
     */
    projectRenamed: (oldName: string, newName: string) => {
        getLogStore().info("Project renamed", `${oldName} → ${newName}`);
    },

    /**
     * 记录一般信息
     */
    info: (action: string, details?: string) => {
        getLogStore().info(action, details);
    },

    /**
     * 记录警告
     */
    warn: (action: string, details?: string) => {
        getLogStore().warn(action, details);
    },

    /**
     * 记录错误
     */
    error: (action: string, details?: string) => {
        getLogStore().error(action, details);
    },
};

export default appLog;
