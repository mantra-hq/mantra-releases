/**
 * Detail 组件导出
 */

export { ToolDetailPanel } from "./ToolDetailPanel";
export type { ToolDetailPanelProps } from "./ToolDetailPanel";

// Story 2.15: 渲染器导出
export { ToolOutputRenderer, getToolRenderer } from "./renderers";
export type { RendererProps } from "./renderers";
export { TerminalRenderer } from "./renderers/TerminalRenderer";
export type { TerminalRendererProps } from "./renderers/TerminalRenderer";
export { FileRenderer } from "./renderers/FileRenderer";
export type { FileRendererProps } from "./renderers/FileRenderer";
export { SearchResultRenderer } from "./renderers/SearchResultRenderer";
export type { SearchResultRendererProps, SearchMatch } from "./renderers/SearchResultRenderer";
export { GenericRenderer } from "./renderers/GenericRenderer";
export type { GenericRendererProps } from "./renderers/GenericRenderer";
