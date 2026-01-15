/**
 * ImageBlock - 图片内容块组件
 * Story 8.16: Task 6
 *
 * 渲染 ContentBlock::Image 类型的图片内容
 * 支持 base64 编码和 URL 两种图片来源
 *
 * AC: #5 (前端 ImageBlock 组件)
 */

import * as React from "react";
import { createPortal } from "react-dom";
import { cn } from "@/lib/utils";
import { Image as ImageIcon, ExternalLink, Maximize2 } from "lucide-react";

export interface ImageBlockProps {
  /** 图片数据 (base64 编码或 URL) */
  data: string;
  /** 媒体类型 (e.g., "image/png", "image/jpeg") */
  mediaType: string;
  /** 来源类型 ("base64" | "url") */
  sourceType?: string;
  /** 替代文本 (用于可访问性) */
  altText?: string;
  /** 自定义 className */
  className?: string;
}

/**
 * ImageBlock 组件
 *
 * 渲染策略:
 * - base64: 直接使用 data URI 显示图片
 * - url: 使用外部 URL 加载图片
 *
 * 视觉规范:
 * - 最大宽度 100%，最大高度 400px
 * - 圆角边框，hover 时显示放大提示
 * - 支持点击放大查看
 */
export function ImageBlock({
  data,
  mediaType,
  sourceType = "base64",
  altText,
  className,
}: ImageBlockProps) {
  const [isLoading, setIsLoading] = React.useState(true);
  const [hasError, setHasError] = React.useState(false);
  const [isExpanded, setIsExpanded] = React.useState(false);
  const modalRef = React.useRef<HTMLDivElement>(null);

  // 放大视图打开时自动聚焦，确保 ESC 键能正常工作
  React.useEffect(() => {
    if (isExpanded && modalRef.current) {
      modalRef.current.focus();
    }
  }, [isExpanded]);

  // 构建图片 src
  const imageSrc = React.useMemo(() => {
    if (sourceType === "url") {
      return data;
    }
    // base64: 构建 data URI
    return `data:${mediaType};base64,${data}`;
  }, [data, mediaType, sourceType]);

  // 图片加载完成
  const handleLoad = React.useCallback(() => {
    setIsLoading(false);
    setHasError(false);
  }, []);

  // 图片加载错误
  const handleError = React.useCallback(() => {
    setIsLoading(false);
    setHasError(true);
  }, []);

  // 点击放大
  const handleExpand = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(true);
  }, []);

  // 关闭放大视图
  const handleClose = React.useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(false);
  }, []);

  // 键盘关闭
  const handleKeyDown = React.useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setIsExpanded(false);
    }
  }, []);

  // 加载失败时的回退 UI
  if (hasError) {
    return (
      <div
        className={cn(
          "flex items-center gap-2 p-3 rounded-lg",
          "bg-destructive/10 text-destructive border border-destructive/20",
          className
        )}
      >
        <ImageIcon className="h-4 w-4 shrink-0" />
        <span className="text-sm">图片加载失败</span>
        {sourceType === "url" && (
          <a
            href={data}
            target="_blank"
            rel="noopener noreferrer"
            className="ml-auto flex items-center gap-1 text-xs hover:underline"
            onClick={(e) => e.stopPropagation()}
          >
            查看原图
            <ExternalLink className="h-3 w-3" />
          </a>
        )}
      </div>
    );
  }

  return (
    <>
      {/* 图片容器 */}
      <div
        className={cn(
          "relative group rounded-lg overflow-hidden",
          "border border-border bg-muted/30",
          "max-w-full",
          className
        )}
        onClick={handleExpand}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => e.key === "Enter" && handleExpand(e as unknown as React.MouseEvent)}
        aria-label={altText || "查看图片"}
      >
        {/* 加载占位 */}
        {isLoading && (
          <div className="flex items-center justify-center h-32 text-muted-foreground">
            <ImageIcon className="h-8 w-8 animate-pulse" />
          </div>
        )}

        {/* 图片 */}
        <img
          src={imageSrc}
          alt={altText || "用户上传的图片"}
          className={cn(
            "max-w-full max-h-[400px] object-contain",
            "cursor-zoom-in",
            isLoading && "hidden"
          )}
          onLoad={handleLoad}
          onError={handleError}
        />

        {/* Hover 放大提示 */}
        {!isLoading && (
          <div
            className={cn(
              "absolute inset-0 flex items-center justify-center",
              "bg-black/40 opacity-0 group-hover:opacity-100",
              "transition-opacity duration-150"
            )}
          >
            <div className="flex items-center gap-1.5 text-white text-sm">
              <Maximize2 className="h-4 w-4" />
              点击放大
            </div>
          </div>
        )}
      </div>

      {/* 放大视图 (Portal to body for true fullscreen) */}
      {isExpanded &&
        createPortal(
          <div
            ref={modalRef}
            className={cn(
              "fixed inset-0 z-[9999] flex items-center justify-center",
              "bg-black/80 backdrop-blur-sm",
              "cursor-zoom-out"
            )}
            onClick={handleClose}
            onKeyDown={handleKeyDown}
            role="dialog"
            aria-modal="true"
            aria-label="图片预览"
            tabIndex={0}
          >
            <img
              src={imageSrc}
              alt={altText || "用户上传的图片"}
              className="max-w-[90vw] max-h-[90vh] object-contain rounded-lg shadow-2xl"
            />

            {/* 关闭提示 */}
            <div className="absolute bottom-4 left-1/2 -translate-x-1/2 text-white/60 text-sm">
              按 ESC 或点击任意位置关闭
            </div>
          </div>,
          document.body
        )}
    </>
  );
}

export default ImageBlock;
