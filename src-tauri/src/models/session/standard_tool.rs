//! Standard tool types and normalization
//!
//! Contains the StandardTool enum and normalize_tool function for unifying
//! tool semantics across different AI coding assistants.

use serde::{Deserialize, Serialize};

/// Standardized tool type enumeration
///
/// Unifies tool semantics across different import sources (Claude, Gemini, Cursor, Codex),
/// eliminating the need for frontend compatibility code.
///
/// Story 8.13: Complete application-level concept coverage.
/// All tools map to semantic types; Unknown should trend toward zero in production.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StandardTool {
    // === 文件操作 ===

    /// Read file content
    FileRead {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        end_line: Option<u32>,
    },

    /// Write/create file
    FileWrite {
        path: String,
        content: String,
    },

    /// Edit file content
    FileEdit {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_string: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_string: Option<String>,
    },

    /// Story 8.17: Delete file (Cursor delete_file)
    FileDelete {
        path: String,
    },

    // === 终端操作 ===

    /// Execute shell command
    ShellExec {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },

    // === 搜索操作 ===

    /// File search (Glob pattern matching)
    FileSearch {
        pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },

    /// Content search (Grep text search)
    ContentSearch {
        pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },

    // === 网络操作 (Story 8.13) ===

    /// Fetch web page content
    WebFetch {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
    },

    /// Search the web
    WebSearch {
        query: String,
    },

    // === 知识查询 (Story 8.13) ===

    /// Query knowledge base (MCP deepwiki, etc.)
    KnowledgeQuery {
        #[serde(skip_serializing_if = "Option::is_none")]
        repo: Option<String>,
        question: String,
    },

    // === 代码操作 (Story 8.13) ===

    /// Execute code (MCP ide executeCode)
    CodeExec {
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    /// Get diagnostics (MCP ide getDiagnostics)
    Diagnostic {
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<String>,
    },

    /// Edit Jupyter notebook cell
    NotebookEdit {
        notebook_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cell_id: Option<String>,
        new_source: String,
    },

    // === 任务管理 (Story 8.13) ===

    /// Manage todo list (TodoWrite)
    TodoManage {
        todos: serde_json::Value,
    },

    // === 代理操作 (Story 8.13) ===

    /// Launch sub-task/agent (Task tool)
    SubTask {
        prompt: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        agent_type: Option<String>,
    },

    // === 用户交互 (Story 8.13) ===

    /// Ask user a question (AskUserQuestion)
    UserPrompt {
        #[serde(skip_serializing_if = "Option::is_none")]
        question: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<serde_json::Value>,
    },

    // === 计划模式 (Story 8.13) ===

    /// Enter/exit plan mode
    PlanMode {
        entering: bool,
    },

    // === 技能调用 (Story 8.13) ===

    /// Invoke a skill
    SkillInvoke {
        skill: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<String>,
    },

    // === 真正的未知 (Story 8.13: Other → Unknown) ===

    /// Unknown tool (should trend toward zero in production)
    /// If you see many Unknown tools, consider extending StandardTool.
    Unknown {
        name: String,
        input: serde_json::Value,
    },
}

/// Structured tool result data
///
/// Preserves structured information from tool execution results (e.g., Claude toolUseResult),
/// enabling frontend to display file paths, line numbers, and other semantic information
/// without parsing strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultData {
    /// File read result
    FileRead {
        file_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        start_line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        num_lines: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        total_lines: Option<u32>,
    },

    /// File write result
    FileWrite { file_path: String },

    /// File edit result
    FileEdit {
        file_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_string: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_string: Option<String>,
    },

    /// Shell command execution result
    ShellExec {
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stdout: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        stderr: Option<String>,
    },

    /// Other result (passthrough original data)
    Other { data: serde_json::Value },
}

/// Normalizes a tool call to a StandardTool variant.
///
/// Maps tool names from various sources (Claude, Gemini, Cursor, Codex) to
/// semantic StandardTool types, extracting standardized parameters from input.
///
/// # Arguments
/// * `name` - Original tool name (e.g., "Read", "read_file", "Bash")
/// * `input` - Tool input parameters as JSON Value
///
/// # Returns
/// Standardized tool type. Unknown tools return `StandardTool::Unknown`.
///
/// # Version Suffix Handling
/// Cursor uses versioned tool names (e.g., "read_file_v2"). Version suffixes
/// are automatically stripped for matching (e.g., "read_file_v2" -> "read_file").
///
/// # Tool Name Mapping
/// | StandardTool    | Claude                  | Gemini            | Cursor                        | Codex         |
/// |-----------------|-------------------------|-------------------|-------------------------------|---------------|
/// | FileRead        | Read, read_file         | read_file         | read_file, view_file          | read_file     |
/// | FileWrite       | Write, write_file       | write_file        | write_file, write_to_file     | write_file    |
/// | FileEdit        | Edit, edit_file         | edit_file         | edit_file, replace_file_content| apply_diff   |
/// | ShellExec       | Bash, bash              | run_shell_command | run_terminal_cmd              | shell         |
/// | FileSearch      | Glob, glob              | glob              | find_by_name, list_dir        | search_files  |
/// | ContentSearch   | Grep, grep              | grep              | grep_search                   | -             |
pub fn normalize_tool(name: &str, input: &serde_json::Value) -> StandardTool {
    // Helper: extract path from input (supports file_path, path, and target_file for Cursor)
    let get_path = || -> String {
        input
            .get("file_path")
            .or_else(|| input.get("path"))
            .or_else(|| input.get("target_file")) // Cursor uses target_file for read_file
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    // Helper: extract optional string field
    let get_str = |key: &str| -> Option<String> {
        input.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    // Helper: extract optional u32 field (supports both u32 and u64 JSON numbers)
    let get_u32 = |key: &str| -> Option<u32> {
        input.get(key).and_then(|v| v.as_u64()).map(|n| n as u32)
    };

    // Normalize tool name: strip version suffixes (e.g., "_v2", "_v3")
    // Cursor uses versioned tool names like "read_file_v2"
    let normalized_name = name.to_lowercase();
    let base_name = if let Some(idx) = normalized_name.rfind("_v") {
        // Check if the suffix after "_v" is a number
        let suffix = &normalized_name[idx + 2..];
        if suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty() {
            &normalized_name[..idx]
        } else {
            normalized_name.as_str()
        }
    } else {
        normalized_name.as_str()
    };

    // Case-insensitive name matching with normalized base name
    match base_name {
        // FileRead: Read, read_file, view_file (Cursor uses view_file)
        "read" | "read_file" | "view_file" => {
            let start = get_u32("start_line").or_else(|| get_u32("offset"));
            // end_line takes priority; if not present, calculate from offset + limit
            // Note: Claude uses offset (start line) + limit (line count), so end = offset + limit
            let end = get_u32("end_line").or_else(|| {
                let offset = get_u32("offset");
                let limit = get_u32("limit");
                match (offset, limit) {
                    (Some(o), Some(l)) => Some(o.saturating_add(l)),
                    _ => None,
                }
            });
            StandardTool::FileRead {
                path: get_path(),
                start_line: start,
                end_line: end,
            }
        }

        // FileWrite: Write, write_file, write_to_file (Cursor)
        "write" | "write_file" | "write_to_file" => StandardTool::FileWrite {
            path: get_path(),
            content: get_str("content").unwrap_or_default(),
        },

        // FileEdit: Edit, edit_file, apply_diff, replace_file_content (Cursor)
        "edit" | "edit_file" | "apply_diff" | "replace_file_content" => StandardTool::FileEdit {
            path: get_path(),
            old_string: get_str("old_string").or_else(|| get_str("OldString")),
            new_string: get_str("new_string").or_else(|| get_str("NewString")).or_else(|| get_str("diff")),
        },

        // Story 8.17: FileDelete - delete_file (Cursor)
        "delete_file" => StandardTool::FileDelete {
            path: get_path(),
        },

        // ShellExec: Bash, bash, run_shell_command, run_terminal_cmd, shell
        "bash" | "run_shell_command" | "run_terminal_cmd" | "shell" => StandardTool::ShellExec {
            command: get_str("command").unwrap_or_default(),
            cwd: get_str("cwd").or_else(|| get_str("working_dir")),
        },

        // FileSearch: Glob, glob, search_files, find_by_name, list_dir, glob_file_search (Cursor)
        "glob" | "search_files" | "find_by_name" | "list_dir" | "glob_file_search" => StandardTool::FileSearch {
            pattern: get_str("pattern").or_else(|| get_str("Pattern")).unwrap_or_default(),
            path: get_str("path").or_else(|| get_str("DirectoryPath")),
        },

        // ContentSearch: Grep, grep, grep_search, codebase_search, semantic_search, ripgrep (Cursor)
        "grep" | "grep_search" | "codebase_search" | "semantic_search_full" | "ripgrep_raw_search" | "rg" => StandardTool::ContentSearch {
            pattern: get_str("pattern").or_else(|| get_str("query")).or_else(|| get_str("Query")).unwrap_or_default(),
            path: get_str("path"),
        },

        // === Story 8.13: New tool mappings ===

        // WebFetch: Fetch web page content
        "webfetch" | "web_fetch" => StandardTool::WebFetch {
            url: get_str("url").unwrap_or_default(),
            prompt: get_str("prompt"),
        },

        // WebSearch: Search the web
        "websearch" | "web_search" => StandardTool::WebSearch {
            query: get_str("query").unwrap_or_default(),
        },

        // NotebookEdit: Edit Jupyter notebook cell
        "notebookedit" | "notebook_edit" => StandardTool::NotebookEdit {
            notebook_path: get_str("notebook_path").unwrap_or_default(),
            cell_id: get_str("cell_id"),
            new_source: get_str("new_source").unwrap_or_default(),
        },

        // TodoManage: Manage todo list (TodoWrite)
        "todowrite" | "todo_write" => StandardTool::TodoManage {
            todos: input.get("todos").cloned().unwrap_or(serde_json::Value::Null),
        },

        // SubTask: Launch sub-task/agent (Task tool)
        "task" => StandardTool::SubTask {
            prompt: get_str("prompt").unwrap_or_default(),
            agent_type: get_str("subagent_type").or_else(|| get_str("agent_type")),
        },

        // UserPrompt: Ask user a question (AskUserQuestion)
        "askuserquestion" | "ask_user_question" => StandardTool::UserPrompt {
            question: get_str("question"),
            options: input.get("options").cloned(),
        },

        // PlanMode: Enter/exit plan mode
        "enterplanmode" | "enter_plan_mode" => StandardTool::PlanMode { entering: true },
        "exitplanmode" | "exit_plan_mode" => StandardTool::PlanMode { entering: false },

        // SkillInvoke: Invoke a skill
        // Claude Code: tool name "Skill", input { skill, args }
        // Gemini CLI: tool name "activate_skill", input { name }
        "skill" | "activate_skill" => StandardTool::SkillInvoke {
            skill: get_str("skill")
                .or_else(|| get_str("name"))
                .unwrap_or_default(),
            args: get_str("args"),
        },

        // TaskOutput and KillShell - treat as ShellExec variants
        "taskoutput" | "task_output" | "killshell" | "kill_shell" => StandardTool::ShellExec {
            command: format!("{}:{}", base_name, input.get("task_id").or(input.get("shell_id")).and_then(|v| v.as_str()).unwrap_or("")),
            cwd: None,
        },

        // Default case: check for MCP tool patterns before falling back to Unknown
        _ => {
            // Handle Claude MCP tools (mcp__server__function pattern with double underscores)
            if base_name.starts_with("mcp__") {
                normalize_mcp_tool(name, base_name, input)
            }
            // Story 8.17 AC7: Handle Cursor MCP tools (mcp_* pattern with single underscore)
            // Preserve original name and map to Unknown for frontend to identify via mcp_ prefix
            else if base_name.starts_with("mcp_") {
                StandardTool::Unknown {
                    name: name.to_string(),
                    input: input.clone(),
                }
            } else {
                StandardTool::Unknown {
                    name: name.to_string(),
                    input: input.clone(),
                }
            }
        }
    }
}

/// Normalizes MCP tools to appropriate StandardTool variants.
///
/// MCP tool naming convention: mcp__<server>__<function>
/// Examples:
/// - mcp__deepwiki__ask_question -> KnowledgeQuery
/// - mcp__ide__executeCode -> CodeExec
/// - mcp__ide__getDiagnostics -> Diagnostic
fn normalize_mcp_tool(original_name: &str, base_name: &str, input: &serde_json::Value) -> StandardTool {
    // Helper: extract string field
    let get_str = |key: &str| -> Option<String> {
        input.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };

    // Extract server and function from mcp__server__function pattern
    let parts: Vec<&str> = base_name.split("__").collect();
    if parts.len() >= 3 {
        let _server = parts[1];
        let function = parts[2..].join("__"); // Handle nested underscores

        match function.as_str() {
            // Knowledge query functions (deepwiki, etc.)
            "ask_question" | "read_wiki_contents" | "read_wiki_structure" => {
                StandardTool::KnowledgeQuery {
                    repo: get_str("repoName").or_else(|| get_str("repo")),
                    question: get_str("question").unwrap_or_default(),
                }
            }

            // Code execution functions
            "executecode" | "execute_code" | "run_code" => StandardTool::CodeExec {
                code: get_str("code").unwrap_or_default(),
                language: get_str("language"),
            },

            // Diagnostic functions
            "getdiagnostics" | "get_diagnostics" | "diagnostics" => StandardTool::Diagnostic {
                uri: get_str("uri").or_else(|| get_str("path")),
            },

            // Default: Unknown MCP tool
            _ => StandardTool::Unknown {
                name: original_name.to_string(),
                input: input.clone(),
            },
        }
    } else {
        // Invalid MCP tool pattern
        StandardTool::Unknown {
            name: original_name.to_string(),
            input: input.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
