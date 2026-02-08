/**
 * useUpdateChecker - 客户端更新检查 Hook
 * Story 14.5: AC #1-#7
 *
 * 封装 @tauri-apps/plugin-updater 的完整更新逻辑：
 * - 启动延迟自动检查 + 24h 周期检查
 * - 静默下载 + 进度追踪
 * - 自动/手动检查的差异化错误处理
 * - Update 资源生命周期管理
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import type { Update, DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

// --- Types (AC #2) ---

export type UpdateStatus = 'idle' | 'checking' | 'downloading' | 'ready' | 'error';

export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

export interface UseUpdateCheckerResult {
  // 状态 (AC #2)
  updateAvailable: boolean;
  updateInfo: UpdateInfo | null;
  downloadProgress: number;
  updateStatus: UpdateStatus;
  errorMessage: string | null;
  // 方法 (AC #3)
  checkForUpdate: () => Promise<void>;
  downloadAndInstall: () => Promise<void>;
  restartToUpdate: () => Promise<void>;
  dismissUpdate: () => void;
}

// --- Constants ---

const LAST_CHECK_KEY = 'mantra-update-last-check';
const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000; // 24 hours
const STARTUP_DELAY_MS = 5000; // 5 seconds

// --- localStorage helpers ---

function safeGetItem(key: string): string | null {
  try {
    if (typeof window === 'undefined') return null;
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeSetItem(key: string, value: string): void {
  try {
    if (typeof window === 'undefined') return;
    window.localStorage.setItem(key, value);
  } catch {
    // ignore
  }
}

// --- Hook ---

export function useUpdateChecker(): UseUpdateCheckerResult {
  // State (AC #2)
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>('idle');
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // Refs for resource management
  const updateRef = useRef<Update | null>(null);
  const startupTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const periodicTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  /**
   * 释放当前 Update 资源
   */
  const cleanupUpdate = useCallback(async () => {
    if (updateRef.current) {
      try {
        await updateRef.current.close();
      } catch {
        // ignore close errors
      }
      updateRef.current = null;
    }
  }, []);

  /**
   * 静默下载更新 (AC #6)
   */
  const performDownload = useCallback(async (update: Update) => {
    if (!mountedRef.current) return;
    setUpdateStatus('downloading');
    setDownloadProgress(0);

    let downloaded = 0;
    let contentLength: number | undefined;

    try {
      await update.downloadAndInstall((event: DownloadEvent) => {
        if (!mountedRef.current) return;
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength;
            downloaded = 0;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            if (contentLength && contentLength > 0) {
              setDownloadProgress(
                Math.min(Math.round((downloaded / contentLength) * 100), 100)
              );
            }
            break;
          case 'Finished':
            setDownloadProgress(100);
            break;
        }
      });

      if (!mountedRef.current) return;
      setUpdateStatus('ready');
    } catch (err) {
      if (!mountedRef.current) return;
      const msg = err instanceof Error ? err.message : 'Download failed';
      console.warn('[useUpdateChecker] download error:', msg);
      setErrorMessage(msg);
      setUpdateStatus('error');
    }
  }, []);

  /**
   * 核心检查逻辑 (AC #4, #7)
   * @param silent - true: 自动检查（错误静默）; false: 手动检查（错误显示）
   */
  const performCheck = useCallback(async (silent: boolean) => {
    if (!mountedRef.current) return;

    // 清理旧的 Update 资源
    await cleanupUpdate();

    setUpdateStatus('checking');
    setErrorMessage(null);

    try {
      const update = await check();

      if (!mountedRef.current) return;

      if (update) {
        updateRef.current = update;
        setUpdateAvailable(true);
        setUpdateInfo({
          version: update.version,
          date: update.date,
          body: update.body,
        });
        setUpdateStatus('idle');

        // 记录检查时间
        safeSetItem(LAST_CHECK_KEY, Date.now().toString());

        // 自动触发静默下载 (AC #6)
        await performDownload(update);
      } else {
        setUpdateAvailable(false);
        setUpdateInfo(null);
        setUpdateStatus('idle');

        // 记录检查时间
        safeSetItem(LAST_CHECK_KEY, Date.now().toString());
      }
    } catch (err) {
      if (!mountedRef.current) return;
      const msg = err instanceof Error ? err.message : 'Check failed';
      console.warn('[useUpdateChecker]', msg);

      if (silent) {
        // AC #7: 自动检查失败 → 回到 idle，不设 errorMessage
        setUpdateStatus('idle');
      } else {
        // AC #7: 手动检查失败 → 设 error + errorMessage
        setErrorMessage(msg);
        setUpdateStatus('error');
      }
    }
  }, [cleanupUpdate, performDownload]);

  // --- 暴露的方法 (AC #3) ---

  /**
   * 手动触发检查 — 非静默模式
   */
  const checkForUpdate = useCallback(async () => {
    await performCheck(false);
  }, [performCheck]);

  /**
   * 下载并安装更新
   */
  const downloadAndInstall = useCallback(async () => {
    if (updateRef.current) {
      await performDownload(updateRef.current);
    }
  }, [performDownload]);

  /**
   * 重启应用以应用更新
   */
  const restartToUpdate = useCallback(async () => {
    try {
      await relaunch();
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Relaunch failed';
      console.warn('[useUpdateChecker] relaunch error:', msg);
      setErrorMessage(msg);
      setUpdateStatus('error');
    }
  }, []);

  /**
   * 忽略当前更新通知 (AC #3)
   */
  const dismissUpdate = useCallback(() => {
    setUpdateAvailable(false);
    setUpdateInfo(null);
    setDownloadProgress(0);
    setUpdateStatus('idle');
    setErrorMessage(null);

    // 异步清理 Update 资源
    cleanupUpdate();
  }, [cleanupUpdate]);

  // --- 自动检查逻辑 (AC #4, #5) ---

  useEffect(() => {
    mountedRef.current = true;

    const lastCheckStr = safeGetItem(LAST_CHECK_KEY);
    const lastCheck = lastCheckStr ? parseInt(lastCheckStr, 10) : 0;
    const elapsed = Date.now() - lastCheck;

    // AC #5: 距上次检查不足 24 小时，跳过启动自动检查
    if (elapsed >= CHECK_INTERVAL_MS || lastCheck === 0) {
      // AC #4: 启动延迟检查
      startupTimerRef.current = setTimeout(() => {
        performCheck(true);
      }, STARTUP_DELAY_MS);
    }

    // 24 小时补充 setTimeout (长时间运行)
    periodicTimerRef.current = setTimeout(() => {
      performCheck(true);
    }, CHECK_INTERVAL_MS);

    return () => {
      mountedRef.current = false;
      if (startupTimerRef.current) {
        clearTimeout(startupTimerRef.current);
        startupTimerRef.current = null;
      }
      if (periodicTimerRef.current) {
        clearTimeout(periodicTimerRef.current);
        periodicTimerRef.current = null;
      }
      // 清理 Update 资源
      cleanupUpdate();
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    updateAvailable,
    updateInfo,
    downloadProgress,
    updateStatus,
    errorMessage,
    checkForUpdate,
    downloadAndInstall,
    restartToUpdate,
    dismissUpdate,
  };
}
