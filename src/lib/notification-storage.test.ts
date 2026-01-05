/**
 * notification-storage Tests - 通知持久化工具测试
 * Tech-Spec: 通知系统 Task 18
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import { notificationStorage } from "./notification-storage";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, "localStorage", { value: localStorageMock });

describe("notificationStorage", () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  describe("getReadIds", () => {
    it("should return empty array when no data", () => {
      const ids = notificationStorage.getReadIds();
      expect(ids).toEqual([]);
    });

    it("should return stored read IDs", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify(["id-1", "id-2"])
      );

      const ids = notificationStorage.getReadIds();
      expect(ids).toEqual(["id-1", "id-2"]);
    });

    it("should handle invalid JSON gracefully", () => {
      localStorageMock.setItem("mantra_notification_v1_read_ids", "invalid json");

      const ids = notificationStorage.getReadIds();
      expect(ids).toEqual([]);
    });

    it("should handle non-array values gracefully", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify({ notAnArray: true })
      );

      const ids = notificationStorage.getReadIds();
      expect(ids).toEqual([]);
    });
  });

  describe("addReadId", () => {
    it("should add new read ID", () => {
      notificationStorage.addReadId("id-1");

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_read_ids") || "[]"
      );
      expect(stored).toContain("id-1");
    });

    it("should move existing ID to front", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify(["id-1", "id-2", "id-3"])
      );

      notificationStorage.addReadId("id-2");

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_read_ids") || "[]"
      );
      expect(stored[0]).toBe("id-2");
      expect(stored).toHaveLength(3);
    });

    it("should enforce FIFO limit of 200", () => {
      // Pre-populate with 200 IDs
      const existingIds = Array.from({ length: 200 }, (_, i) => `old-id-${i}`);
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify(existingIds)
      );

      notificationStorage.addReadId("new-id");

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_read_ids") || "[]"
      );
      expect(stored).toHaveLength(200);
      expect(stored[0]).toBe("new-id");
      expect(stored).not.toContain("old-id-199"); // Oldest should be evicted
    });
  });

  describe("addReadIds", () => {
    it("should add multiple read IDs", () => {
      notificationStorage.addReadIds(["id-1", "id-2", "id-3"]);

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_read_ids") || "[]"
      );
      expect(stored).toEqual(["id-1", "id-2", "id-3"]);
    });

    it("should deduplicate when adding", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify(["existing-1", "existing-2"])
      );

      notificationStorage.addReadIds(["new-1", "existing-1"]);

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_read_ids") || "[]"
      );
      expect(stored).toEqual(["new-1", "existing-1", "existing-2"]);
    });
  });

  describe("getDismissedBanners", () => {
    it("should return empty array when no data", () => {
      const ids = notificationStorage.getDismissedBanners();
      expect(ids).toEqual([]);
    });

    it("should return stored dismissed banner IDs", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_dismissed_banners",
        JSON.stringify(["banner-1", "banner-2"])
      );

      const ids = notificationStorage.getDismissedBanners();
      expect(ids).toEqual(["banner-1", "banner-2"]);
    });
  });

  describe("dismissBanner", () => {
    it("should add banner ID to dismissed list", () => {
      notificationStorage.dismissBanner("banner-1");

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_dismissed_banners") || "[]"
      );
      expect(stored).toContain("banner-1");
    });

    it("should not duplicate banner IDs", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_dismissed_banners",
        JSON.stringify(["banner-1"])
      );

      notificationStorage.dismissBanner("banner-1");

      const stored = JSON.parse(
        localStorageMock.getItem("mantra_notification_v1_dismissed_banners") || "[]"
      );
      expect(stored).toHaveLength(1);
    });
  });

  describe("clear", () => {
    it("should remove all notification storage keys", () => {
      localStorageMock.setItem(
        "mantra_notification_v1_read_ids",
        JSON.stringify(["id-1"])
      );
      localStorageMock.setItem(
        "mantra_notification_v1_dismissed_banners",
        JSON.stringify(["banner-1"])
      );

      notificationStorage.clear();

      expect(localStorageMock.removeItem).toHaveBeenCalledWith(
        "mantra_notification_v1_read_ids"
      );
      expect(localStorageMock.removeItem).toHaveBeenCalledWith(
        "mantra_notification_v1_dismissed_banners"
      );
    });
  });
});
