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

        // ShellExec: Bash, bash, run_shell_command, run_terminal_cmd, shell
        "bash" | "run_shell_command" | "run_terminal_cmd" | "shell" => StandardTool::ShellExec {
            command: get_str("command").unwrap_or_default(),
            cwd: get_str("cwd").or_else(|| get_str("working_dir")),
        },

        // FileSearch: Glob, glob, search_files, find_by_name, list_dir (Cursor)
        "glob" | "search_files" | "find_by_name" | "list_dir" => StandardTool::FileSearch {
            pattern: get_str("pattern").or_else(|| get_str("Pattern")).unwrap_or_default(),
            path: get_str("path").or_else(|| get_str("DirectoryPath")),
        },

        // ContentSearch: Grep, grep, grep_search (Cursor)
        "grep" | "grep_search" => StandardTool::ContentSearch {
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
        "skill" => StandardTool::SkillInvoke {
            skill: get_str("skill").unwrap_or_default(),
            args: get_str("args"),
        },

        // TaskOutput and KillShell - treat as ShellExec variants
        "taskoutput" | "task_output" | "killshell" | "kill_shell" => StandardTool::ShellExec {
            command: format!("{}:{}", base_name, input.get("task_id").or(input.get("shell_id")).and_then(|v| v.as_str()).unwrap_or("")),
            cwd: None,
        },

        // Default case: check for MCP tool patterns before falling back to Unknown
        _ => {
            // Handle MCP tools (mcp__server__function pattern)
            if base_name.starts_with("mcp__") {
                normalize_mcp_tool(name, base_name, input)
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
mod tests {
    use super::*;

    // ===== StandardTool 序列化/反序列化测试 =====

    #[test]
    fn test_standard_tool_file_read_serialization() {
        let tool = StandardTool::FileRead {
            path: "/tmp/test.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""path":"/tmp/test.rs""#));
        assert!(json.contains(r#""start_line":10"#));
        assert!(json.contains(r#""end_line":20"#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_read_skip_none_lines() {
        let tool = StandardTool::FileRead {
            path: "/tmp/test.rs".to_string(),
            start_line: None,
            end_line: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""path":"/tmp/test.rs""#));
        assert!(!json.contains("start_line"));
        assert!(!json.contains("end_line"));
    }

    #[test]
    fn test_standard_tool_file_write_serialization() {
        let tool = StandardTool::FileWrite {
            path: "/tmp/output.txt".to_string(),
            content: "Hello World".to_string(),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_write""#));
        assert!(json.contains(r#""path":"/tmp/output.txt""#));
        assert!(json.contains(r#""content":"Hello World""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_file_edit_serialization() {
        let tool = StandardTool::FileEdit {
            path: "/tmp/edit.rs".to_string(),
            old_string: Some("old".to_string()),
            new_string: Some("new".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"file_edit""#));
        assert!(json.contains(r#""path":"/tmp/edit.rs""#));
        assert!(json.contains(r#""old_string":"old""#));
        assert!(json.contains(r#""new_string":"new""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_shell_exec_serialization() {
        let tool = StandardTool::ShellExec {
            command: "ls -la".to_string(),
            cwd: Some("/home/user".to_string()),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));
        assert!(json.contains(r#""command":"ls -la""#));
        assert!(json.contains(r#""cwd":"/home/user""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_standard_tool_unknown_serialization() {
        let tool = StandardTool::Unknown {
            name: "custom_tool".to_string(),
            input: serde_json::json!({"key": "value", "number": 42}),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains(r#""type":"unknown""#));
        assert!(json.contains(r#""name":"custom_tool""#));
        assert!(json.contains(r#""input""#));

        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    // ===== normalize_tool 函数测试 =====

    #[test]
    fn test_normalize_tool_claude_read() {
        let input = serde_json::json!({"file_path": "/tmp/test.rs"});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert!(start_line.is_none());
                assert!(end_line.is_none());
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_claude_read_with_lines() {
        let input = serde_json::json!({"file_path": "/tmp/test.rs", "offset": 10, "limit": 100});
        let tool = normalize_tool("Read", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/tmp/test.rs");
                assert_eq!(start_line, Some(10));
                assert_eq!(end_line, Some(110)); // 10 + 100
            }
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_cursor_versioned_read_file() {
        let input = serde_json::json!({"file_path": "/src/main.rs", "start_line": 1, "end_line": 50});
        let tool = normalize_tool("read_file_v2", &input);
        match tool {
            StandardTool::FileRead { path, start_line, end_line } => {
                assert_eq!(path, "/src/main.rs");
                assert_eq!(start_line, Some(1));
                assert_eq!(end_line, Some(50));
            }
            _ => panic!("Expected FileRead, got {:?}", tool),
        }
    }

    #[test]
    fn test_normalize_tool_case_insensitive() {
        let input = serde_json::json!({"file_path": "/tmp/test"});
        let tool1 = normalize_tool("READ", &input);
        let tool2 = normalize_tool("Read", &input);
        let tool3 = normalize_tool("read", &input);

        match (&tool1, &tool2, &tool3) {
            (
                StandardTool::FileRead { path: p1, .. },
                StandardTool::FileRead { path: p2, .. },
                StandardTool::FileRead { path: p3, .. },
            ) => {
                assert_eq!(p1, p2);
                assert_eq!(p2, p3);
            }
            _ => panic!("Expected all FileRead"),
        }
    }

    #[test]
    fn test_normalize_tool_unknown() {
        let input = serde_json::json!({"custom_param": "value"});
        let tool = normalize_tool("CustomTool", &input);
        match tool {
            StandardTool::Unknown { name, input: tool_input } => {
                assert_eq!(name, "CustomTool");
                assert_eq!(tool_input["custom_param"], "value");
            }
            _ => panic!("Expected Unknown"),
        }
    }

    #[test]
    fn test_normalize_tool_mcp_tool_knowledge_query() {
        let input = serde_json::json!({"repoName": "facebook/react", "question": "How does React work?"});
        let tool = normalize_tool("mcp__deepwiki__ask_question", &input);
        match tool {
            StandardTool::KnowledgeQuery { repo, question } => {
                assert_eq!(repo, Some("facebook/react".to_string()));
                assert_eq!(question, "How does React work?");
            }
            _ => panic!("Expected KnowledgeQuery, got {:?}", tool),
        }
    }

    // ===== ToolResultData 测试 =====

    #[test]
    fn test_tool_result_data_file_read_serialization() {
        let data = ToolResultData::FileRead {
            file_path: "/tmp/test.rs".to_string(),
            start_line: Some(10),
            num_lines: Some(100),
            total_lines: Some(500),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"file_read""#));
        assert!(json.contains(r#""file_path":"/tmp/test.rs""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    #[test]
    fn test_tool_result_data_shell_exec_serialization() {
        let data = ToolResultData::ShellExec {
            exit_code: Some(0),
            stdout: Some("output".to_string()),
            stderr: Some("error".to_string()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains(r#""type":"shell_exec""#));

        let deserialized: ToolResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, data);
    }

    // ===== Story 8.13: New StandardTool Types Tests =====

    #[test]
    fn test_normalize_tool_webfetch() {
        let input = serde_json::json!({"url": "https://example.com", "prompt": "summarize"});
        let tool = normalize_tool("WebFetch", &input);
        match tool {
            StandardTool::WebFetch { url, prompt } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(prompt, Some("summarize".to_string()));
            }
            _ => panic!("Expected WebFetch, got {:?}", tool),
        }
    }

    #[test]
    fn test_normalize_tool_task() {
        let input = serde_json::json!({"prompt": "explore the codebase", "subagent_type": "Explore"});
        let tool = normalize_tool("Task", &input);
        match tool {
            StandardTool::SubTask { prompt, agent_type } => {
                assert_eq!(prompt, "explore the codebase");
                assert_eq!(agent_type, Some("Explore".to_string()));
            }
            _ => panic!("Expected SubTask, got {:?}", tool),
        }
    }

    #[test]
    fn test_standard_tool_roundtrip_all_new_variants() {
        let new_tools = vec![
            StandardTool::WebFetch {
                url: "https://example.com".to_string(),
                prompt: None,
            },
            StandardTool::WebSearch {
                query: "test".to_string(),
            },
            StandardTool::KnowledgeQuery {
                repo: None,
                question: "test".to_string(),
            },
            StandardTool::CodeExec {
                code: "test".to_string(),
                language: None,
            },
            StandardTool::Diagnostic { uri: None },
            StandardTool::NotebookEdit {
                notebook_path: "test.ipynb".to_string(),
                cell_id: None,
                new_source: "test".to_string(),
            },
            StandardTool::TodoManage {
                todos: serde_json::json!([]),
            },
            StandardTool::SubTask {
                prompt: "test".to_string(),
                agent_type: None,
            },
            StandardTool::UserPrompt {
                question: None,
                options: None,
            },
            StandardTool::PlanMode { entering: false },
            StandardTool::SkillInvoke {
                skill: "test".to_string(),
                args: None,
            },
        ];

        for tool in new_tools {
            let json = serde_json::to_string(&tool).unwrap();
            let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, tool);
        }
    }
}
