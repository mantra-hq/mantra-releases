/**
 * SyncStatus - 单元测试
 * Story 2.14: Task 7.4
 */

import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { SyncStatus } from "./SyncStatus";

describe("SyncStatus", () => {
    it("AC #11: 默认显示已同步状态", () => {
        render(<SyncStatus />);

        const status = screen.getByTestId("sync-status");
        expect(status).toHaveTextContent("已同步");
        expect(status).toHaveAttribute("data-status", "synced");
        expect(status).toHaveClass("text-green-500");
    });

    it("应该显示同步中状态", () => {
        render(<SyncStatus status="syncing" />);

        const status = screen.getByTestId("sync-status");
        expect(status).toHaveTextContent("同步中");
        expect(status).toHaveClass("text-blue-500");
    });

    it("应该显示有远程更新状态", () => {
        render(<SyncStatus status="behind" />);

        const status = screen.getByTestId("sync-status");
        expect(status).toHaveTextContent("有远程更新");
        expect(status).toHaveClass("text-amber-500");
    });

    it("应该显示离线状态", () => {
        render(<SyncStatus status="offline" />);

        const status = screen.getByTestId("sync-status");
        expect(status).toHaveTextContent("离线");
        expect(status).toHaveClass("text-muted-foreground");
    });

    it("应该应用自定义类名", () => {
        render(<SyncStatus className="custom-class" />);

        expect(screen.getByTestId("sync-status")).toHaveClass("custom-class");
    });
});
