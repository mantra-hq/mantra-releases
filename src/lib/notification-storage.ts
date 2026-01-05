/**
 * Notification Storage - 通知持久化工具
 * Tech-Spec: 通知系统 ADR-003
 *
 * 使用 localStorage 存储已读 ID 和已关闭的 Banner
 * - 键名使用版本前缀便于未来迁移
 * - 已读 ID 最多保留 200 条，FIFO 淘汰
 */

/** localStorage 键名 */
const STORAGE_KEYS = {
  READ_IDS: "mantra_notification_v1_read_ids",
  DISMISSED_BANNERS: "mantra_notification_v1_dismissed_banners",
} as const;

/** 已读 ID 最大数量 */
const MAX_READ_IDS = 200;

/**
 * 安全地从 localStorage 读取 JSON 数组
 */
function getStringArray(key: string): string[] {
  try {
    const value = localStorage.getItem(key);
    if (!value) return [];
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * 安全地写入 JSON 数组到 localStorage
 */
function setStringArray(key: string, value: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // 忽略存储失败（如存储空间不足）
  }
}

/**
 * 通知持久化工具对象
 */
export const notificationStorage = {
  /**
   * 获取所有已读通知 ID
   */
  getReadIds: (): string[] => {
    return getStringArray(STORAGE_KEYS.READ_IDS);
  },

  /**
   * 添加已读通知 ID
   * 自动执行 FIFO 淘汰，保持最多 200 条
   */
  addReadId: (id: string): void => {
    const ids = getStringArray(STORAGE_KEYS.READ_IDS);
    // 如果已存在，移到最前面
    const filtered = ids.filter((existingId) => existingId !== id);
    const updated = [id, ...filtered];
    // FIFO 淘汰：保留最新的 200 条
    const trimmed = updated.slice(0, MAX_READ_IDS);
    setStringArray(STORAGE_KEYS.READ_IDS, trimmed);
  },

  /**
   * 批量添加已读通知 ID
   */
  addReadIds: (newIds: string[]): void => {
    const ids = getStringArray(STORAGE_KEYS.READ_IDS);
    // 移除重复的，保持新 ID 在前
    const filtered = ids.filter((id) => !newIds.includes(id));
    const updated = [...newIds, ...filtered];
    const trimmed = updated.slice(0, MAX_READ_IDS);
    setStringArray(STORAGE_KEYS.READ_IDS, trimmed);
  },

  /**
   * 获取所有已关闭的 Banner ID
   */
  getDismissedBanners: (): string[] => {
    return getStringArray(STORAGE_KEYS.DISMISSED_BANNERS);
  },

  /**
   * 永久关闭 Banner
   */
  dismissBanner: (id: string): void => {
    const ids = getStringArray(STORAGE_KEYS.DISMISSED_BANNERS);
    if (!ids.includes(id)) {
      setStringArray(STORAGE_KEYS.DISMISSED_BANNERS, [...ids, id]);
    }
  },

  /**
   * 清除所有存储数据
   */
  clear: (): void => {
    localStorage.removeItem(STORAGE_KEYS.READ_IDS);
    localStorage.removeItem(STORAGE_KEYS.DISMISSED_BANNERS);
  },
};
