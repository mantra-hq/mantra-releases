# Mantra - Changelog

**English** | [中文](./CHANGELOG.zh-CN.md)

**Mantra** is a local-first "time machine" for AI-assisted coding sessions. It aligns AI conversation logs (from tools like Claude Code, Cursor, Gemini CLI, etc.) with Git history on a unified timeline, allowing developers to replay context and understand the "why" behind code changes.

---

## [v0.8.0] - 2026-02-09

### Added

- **Auto-Updater System (Epic 14)**:
    - **Tauri Plugin Updater Integration**: Configured `tauri-plugin-updater` with signing key verification and update endpoint (Story 14.1, 14.2).
    - **Cloudflare Worker Update Endpoint**: Deployed a Cloudflare Worker to serve update metadata from the public release repository (Story 14.3).
    - **CI/CD Updater Artifacts**: Extended `release.yml` and `publish-public.yml` to build, sign, and publish updater artifacts (`.sig`, `.app.tar.gz`) across all platforms (Story 14.4).
    - **useUpdateChecker Hook**: Implemented a React Hook for automatic update checking (every 24h), silent background downloading, and restart-to-update flow (Story 14.5).
    - **Settings "About & Update" Section**: Added version display and manual update check to the General Settings page, with download progress and restart button (Story 14.7).
    - **Lightweight Update UX (VS Code-style)**: Badge indicator on the settings button when an update is ready; auto-check toggle with localStorage persistence; changelog opens as external link (Story 14.10).
    - **i18n**: Added `updater` namespace translations for English and Chinese (Story 14.8).
    - **E2E Verification**: Completed full end-to-end update flow validation across platforms (Story 14.9).

- **Settings Page Overhaul**:
    - **Nested Routing**: Refactored Settings into sidebar navigation with General, Development, and Privacy sub-pages.
    - **ToolConfigPathManager**: Added UI for managing custom AI tool configuration paths (Claude Code, Cursor, etc.).

- **Session Player**:
    - **SkillRenderer**: New component for displaying skill/command invocation details (name, arguments, results) in session narratives.
    - **PrivacyPledge**: Added privacy commitment display to the Player empty state.

- **MCP Enhancements**:
    - **Custom Config Path Resolution**: Support user-defined configuration directories for MCP tool scanning and takeover.

### Changed

- **Settings Layout**: Centered content layout inspired by VS Code settings for improved readability.
- **CI/CD**: Improved version string handling in release workflows (awk-based changelog extraction).

## [v0.7.0] - 2026-02-06

### Added

- **MCP Gateway (Epic 11)**:
    - **SSE Server Core**: Implemented an embedded Axum SSE Server with JSON-RPC protocol support (Story 11.1).
    - **MCP Service Data Model**: Built CRUD operations and state management for MCP services on SQLite (Story 11.2).
    - **Config Import & Smart Takeover**: One-click import of MCP configurations from Claude Code, Cursor, Gemini CLI, and Codex, with conflict diff comparison and shadow mode preview (Story 11.3, 11.13).
    - **Environment Variable Management**: Centralized management of sensitive information (API keys, etc.) with encrypted storage and cross-service reference detection (Story 11.4).
    - **Context Routing**: Implemented Longest Prefix Match (LPM) routing algorithm to automatically select MCP services based on project context (Story 11.5).
    - **Mantra Hub UI**: New Hub page integrating Gateway status card, MCP service list, service forms, environment variable manager, and project association features (Story 11.6).
    - **System Tray Integration**: System tray with quick actions including project switching and Gateway status management (Story 11.7).
    - **Architecture Refactor (ADR-001)**: Modular adapter pattern supporting Claude, Gemini, Cursor, and Codex configuration adapters (Story 11.8).
    - **Project Detail MCP Integration**: Embedded MCP management entry in project detail page with project-level service association and status viewing (Story 11.9).
    - **Tool Granular Management**: Implemented ToolPolicy model with global and service-level tool permission control and Gateway interception (Story 11.10).
    - **Built-in MCP Inspector**: Integrated debugger with ToolExplorer, ToolTester, and RpcLogViewer components (Story 11.11).
    - **Remote MCP OAuth Support**: OAuth authentication flow for remote MCP services like Google Drive and Slack (Story 11.12).
    - **Streamable HTTP Compliance**: Compliant with MCP Streamable HTTP spec (2025-03-26), unified `/mcp` endpoint with Origin validation and Session Header (Story 11.14).
    - **Smart Takeover Merge Engine**: Three-tier classification (add/update/conflict) with source tracking and conflict resolution (Story 11.19).
    - **Full Tool Auto-Takeover**: Full tool grouped preview, tool selection, and transactional takeover (Story 11.20).
    - **Claude Local Scope Support**: Full support for Claude Code local scope configuration (Story 11.21).
    - **Atomic Backup & Restore**: Backup integrity verification and atomic operations, retaining the last 5 versions with auto-cleanup (Story 11.22, 11.23).
    - **MCP Roots Mechanism**: Implemented MCP Roots protocol with LPM integration for project-level configuration awareness (Story 11.26, 11.27).
    - **Strict Mode Service Filtering**: Project context-based service filtering to expose only relevant services (Story 11.28).
    - **Post-Import Auto-Linking**: Automatic project association guidance after config import, with name matching and parallel linking (Story 11.29).

- **UX Interaction Consistency (Epic 12)**:
    - **Dialog → Sheet Migration**: Migrated non-confirmation Dialogs to Sheet (side drawer) pattern, including ProjectInfo, McpConfigImport, EditMessage, EnvVariable, OAuthConfig, and more (Story 12.1 ~ 12.4).
    - **ActionSheet Wrapper**: Introduced unified ActionSheet wrapper component to standardize overlay interaction patterns (Story 12.4).
    - **Tool Policy UX Optimization**: Improved tool permission management entry points for better clarity and consistency (Story 12.5).

- **Database**: Introduced lightweight database connection method for optimized query performance (Story 11.2).

### Changed

- **MCP Architecture**: Refactored MCP command structure and Gateway startup logic, unified configuration writing flow.
- **UI Components**: Standardized Dialog sizing across the application for improved responsiveness and usability.
- **i18n**: Updated English and Chinese translations covering backup management, OAuth, tool policy, and other new features.

### Fixed

- **MCP**: Fixed service configuration handling during toggle, tool selection initialization logic, and project-level config file cleanup (Story 11.25).
- **Hub**: Fixed ToolPolicyEditor layout consistency issues and TakeoverStatusCard test implementations.
- **Gateway**: Fixed SSE stream handling for roots/list requests and Gateway restart behavior.

## [v0.6.0] - 2026-01-25

### Added

- **Project Management (Epic 1)**:
    - **View-based Aggregation**: Implemented a dual-layer architecture separating the storage layer (original import structure) from the view layer (logical projects aggregated by physical path). This allows sessions from different AI tools (Claude, Gemini, Cursor) to be unified under a single workspace (Story 1.12).
    - **Logical Project Renaming**: Added support for custom display names for logical projects, stored independently of the imported project names (Story 1.13).
    - **Improved Project Identification**: Enhanced detection of path existence and type (Local, Virtual, Remote) to better manage "needs association" states for projects.
    - **Unlinking Support**: Added the ability to unlink specific source projects from an aggregated logical project.

- **IDE Support**: Expanded support for additional IDE configurations and environments.

- **Documentation**: Integrated Gemini CLI context documentation for enhanced agent assistance.

### Fixed

- **UI/UX**:
    - Resolved synchronization issues where the TopBar session list and project name did not update immediately after association.
    - Fixed file tree display issues when associated paths are subdirectories of a Git repository.
    - Addressed missing i18n translations for project renaming and unlinking actions.

- **Packaging**:
    - Fixed RPM build issues.
    - Refined release documentation in the publishing workflow.

## [v0.5.2] - 2026-01-22

### Fixed

- **UI/UX**: Addressed UI overflow issues and improved project path handling.

### Maintenance

- **Documentation**: Updated Story 8.19 status to done after review.

## [v0.5.1] - 2026-01-22

### Changed

- **Release**: Update client version to v0.5.1.

## [v0.5.0] - 2026-01-20

### Added

- **Mode Switching (Epic 10)**:
    - **Compact Mode**: Introduced a new "Refine" (Compact) mode for session context optimization, allowing users to focus on essential information.
    - **Mode Switch UI**: Integrated a seamless mode switcher into the TopBar (Story 10.11, 10.1).
    - **Keyboard Shortcuts**: Added support for global keyboard shortcuts to toggle modes and perform actions (Story 10.10).
    - **Undo/Redo Stack**: Implemented robust undo/redo functionality for session editing operations (Story 10.8).
    - **Edit State Persistence**: Ensured edit states are preserved across mode switches (Story 10.9).

- **Session Management Enhancements**:
    - **Token Statistics**: Added real-time token usage statistics and calculation (Story 10.6).
    - **Message Insertion**: Implemented the ability to insert new messages into the session narrative with improved UI/UX (Story 10.5).
    - **Message Annotation**: Added support for message annotations (Story 10.4).
    - **Compress Preview**: Introduced a preview component for real-time message optimization (Story 10.3).
    - **Export Functionality**: Enabled exporting sessions, including support for processed/refined sessions (Story 10.7).

### Fixed

- **Bug Fixes**: Various bug fixes and improvements identified during code reviews for stories 10.4, 10.5, 10.7, 10.8, 10.9, 10.10.

## [v0.4.0] - 2026-01-10

### Added

- **Privacy & Security (Epic 3)**:
    - **Privacy Protection Module**: Implemented a comprehensive module for handling sensitive information.
    - **Custom Detection Rules**: Added management interface for custom privacy detection rules (Story 3.10).
    - **Pre-Tool Use Detection**: Implemented `PreToolUse` file content detection to prevent sensitive data leakage before tool execution (Story 3.11).
    - **Interception Records**: Added storage and a dedicated page for viewing privacy interception records (Story 3.7, 3.8).
    - **Pre-upload Privacy Check**: Integrated privacy checks before session uploads (Story 3.9).

- **Project Analytics (Epic 2)**:
    - **Project Analytics**: Finalized the implementation of project analytics features (Story 2.34).
    - **Global Search Enhancements**: Enhanced global search with filters and grouped results (Story 2.33).
    - **Submodule Support**: Enhanced Git submodule support in the FileTree component (Story 2.31).

- **Testing & Quality (Epic 9)**:
    - **Visual Regression Testing**: Implemented a visual regression testing framework (Story 9.5).
    - **Core E2E Tests**: Added core End-to-End tests for key user flows (Story 9.4).
    - **IPC Mock**: Implemented IPC Mock layer for reliable E2E testing (Story 9.2).

- **Parser Enhancements (Epic 8)**:
    - **Cursor Parser**: Completed enhancements and tool mapping for Cursor logs (Story 8.17).
    - **Image Support**: Added support for image content blocks in sessions (Story 8.16).
    - **Resilience**: Enhanced parser resilience and added format compatibility monitoring (Story 8.15).
    - **Frontend Adaptation**: Completed frontend component adaptation, including FileEdit Diff view integration (Story 8.11).
    - **Standardization**: Refactored parsers to a unified directory structure (Story 8.14) and enhanced data models (Story 8.1).

### Changed

- **Refactoring**: Refactored privacy hook architecture.
- **Documentation**: Updated documentation for Claude Code Hook plugin and privacy features.


## [v0.3.0] - 2025-12-30

### Added

- **Privacy Sanitizer (Epic 3)**:
  - **Rust Regex Engine**: Implemented high-performance regex-based sanitization engine (Story 3.1).
  - **Custom Rules**: Added support for user-defined sanitization rules (Story 3.3).
  - **Diff Preview**: Implemented side-by-side diff preview for sanitized content (Story 3.2, 3.4).
  - **Rule Matrix**: Added comprehensive test matrix for sanitization rules.

- **Project Management (Epic 2)**:
  - **Multi-Source Aggregation**: Unified view for projects from different sources (Claude, Cursor, Gemini) (Story 2.25).
  - **Project Drawer**: Implemented collapsible project drawer for better navigation (Story 2.18).
  - **Project Info**: Added dialog to view detailed project metadata (Story 2.27).
  - **Empty State Handling**: Improved handling and filtering of empty projects/sessions (Story 2.29).
  - **One-Click Copy**: Added one-click copy functionality for messages and logs (Story 2.22, 2.28).

- **Import Experience**:
  - **Enhanced Wizard**: Added real-time progress feedback and cancellation support (Story 2.20, 2.23).
  - **Smart Identification**: Improved project identification using Git remote URLs (Story 1.9).
  - **Gemini Support**: Added full support for importing and parsing Gemini CLI logs (Story 1.6).

- **Internationalization (i18n)**:
  - **Full Support**: Completed i18n implementation for all core components (Story 2.26).
  - **Bilingual**: Added support for English and Simplified Chinese.

### Changed

- **UX Refinements**:
  - **TopBar**: Refactored TopBar for better navigation and mode switching (Story 2.17).
  - **Feedback**: Unified user feedback mechanism (Toast notifications).
  - **Visuals**: Updated source icons and project tree animations.
  - **Dashboard**: Removed Dashboard in favor of a direct Player view when appropriate (Story 2.21).

## [v0.2.0] - 2025-12-15

### Added

- **Session Navigation (Epic 2)**:
  - **Message Filtering**: Added ability to filter session messages by type (Story 2.16).
  - **Detail Panel**: Implemented collapsible detail panel for message inspection (Story 2.15).
  - **History UX**: Improved user experience for navigating history states (Story 2.14).
  - **Breadcrumbs**: Added breadcrumb navigation (Story 2.17).

- **Code Exploration**:
  - **File Browser**: Implemented multi-tab file browser (Story 2.13).
  - **Smart Selection**: Added smart logic for initial file selection (Story 2.12).
  - **Git Association**: Linked initial code state with Git repository (Story 2.11).
  - **Global Search**: Implemented global search across sessions and content (Story 2.10).

- **Parsers (Epic 1)**:
  - **Cursor Support**: Added initial support for parsing Cursor session logs (Story 1.7).

- **Core Components**:
  - **TimberLine**: Implemented the timeline controller for session playback (Story 2.6).
  - **CodeSnapshot**: Added view for displaying code snapshots at specific points in time (Story 2.5).

### Changed

- **Fonts**: integrated JetBrains Mono for better code readability.
- **Layout**: Optimized split-pane layout and adaptive heights.

## [v0.1.0] - 2025-12-01

### Added

- **Initial Release**:
  - **Tauri v2 Setup**: Project initialization with Tauri v2 and React 19 (Story 1.1).
  - **Design System**: Implemented base design system and theme configuration (Story 2.1).
  - **Dual Stream Layout**: Core layout engine for side-by-side narrative and code views (Story 2.2).
  - **Narrative Stream**: Component for rendering the chat history (Story 2.3).
  - **Message Bubbles**: Base components for different message types (Story 2.4).
  - **Time Travel**: Basic Git time travel service implementation (Story 1.4, 2.7).
  - **Project Scanner**: Service to scan and aggregate local projects (Story 1.5).
  - **Dashboard**: Initial dashboard for listing projects (Story 2.8).
  - **Import UI**: Basic UI for importing session logs (Story 2.9).
  - **Claude Parser**: Initial parser for Claude Code JSONL logs (Story 1.3).
