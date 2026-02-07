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

// ===== SkillInvoke normalize_tool Tests =====

#[test]
fn test_normalize_tool_claude_skill() {
    // Claude Code: tool name "Skill", input { skill, args }
    let input = serde_json::json!({"skill": "commit", "args": "-m 'fix bug'"});
    let tool = normalize_tool("Skill", &input);
    match tool {
        StandardTool::SkillInvoke { skill, args } => {
            assert_eq!(skill, "commit");
            assert_eq!(args, Some("-m 'fix bug'".to_string()));
        }
        _ => panic!("Expected SkillInvoke, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_claude_skill_no_args() {
    let input = serde_json::json!({"skill": "review-pr"});
    let tool = normalize_tool("Skill", &input);
    match tool {
        StandardTool::SkillInvoke { skill, args } => {
            assert_eq!(skill, "review-pr");
            assert!(args.is_none());
        }
        _ => panic!("Expected SkillInvoke, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_gemini_activate_skill() {
    // Gemini CLI: tool name "activate_skill", input { name }
    let input = serde_json::json!({"name": "commit"});
    let tool = normalize_tool("activate_skill", &input);
    match tool {
        StandardTool::SkillInvoke { skill, args } => {
            assert_eq!(skill, "commit");
            assert!(args.is_none());
        }
        _ => panic!("Expected SkillInvoke, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_gemini_activate_skill_with_skill_field() {
    // Gemini: if both "skill" and "name" present, "skill" takes precedence
    let input = serde_json::json!({"skill": "primary", "name": "fallback"});
    let tool = normalize_tool("activate_skill", &input);
    match tool {
        StandardTool::SkillInvoke { skill, .. } => {
            assert_eq!(skill, "primary");
        }
        _ => panic!("Expected SkillInvoke, got {:?}", tool),
    }
}

// ===== Story 8.17: FileDelete Tests =====

#[test]
fn test_standard_tool_file_delete_serialization() {
    let tool = StandardTool::FileDelete {
        path: "/tmp/to_delete.txt".to_string(),
    };
    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains(r#""type":"file_delete""#));
    assert!(json.contains(r#""path":"/tmp/to_delete.txt""#));

    let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, tool);
}

#[test]
fn test_normalize_tool_delete_file() {
    let input = serde_json::json!({"file_path": "/src/obsolete.rs"});
    let tool = normalize_tool("delete_file", &input);
    match tool {
        StandardTool::FileDelete { path } => {
            assert_eq!(path, "/src/obsolete.rs");
        }
        _ => panic!("Expected FileDelete, got {:?}", tool),
    }
}

// ===== Story 8.17: Cursor Tool Mapping Tests =====

#[test]
fn test_normalize_tool_cursor_glob_file_search() {
    let input = serde_json::json!({"pattern": "*.rs", "path": "/src"});
    let tool = normalize_tool("glob_file_search", &input);
    match tool {
        StandardTool::FileSearch { pattern, path } => {
            assert_eq!(pattern, "*.rs");
            assert_eq!(path, Some("/src".to_string()));
        }
        _ => panic!("Expected FileSearch, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_codebase_search() {
    let input = serde_json::json!({"query": "fn main"});
    let tool = normalize_tool("codebase_search", &input);
    match tool {
        StandardTool::ContentSearch { pattern, path } => {
            assert_eq!(pattern, "fn main");
            assert!(path.is_none());
        }
        _ => panic!("Expected ContentSearch, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_semantic_search() {
    let input = serde_json::json!({"query": "error handling"});
    let tool = normalize_tool("semantic_search_full", &input);
    match tool {
        StandardTool::ContentSearch { pattern, .. } => {
            assert_eq!(pattern, "error handling");
        }
        _ => panic!("Expected ContentSearch, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_ripgrep() {
    let input = serde_json::json!({"pattern": "TODO", "path": "/project"});
    let tool = normalize_tool("ripgrep_raw_search", &input);
    match tool {
        StandardTool::ContentSearch { pattern, path } => {
            assert_eq!(pattern, "TODO");
            assert_eq!(path, Some("/project".to_string()));
        }
        _ => panic!("Expected ContentSearch, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_rg() {
    // 'rg' is an alias for ripgrep
    let input = serde_json::json!({"pattern": "FIXME"});
    let tool = normalize_tool("rg", &input);
    match tool {
        StandardTool::ContentSearch { pattern, .. } => {
            assert_eq!(pattern, "FIXME");
        }
        _ => panic!("Expected ContentSearch, got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_web_search() {
    let input = serde_json::json!({"query": "rust async await"});
    let tool = normalize_tool("web_search", &input);
    match tool {
        StandardTool::WebSearch { query } => {
            assert_eq!(query, "rust async await");
        }
        _ => panic!("Expected WebSearch, got {:?}", tool),
    }
}

// Story 8.17 AC7: Cursor MCP tool tests
#[test]
fn test_normalize_tool_cursor_mcp_single_underscore() {
    // Cursor uses single underscore MCP tool naming (mcp_*)
    let input = serde_json::json!({"url": "https://example.com"});
    let tool = normalize_tool("mcp_browser_navigate", &input);
    match tool {
        StandardTool::Unknown { name, input: tool_input } => {
            // Original name should be preserved for frontend identification
            assert_eq!(name, "mcp_browser_navigate");
            assert_eq!(tool_input.get("url").unwrap().as_str().unwrap(), "https://example.com");
        }
        _ => panic!("Expected Unknown (MCP tool), got {:?}", tool),
    }
}

#[test]
fn test_normalize_tool_cursor_mcp_fetch_resource() {
    // Test fetch_mcp_resource tool mapping
    let input = serde_json::json!({"resource": "some_resource"});
    let tool = normalize_tool("mcp_fetch_resource", &input);
    match tool {
        StandardTool::Unknown { name, .. } => {
            assert!(name.starts_with("mcp_"));
        }
        _ => panic!("Expected Unknown (MCP tool), got {:?}", tool),
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
        // Story 8.17: FileDelete
        StandardTool::FileDelete {
            path: "/tmp/delete_me.txt".to_string(),
        },
    ];

    for tool in new_tools {
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: StandardTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }
}
