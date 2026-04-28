# Mantra - Changelog

**English** | [中文](./CHANGELOG.zh-CN.md)

**Mantra** is a local-first "time machine" for AI-assisted coding sessions. It aligns AI conversation logs (from tools like Claude Code, Cursor, Gemini CLI, etc.) with Git history on a unified timeline, allowing developers to replay context and understand the "why" behind code changes.

---

## [v0.11.7] - 2026-04-28

### Added

- **Skills Tag System**: Users can now attach tags to skills and manage them in bulk via a dedicated **Tag Manager** panel (rename, delete, merge tags; batch-assign tags to multiple skills). Tags declared in a skill's frontmatter are automatically imported as local tag associations on takeover. SkillCards display tag chips (up to 3 + overflow count), and the SkillsHub list can be filtered by one or more tags simultaneously.
- **Local Skill Ratings & Full-Text Search**: Skills can now be given a private 1–5 star rating that stays local and never leaks into the shared file. A full-text search engine indexes skill names, descriptions, body content, tags, and associated projects, returning highlighted result snippets. The filter sidebar lets users narrow results by tool adapter, tag, rating range, conflict state, and associated project — with active filters shown as dismissible chips.
- **Skill Version Snapshots & Timeline Rollback**: After a skill is under Mantra management, content changes are automatically captured as passive snapshots. Users can browse the snapshot timeline in the Skill Detail view and roll back to any prior version with a single click.
- **GitHub Repository Bulk Import**: A new **GitHub Import** tab in the import sheet lets users import all skills from a GitHub repository at once via HTTPS shallow clone. Imported skills are tracked with their remote origin so that future upstream changes can be detected.
- **Upstream Change Detection & Three-Way Diff**: For GitHub-imported skills, users can check for upstream updates. When the remote version diverges from the local copy, a **Three-Way Compare** dialog presents a line-level diff across the original imported version, the current local state, and the latest remote version. Users can resolve conflicts skill-by-skill or apply a batch strategy (Overwrite / Rename with Suffix / Skip).
- **Skill Name Disambiguation on Import**: When an imported skill shares a display name with an existing skill, Mantra now automatically appends a suffix to keep the new entry distinct. The original name is preserved and no data is lost.
- **Skill Source Origin Detection**: During scanning, Mantra reads the Git remote configuration of each skill directory and records its GitHub origin (host / owner / repository). When multiple skills share the same display name, the Skill Card subtitle now shows the origin path to help users distinguish skills cloned from different accounts or forks.

---

## [v0.11.6] - 2026-04-25

### Added

- **Skills Custom Source Directories (Story 15.22)**: Added a dedicated **Skills Source Path Manager** in Settings → Development that lets users register custom directories as additional skill sources. Users can list, add, update, and remove source paths, with backend validation (`notAccessible`, `notFound` mapped to localized errors). Custom sources participate in the import scan with progress feedback alongside the four built-in adapters.
- **Skills Canonical Aggregation**: The skill import flow now aggregates detected skills by their **canonical filesystem source**, not just by display name. New `SkillImportUnit` units classify each skill into one of four buckets: `AutoImport` (clean cases), `AutoSkip` (already managed), `NeedsDecision` (ambiguous duplicates), or `Broken` (dangling symlinks / unreadable paths). Symlinks pointing to the same target across multiple tools are recognized as a single unit. The `DetectedSkill` model gains `canonical_source` and `broken` fields surfaced throughout the UI.
- **Canonical Multi-Source Takeover**: Skill backups now distinguish between `Single` and `Canonical` takeovers via a new `TakeoverKind` enum, and track each discovery location through `OccurrenceRecord` entries. Restoring a canonical takeover now correctly rebuilds **all** original adapter-side junctions/symlinks (verified with a 4-adapter Windows roundtrip test).
- **Skill Cleanup Confirmation & External Warnings**: New `SkillCleanupConfirmDialog` requires explicit confirmation before cleaning up external sources, and `SkillExternalWarning` highlights skills sourced from system paths or directories outside the managed set. The new `SkillOccurrencesList` shows where each skill was discovered.
- **MCP Hot-Reload Aggregator**: After adding or importing an MCP service, the gateway aggregator now refreshes and emits `notifications/tools|resources|prompts/list_changed` to active `/mcp` sessions. Connected clients see new tools immediately — **no gateway restart required**. Broadcast uses non-blocking `try_send` with dead-channel pruning so a slow subscriber cannot stall command handling.
- **Dynamic App Path Resolution**: New `useAppPaths` React hook surfaces real platform-resolved directory paths (backups, data, config) into UI text. `TakeoverStatusCard`, `SkillCleanupConfirmDialog`, and `SkillExternalWarning` now display the actual path instead of hardcoded values, with i18n placeholders for localized formatting.

### Improved

- **MCP Project-Scope Pollution Prevention (Story 11.30)**: Two-layer protection ensures the `mantra-gateway` server entry is **only** written to user-scope configs. Source-side guard (`is_user_scope_config_path` allowlist) rejects writes to project-level configs with `InvalidInput` (no spurious backups). A `ProjectScopeGuard` scans known projects on startup to clean legacy residue, and a `notify::RecommendedWatcher` (300 ms debounced) re-scans when `.cursor` / `.codex` / `.gemini` subdirectories appear at runtime. `sync_active_takeovers` silently skips legacy project-level backups instead of surfacing false-positive failures in the UI.
- **MCP Service Add/Import Goes Async**: `create_mcp_service` and `execute_mcp_import` are now async, fold in the unified `refresh_aggregator_and_notify` helper, and the import path consolidates DB reads under a single lock to eliminate a TOCTOU window. Env resolver failures degrade to an empty snapshot rather than dropping the freshly written DB row.
- **Production Build Hardening**: Test-only hooks (`set_test_home_override`, `PathResolver::with_paths`, `atomic_fs::rename_with_cross_fs_fallback`) are now gated behind a `test-hooks` Cargo feature; production builds no longer expose these surfaces. All four `SkillToolAdapter::user_skills_dir()` implementations and `safety::resolve_home` migrated from `dirs::home_dir` to `home::home_dir` so Windows tests honor `USERPROFILE` overrides without polluting the real user profile.

### Fixed

- **Skill Scanning — Absolute Canonical Source**: Fixed an issue where `canonical_source` could be returned as a relative path during skill detection. The scanner now guarantees an absolute path, preventing downstream path-joining bugs.
- **Skill Backup Validation**: Tightened null-handling in `CreateSkillBackupRequest` so unset optional fields no longer round-trip as the string `"null"`.
- **Linux AppImage Release Workflow**: Multiple iterations on the AppImage build pipeline — refined icon embedding, improved file handling and error paths, and streamlined the overall workflow. Release builds for Linux are now more reliable.
- **Release Deployment**: Added interactive GitHub account selection during release deployment to prevent pushing to the wrong remote when multiple accounts are configured.

## [v0.11.5] - 2026-04-01

### Fixed

- **Claude Code Parser — Tool Result Array Content**: Fixed a critical issue where `tool_result` blocks with array-form content (e.g., mixed text + image results from BashTool or FileReadTool) were serialized as raw JSON strings instead of being properly extracted. Text elements are now concatenated and image elements are converted to standalone `Image` content blocks.
- **Claude Code Parser — Redacted Thinking**: Added support for `redacted_thinking` content blocks from the Anthropic API. Previously these were degraded as unknown format entries; they are now recognized as a known type and mapped to an informational placeholder.
- **Claude Code Parser — Session Title Extraction**: Added extraction of `custom-title` and `ai-title` entry types from JSONL transcripts, with priority `ai-title` > `custom-title` > `summary`. This ensures more accurate session titles when Claude Code generates them.

### Improved

- **Claude Code Parser — Model Name**: The parser now extracts the actual model name (e.g., `claude-sonnet-4-5-20250514`) from assistant messages instead of using the Claude Code CLI version number as a fallback. The CLI version is retained as a fallback when no assistant messages are present.

## [v0.11.4] - 2026-03-13

### Improved

- **Activation Guidance**: Enhanced first-use onboarding flow — added a horizontal bounce animation to the arrow hint in the empty code state to draw user attention, and auto-navigates to the first imported session after import wizard completes, reducing friction.

## [v0.11.3] - 2026-03-10

### Added

- **Built-in Demo Session**: New users are greeted with a pre-loaded demo session on first launch, showcasing Mantra's core features — AI conversation replay, code timeline alignment, and narrative exploration — without needing to import their own sessions first. Demo projects and sessions are visually distinguished in the sidebar with a dedicated badge.

### Fixed

- **Project List Virtualization**: Fixed a rendering issue where the project list could appear empty when re-opening the ProjectDrawer, caused by the virtual scroller not being ready during re-mount.

## [v0.11.2] - 2026-03-08

### Improved

- **First-Launch Experience**: Streamlined the first-launch privacy notification for a cleaner, more intuitive onboarding flow.

## [v0.11.1] - 2026-03-07

### Added

- **Session ID Copy**: Added a copy button for the session ID in the TopBar dropdown, with clipboard support (including fallback for older browsers) and visual feedback via tooltip.
- **Project List Virtualization**: Implemented virtual scrolling for the project list in the ProjectDrawer component, significantly improving rendering performance for large project collections.
- **Device ID Telemetry Correlation**: The update checker now sends a device ID header for cross-system telemetry correlation, allowing better insights into update adoption across devices.
- **i18n**: Added missing translations for DiffModeToggle labels/tooltips and ContentBlockRenderer line count and expand prompts in both English and Chinese.

### Fixed

- **macOS WKWebView Rendering**: Fixed a visual occlusion bug where the backdrop-filter on the Hub page's sticky header caused rendering artifacts during re-paints. Resolved by forcing a separate compositing layer via `transform` style.
- **Database Migration**: Improved database migration from the root data directory to a subdirectory for Tauri-native installations, ensuring compatibility with older builds and better error handling during the migration process.

## [v0.11.0] - 2026-03-03

### Added

- **AI Context Mapping (Epic 18)**:
  Mantra now understands the "why" behind AI actions by mapping conversation context directly to code changes at the data layer.
  - **Mentioned Files Extraction**: Enhanced parser intelligence to automatically extract all files referenced by the AI across all four parsers and live watchers, providing richer context for code exploration.
  - **Reference Content Blocks**: Tool execution results (file reads, searches, etc.) are promoted to standalone reference blocks within the narrative, enabling cross-message context tracking.
  - **Causality Mapping Engine**: Automatically identifies relationships between AI-read context and resulting code modifications, with a cost-efficient three-tier analysis (heuristic pre-filter, semantic analysis, persistence). Works offline in heuristic mode when no API key is configured.

- **Deterministic Session Replay (Epic 19)**:
  A powerful "Replay" capability that allows re-executing AI sessions in a controlled environment to verify, understand, and learn from AI coding processes.
  - **Operation Extraction**: Automatically extracts replayable operations (file creation, modification, deletion, command execution) from session logs into a structured replay plan.
  - **Git Checkpoint Management**: Automated management of temporary Git shadow repositories to ensure the workspace precisely matches the original session state at every replay step.
  - **Multi-Level Security Sandbox**: Safe execution environment with three configurable isolation levels (Display-Only, Filesystem Isolation, Full Sandbox) to protect the host system during replay.
  - **Resilient Diff Execution**: Advanced four-tier matching strategy (Exact, Whitespace-Normalized, Indentation-Agnostic, Hunk-Split) to apply code changes reliably even in evolving codebases.
  - **Step-by-Step Interactive Mode (Default)**: Each AI operation is previewed before execution — showing the AI's reasoning, the planned change, and a diff preview — allowing manual confirmation or skip before applying. Perfect for learning or auditing complex changes.
  - **Auto-Play Mode**: Switch to automatic execution with adjustable speed (1x / 2x / 5x), pause at any time to return to step-by-step mode.
  - **Zero-Friction Entry**: Default workspace is auto-created per session, eliminating the need for manual directory selection. A guided entry card explains what will happen and provides an option to choose a custom directory.
  - **Workspace Safety Validation**: Multi-layer protection prevents replay from accidentally modifying important directories (system paths, home directory, existing Git repos). Validation runs at both the UI and engine layers for defense in depth.
  - **Replay Player UI**: Interactive panel with AI reasoning display, live diff preview, step timeline with status indicators (completed/skipped/pending), and a completion summary page with statistics and quick actions.
  - **Replay Workspace Settings**: Configurable default replay workspace location in the Settings page, with browse and reset-to-default options.

- **Enhanced Telemetry & Privacy Control**:
  - Expanded telemetry coverage to 14 core events for better feature usage insights.
  - New privacy-first telemetry consent system, allowing users to opt-in or out of anonymous analytics at any time.

- **Unified App Directory Strategy**:
  Mantra now follows platform-standard directory conventions on all operating systems, replacing the legacy `~/.mantra/` hardcoded path.
  - **Platform-Standard Directories**: Data, configuration, cache, and log files are now stored in OS-standard locations (Linux XDG, macOS `~/Library`, Windows `%APPDATA%`), managed by a centralized PathResolver service.
  - **Automatic Migration**: On first launch after upgrade, existing data in `~/.mantra/` is seamlessly migrated to the new standard locations. Partial failures are retried on next launch.
  - **Config/Data Separation**: Configuration files (settings, privacy rules) and data files (sessions, skills, backups) are properly separated into distinct directories, following OS conventions.

- **UI/UX Enhancements**:
  - **Enhanced Mode Switcher**: Support for a four-state mode system (Narrative, Analytics, Compact, Replay).
  - **Replay Branding**: Replay mode is branded as "Replay" (English) / "推演" (Chinese), distinct from "Playback" / "回放".
  - **Visual Consistency**: Significant updates to Replay components for a more cohesive and responsive layout.
  - **Update Visibility**: Improved update badge logic in the TopBar for better visibility of new versions.

### Changed

- **Causality UI Deferred**: The frontend causality highlighting visualization (Epic 18.4) has been deferred for UX redesign. The causality data layer is fully operational and will power an improved visualization in a future release.

### Fixed

- **Replay**: Fixed an issue where the replay engine would fail on files with unconventional indentation.
- **UI**: Fixed update badge visibility logic in the TopBar.

## [v0.10.0] - 2026-02-22

### Added

- **Session Live Streaming (Epic 16)**:
  Mantra now supports real-time session updates — open an in-progress AI coding session and watch new messages appear live, like watching a livestream.
  - **Unified Session Watcher**: A unified session watcher abstraction supports all four data sources (Claude Code, Codex, Cursor, Gemini CLI), serving as the foundation for both local and remote live updates.
  - **Local JSONL Live Watch**: For Claude Code and Codex sessions, new messages are detected via file monitoring with incremental reading — only new content is processed, keeping resource usage minimal.
  - **Local Cursor Live Watch**: For Cursor sessions, new messages are detected via periodic read-only database polling, ensuring no interference with Cursor's own operations.
  - **Local Gemini Live Watch**: For Gemini CLI sessions, file changes trigger a full re-parse with diff comparison to extract new messages.
  - **Live Message Rendering**: New messages appear in the narrative panel with fade-in animation. Smart scrolling auto-follows when you're at the bottom, or shows a "N new messages" floating button when you've scrolled up.
  - **Live Status Indicator**: A status indicator next to the session title shows the current watch state — Live, Stopped, or Reconnecting.
  - **Error Recovery**: Automatic reconnection with exponential backoff (up to 3 retries). Manual reconnect button available when automatic recovery fails. Idle detection lowers polling frequency when a session appears to have ended.

- **Remote SSH Project Access (Epic 17)**:
  Connect to remote servers via SSH and manage AI coding sessions as if they were local — import, browse files, and watch live updates, all without installing anything on the remote server.
  - **Zero-Config SSH Connection**: Reads your existing `~/.ssh/config` to auto-discover remote hosts. Supports key file, ssh-agent, and password authentication with automatic fallback.
  - **SSH Connection Persistence**: Successfully connected servers are saved locally for one-click reuse, sorted by last-used time on the import page.
  - **File System Abstraction Layer**: A unified file system interface makes local and remote file operations interchangeable — all import, scan, and browse operations work identically regardless of data source location.
  - **Remote Source Auto-Detection**: After connecting to a remote server, Mantra automatically detects available AI tool sessions (Claude Code, Cursor, Gemini CLI, Codex) with estimated session counts.
  - **Remote Session Import**: Import sessions from remote servers via SFTP streaming (for JSONL sources) or remote command execution (for Cursor). No files are downloaded to your local machine.
  - **Import Page Redesign**: The import page now uses a flat, location-grouped layout — local sources on top, each remote server below with its detected sessions. Add new remote servers from the SSH host list or manual input.
  - **Remote Project File Browsing**: Browse remote project directories and read source code files directly in the session detail view, with memory cache for frequently accessed files.
  - **Remote Session Live Updates**: Watch in-progress remote sessions in real-time — JSONL sources via streaming tail, Cursor via periodic remote database queries, and Gemini via remote file polling. Automatic reconnection from the last offset on SSH disconnection.
  - **SSH Connection Pool & Keepalive**: Multiple operations (file browsing, live watch, import) share a single SSH connection with automatic channel multiplexing. Keepalive messages prevent idle timeout. Disconnected connections auto-recover on next use with a 60-second grace period before cleanup.
  - **Remote Project Sync**: The "Sync" button works for remote projects — scans the remote server for new sessions and message updates without re-importing, with automatic SSH reconnection on network interruptions.

### Fixed

- **Live Watch**: Fixed session live watch not starting correctly for Claude Code sessions.
- **Remote Live Recovery**: Fixed automatic recovery of remote session live updates after SSH disconnection — the connection ID is now preserved during disconnections, enabling seamless reconnection.

## [v0.9.1] - 2026-02-15

### Added

- **Search Result Message Navigation (Story 2.36)**:
  - **Precise Message Positioning**: Clicking a search result now scrolls directly to the matching message with smooth animation and center alignment.
  - **Message Highlight**: The focused message displays a blue left border with a pulsing background glow; other matches in the same session show a subtle blue marker.
  - **In-Message Keyword Highlighting**: Matched keywords within message text are highlighted in yellow, case-insensitive. Code blocks and thinking blocks are excluded to avoid visual noise.
  - **Multi-Match Navigator**: When a session has 2+ matches, a floating navigation bar appears at the top-right of the narrative panel — showing the keyword, match counter (e.g. "2/5"), and Previous/Next buttons with wrap-around.
  - **Keyboard Shortcuts**: `Enter` jumps to the next match, `Shift+Enter` to the previous, and `Esc` dismisses all highlights.
  - **i18n**: Full English and Chinese translations for all navigator labels.

- **Skills Hub — Project Detail Integration Enhancement (Story 15.15)**:
  - **SkillContextCard**: New compact overview card embedded in the project detail sheet, showing associated skill count, tool brand icons, and expandable skill list — symmetric with MCP Hub's project integration.
  - **Unmanaged Skills Alert**: When unmanaged skills are detected in a project directory, an amber banner appears inside the card with a one-click import button.
  - **Cross-Page Navigation**: Bidirectional navigation between Skills Hub and project detail pages — skill cards link to associated projects, project detail links back to Skills Hub.
  - **Associated Skills Search**: When a project has more than 5 associated skills, a search box appears for quick filtering by name or description.

### Fixed

- **Search**: Fixed search result navigation losing message positioning parameters when switching between Compress/Analytics and Playback modes.

## [v0.9.0] - 2026-02-14

### Added

- **Skills Hub — Unified Skills Management (Epic 15)**:
  Skills Hub is the second pillar of Mantra's "Unified Configuration Plane" (alongside MCP Hub). It brings scattered AI tool skills into one place: **import once, use everywhere**.
  - **Multi-Tool Skills Scanning**: Automatically discover skills across Claude Code, Cursor, Codex, and Gemini CLI — both user-level and project-level directories.
  - **Smart Import with Three-Tier Preview**: Before importing, the system classifies each detected skill as "auto import", "auto skip", or "needs decision" based on content comparison, so you always know what will happen.
  - **Safe Takeover with Automatic Backup**: Original skill directories are backed up before import. If anything goes wrong, changes are automatically rolled back. You can also manually restore backups at any time.
  - **Cross-Tool Symlink Distribution**: Imported skills are stored centrally and distributed to each AI tool's skills directory via symlinks — all tools see the same skills without duplication.
  - **Project-Level Skill Linking**: Flexibly associate skills to projects. User-level skills automatically link to all projects; project-level skills link only to their source project. Link or unlink at any time, and tool directories update automatically.
  - **Reverse Flow Detection**: When opening a project, Mantra detects new skills created directly by AI tools (outside of Mantra) and prompts you to import them.
  - **Skills Hub Page**: A dedicated management page with overview metrics, source-tool filter chips, search, list/grid view toggle, and collapsible backup status summary.
  - **5-Step Import Wizard**: Guided import flow — Scan, Preview, Conflict Resolution, Execute, Link — with per-tool scan progress, structured diff highlighting, and post-import highlight animation.
  - **Project Detail Integration**: Skills card embedded in the project detail page showing associated skills, unmanaged skill alerts, and quick-link actions — symmetric with MCP Hub's project integration.
  - **Skill Detail & Lifecycle Management**: View full skill metadata and content, manage associated projects, delete skills with impact preview, and restore from backups with integrity verification.
  - **Cross-Page Navigation**: Seamless navigation between Skills Hub and project detail pages in both directions.
  - **Cross-Platform Support**: Works on Linux, macOS, and Windows (with automatic junction fallback on Windows).
  - **i18n**: Full English and Chinese translations for all Skills Hub features.

### Fixed

- **Skills**: Fixed an error when syncing skill links for projects with virtual paths.

## [v0.8.2] - 2026-02-12

### Added

- **MCP Project Management (Story 11.30, 11.31)**:
  - **McpManagementSheet**: New sidebar sheet for managing MCP service associations directly from the project context menu, with associated service list, detected project configurations, error handling, and retry functionality.
  - **Scoped Config Import**: `McpConfigImportSheet` now supports `initialScanResult` and `scopeFilter` props for automatic scanning and filtering based on project scope, skipping redundant scans.
  - **Detectable Config Enhancement**: `DetectableConfig` now reports new service names and counts, filtering out services already present in the Hub for accurate representation.

### Changed

- **MCP Hub Icon**: Replaced Radio icon with Plug icon in GatewayStatusCard, TopBarActions, and Hub components for improved visual clarity.

### Fixed

- **AppImage Environment**: Fixed external subprocesses (Python, etc.) failing inside AppImage by cleaning up AppImage-injected environment variables (`LD_LIBRARY_PATH`, etc.) before spawning child processes.
- **MCP Configuration Import**: Improved error handling for non-Error string messages during import; enhanced service detection with typed `DetectedServiceInfo` interface.

### Internal

- **Local Cross-Platform Release Workflow**: Added GitHub Actions workflows (`release-local.yml`, `release-act.yml`) for building on self-hosted macOS (x86_64 + ARM64), Linux, and Windows VMs via SSH.
- **Local Release Script**: New `scripts/local-release.sh` with `--publish-only`, `--repo`, and platform selection options; comprehensive documentation (`release-act-setup.zh-CN.md`, `release-local-setup.zh-CN.md`).
- **Makefile Release Targets**: Added `release-check`, `release-build`, and `release-publish` targets for streamlined cross-platform release management.
- **Automated Release Body**: CI now auto-generates release body with download links and extracts changelog entries for the published version; syncs CHANGELOG to the public repository.

## [v0.8.1] - 2026-02-09

### Added

- **Single Instance**: Added `tauri-plugin-single-instance` to prevent multiple app instances from running simultaneously; second launch focuses the existing window.

### Changed

- **Branding**: Renamed "Gateway" to "MCP Hub" across all UI text, system tray menus, tooltips, and i18n translations (zh-CN & en).
- **App Icon**: Added transparent padding (~10%) to the app icon so it visually matches the size of other applications on macOS/Windows/Linux; tray icon uses a separate full-bleed icon to remain legible at small sizes.

### Fixed

- **System Tray Status Sync**: Fixed tray status never refreshing because the tray was created without an explicit ID (`TrayIconBuilder::new()` → `TrayIconBuilder::with_id("main")`). Start/stop/restart gateway commands now correctly sync tray state.
- **System Tray Cleanup**: Removed the "Switch Project" submenu from the tray menu.

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
