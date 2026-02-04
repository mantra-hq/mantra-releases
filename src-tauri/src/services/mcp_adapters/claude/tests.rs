use super::*;

#[test]
fn test_claude_adapter_id_and_name() {
    let adapter = ClaudeAdapter;
    assert_eq!(adapter.id(), "claude");
    assert_eq!(adapter.name(), "Claude Code");
}

#[test]
fn test_claude_scan_patterns() {
    let adapter = ClaudeAdapter;
    let patterns = adapter.scan_patterns();

    assert_eq!(patterns.len(), 2);
    assert!(patterns.contains(&(ConfigScope::Project, ".mcp.json".to_string())));
    assert!(patterns.contains(&(ConfigScope::User, "~/.claude.json".to_string())));
}

#[test]
fn test_claude_parse_basic() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {
            "git-mcp": {
                "command": "npx",
                "args": ["-y", "@anthropic/git-mcp"]
            },
            "postgres-mcp": {
                "command": "uvx",
                "args": ["mcp-server-postgres"],
                "env": {
                    "DATABASE_URL": "$DATABASE_URL"
                }
            }
        }
    }"#;

    let path = Path::new("/test/.mcp.json");
    let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

    assert_eq!(services.len(), 2);

    let git_mcp = services.iter().find(|s| s.name == "git-mcp").unwrap();
    assert_eq!(git_mcp.command, "npx");
    assert_eq!(git_mcp.args, Some(vec!["-y".to_string(), "@anthropic/git-mcp".to_string()]));
    assert_eq!(git_mcp.adapter_id, "claude");
    assert_eq!(git_mcp.scope, ConfigScope::Project);

    let postgres_mcp = services.iter().find(|s| s.name == "postgres-mcp").unwrap();
    assert_eq!(postgres_mcp.command, "uvx");
    assert!(postgres_mcp.env.is_some());
}

#[test]
fn test_claude_parse_with_comments() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        // MCP configuration for Claude
        "mcpServers": {
            /* Git server */
            "git-mcp": {
                "command": "npx",
                "args": ["-y", "@anthropic/git-mcp"]
            }
        }
    }"#;

    let path = Path::new("/test/.mcp.json");
    let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "git-mcp");
}

#[test]
fn test_claude_parse_includes_http_servers() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {
            "local-server": {
                "command": "npx",
                "args": ["-y", "local-mcp"]
            },
            "remote-gateway": {
                "url": "http://remote.example.com/message",
                "headers": {"Authorization": "Bearer xxx"}
            }
        }
    }"#;

    let path = Path::new("/test/.mcp.json");
    let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

    // Both stdio and HTTP services should be parsed
    assert_eq!(services.len(), 2);

    let local = services.iter().find(|s| s.name == "local-server").unwrap();
    assert_eq!(local.transport_type, crate::models::mcp::McpTransportType::Stdio);
    assert_eq!(local.command, "npx");

    let remote = services.iter().find(|s| s.name == "remote-gateway").unwrap();
    assert_eq!(remote.transport_type, crate::models::mcp::McpTransportType::Http);
    assert_eq!(remote.url, Some("http://remote.example.com/message".to_string()));
    assert!(remote.headers.is_some());
}

#[test]
fn test_claude_parse_empty_servers() {
    let adapter = ClaudeAdapter;
    let content = r#"{"mcpServers": {}}"#;

    let path = Path::new("/test/.mcp.json");
    let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

    assert!(services.is_empty());
}

#[test]
fn test_claude_parse_no_servers_key() {
    let adapter = ClaudeAdapter;
    let content = r#"{"autoApprove": ["read"]}"#;

    let path = Path::new("/test/.mcp.json");
    let services = adapter.parse(path, content, ConfigScope::Project).unwrap();

    assert!(services.is_empty());
}

#[test]
fn test_claude_inject_gateway() {
    let adapter = ClaudeAdapter;
    let original = r#"{
        "autoApprove": ["read", "write"],
        "mcpServers": {
            "old-server": {"command": "old"}
        }
    }"#;

    let config = GatewayInjectionConfig::new(
        "http://127.0.0.1:8080/mcp",
        "test-token-123",
    );

    let result = adapter.inject_gateway(original, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证 autoApprove 保留
    assert_eq!(parsed["autoApprove"], serde_json::json!(["read", "write"]));

    // 验证 gateway 注入
    assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    assert_eq!(
        parsed["mcpServers"]["mantra-gateway"]["url"],
        "http://127.0.0.1:8080/mcp"
    );
    assert_eq!(
        parsed["mcpServers"]["mantra-gateway"]["headers"]["Authorization"],
        "Bearer test-token-123"
    );

    // 验证旧服务被移除
    assert!(parsed["mcpServers"]["old-server"].is_null());
}

#[test]
fn test_claude_inject_gateway_empty_file() {
    let adapter = ClaudeAdapter;
    let config = GatewayInjectionConfig::new(
        "http://127.0.0.1:8080/mcp",
        "token",
    );

    let result = adapter.inject_gateway("", &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
}

#[test]
fn test_claude_inject_gateway_with_permissions() {
    let adapter = ClaudeAdapter;
    let original = r#"{
        "permissions": {
            "allowedPaths": ["/home/user/projects"],
            "disallowedTools": ["dangerous_tool"]
        },
        "mcpServers": {}
    }"#;

    let config = GatewayInjectionConfig::new(
        "http://127.0.0.1:8080/mcp",
        "token",
    );

    let result = adapter.inject_gateway(original, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证 permissions 保留
    assert_eq!(
        parsed["permissions"]["allowedPaths"],
        serde_json::json!(["/home/user/projects"])
    );
    assert_eq!(
        parsed["permissions"]["disallowedTools"],
        serde_json::json!(["dangerous_tool"])
    );
}

// ===== Story 11.21: Local Scope 测试 =====

#[test]
fn test_parse_local_scopes_basic() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {
            "user-service": {"command": "npx", "args": ["-y", "user-mcp"]}
        },
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {
                    "project-a-mcp": {"command": "npx", "args": ["-y", "project-a-mcp"]}
                }
            },
            "/home/user/project-b": {
                "mcpServers": {
                    "project-b-mcp": {"command": "uvx", "args": ["project-b-mcp"]},
                    "project-b-http": {"url": "http://localhost:8080/mcp"}
                }
            }
        }
    }"#;

    let path = Path::new("/home/user/.claude.json");
    let services = adapter.parse_local_scopes(path, content).unwrap();

    // 应该有 3 个 local scope 服务（不包括顶层 user scope 的服务）
    assert_eq!(services.len(), 3);

    // 验证所有服务都是 local scope
    for service in &services {
        assert_eq!(service.scope, ConfigScope::Local);
        assert!(service.local_project_path.is_some());
        assert_eq!(service.adapter_id, "claude");
    }

    // 验证 project-a 的服务
    let project_a_service = services.iter().find(|s| s.name == "project-a-mcp").unwrap();
    assert_eq!(project_a_service.local_project_path, Some("/home/user/project-a".to_string()));
    assert_eq!(project_a_service.command, "npx");

    // 验证 project-b 的服务
    let project_b_stdio = services.iter().find(|s| s.name == "project-b-mcp").unwrap();
    assert_eq!(project_b_stdio.local_project_path, Some("/home/user/project-b".to_string()));

    let project_b_http = services.iter().find(|s| s.name == "project-b-http").unwrap();
    assert_eq!(project_b_http.transport_type, crate::models::mcp::McpTransportType::Http);
    assert_eq!(project_b_http.url, Some("http://localhost:8080/mcp".to_string()));
}

#[test]
fn test_parse_local_scopes_empty_projects() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {"user-service": {"command": "npx"}}
    }"#;

    let path = Path::new("/home/user/.claude.json");
    let services = adapter.parse_local_scopes(path, content).unwrap();

    assert!(services.is_empty());
}

#[test]
fn test_parse_local_scopes_project_without_mcp_servers() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/empty-project": {
                "allowedPaths": ["/tmp"]
            }
        }
    }"#;

    let path = Path::new("/home/user/.claude.json");
    let services = adapter.parse_local_scopes(path, content).unwrap();

    assert!(services.is_empty());
}

#[test]
fn test_parse_local_scope_for_project() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {
                    "service-a1": {"command": "a1"},
                    "service-a2": {"command": "a2"}
                }
            },
            "/home/user/project-b": {
                "mcpServers": {
                    "service-b1": {"command": "b1"}
                }
            }
        }
    }"#;

    let path = Path::new("/home/user/.claude.json");

    // 查询 project-a
    let services_a = adapter.parse_local_scope_for_project(path, content, "/home/user/project-a").unwrap();
    assert_eq!(services_a.len(), 2);

    // 查询 project-b
    let services_b = adapter.parse_local_scope_for_project(path, content, "/home/user/project-b").unwrap();
    assert_eq!(services_b.len(), 1);

    // 查询不存在的项目
    let services_none = adapter.parse_local_scope_for_project(path, content, "/home/user/nonexistent").unwrap();
    assert!(services_none.is_empty());
}

#[test]
fn test_list_local_scope_projects() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {
                    "service-a1": {"command": "a1"},
                    "service-a2": {"command": "a2"}
                }
            },
            "/home/user/project-b": {
                "mcpServers": {
                    "service-b1": {"command": "b1"}
                }
            },
            "/home/user/empty-project": {
                "allowedPaths": []
            }
        }
    }"#;

    let projects = adapter.list_local_scope_projects(content).unwrap();

    // 应该有 2 个项目（空项目被排除）
    assert_eq!(projects.len(), 2);

    // 验证按路径排序
    assert_eq!(projects[0].project_path, "/home/user/project-a");
    assert_eq!(projects[1].project_path, "/home/user/project-b");

    // 验证服务数量
    assert_eq!(projects[0].service_count, 2);
    assert_eq!(projects[1].service_count, 1);

    // 验证服务名称
    assert!(projects[0].service_names.contains(&"service-a1".to_string()));
    assert!(projects[0].service_names.contains(&"service-a2".to_string()));
    assert!(projects[1].service_names.contains(&"service-b1".to_string()));
}

#[test]
fn test_parse_local_scopes_with_comments() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        // User-level MCP servers
        "mcpServers": {},
        /* Project-specific configurations */
        "projects": {
            "/home/user/my-project": {
                "mcpServers": {
                    // Git MCP for this project
                    "git-mcp": {"command": "npx", "args": ["-y", "@anthropic/git-mcp"]}
                }
            }
        }
    }"#;

    let path = Path::new("/home/user/.claude.json");
    let services = adapter.parse_local_scopes(path, content).unwrap();

    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "git-mcp");
    assert_eq!(services[0].local_project_path, Some("/home/user/my-project".to_string()));
}

// ===== Story 11.21: Local Scope 接管（清空/恢复）测试 =====

#[test]
fn test_clear_local_scope_mcp_servers() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {
            "user-service": {"command": "npx"}
        },
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {
                    "service-a": {"command": "a"}
                },
                "allowedTools": ["*"]
            },
            "/home/user/project-b": {
                "mcpServers": {
                    "service-b1": {"command": "b1"},
                    "service-b2": {"command": "b2"}
                }
            }
        },
        "autoApprove": ["read"]
    }"#;

    let result = adapter.clear_local_scope_mcp_servers(content).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证 user scope 的 mcpServers 被保留
    assert!(parsed["mcpServers"]["user-service"].is_object());

    // 验证 project-a 的 mcpServers 被清空
    assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
    // 验证 project-a 的其他字段被保留
    assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["*"]));

    // 验证 project-b 的 mcpServers 被清空
    assert_eq!(parsed["projects"]["/home/user/project-b"]["mcpServers"], serde_json::json!({}));

    // 验证顶层其他字段被保留
    assert_eq!(parsed["autoApprove"], serde_json::json!(["read"]));
}

#[test]
fn test_clear_local_scope_for_project() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {"service-a": {"command": "a"}}
            },
            "/home/user/project-b": {
                "mcpServers": {"service-b": {"command": "b"}}
            }
        }
    }"#;

    let result = adapter.clear_local_scope_for_project(content, "/home/user/project-a").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // project-a 被清空
    assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
    // project-b 保持不变
    assert!(parsed["projects"]["/home/user/project-b"]["mcpServers"]["service-b"].is_object());
}

#[test]
fn test_clear_local_scope_nonexistent_project() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {"service-a": {"command": "a"}}
            }
        }
    }"#;

    // 清空不存在的项目不会报错
    let result = adapter.clear_local_scope_for_project(content, "/home/user/nonexistent").unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // project-a 保持不变
    assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["service-a"].is_object());
}

#[test]
fn test_inject_gateway_with_local_scope_clear() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {
            "old-user-service": {"command": "old"}
        },
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {"local-service-a": {"command": "a"}},
                "allowedTools": ["read", "write"]
            }
        },
        "permissions": {"allowedPaths": ["/tmp"]}
    }"#;

    let config = GatewayInjectionConfig::new("http://127.0.0.1:8080/mcp", "test-token");
    let result = adapter.inject_gateway_with_local_scope_clear(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证 user scope 被 gateway 替换
    assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
    assert_eq!(parsed["mcpServers"]["mantra-gateway"]["url"], "http://127.0.0.1:8080/mcp");
    assert!(parsed["mcpServers"]["old-user-service"].is_null());

    // 验证 local scope 被清空
    assert_eq!(parsed["projects"]["/home/user/project-a"]["mcpServers"], serde_json::json!({}));
    // 验证 local scope 的其他字段被保留
    assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["read", "write"]));

    // 验证顶层其他字段被保留
    assert!(parsed["permissions"]["allowedPaths"].is_array());
}

#[test]
fn test_restore_local_scope_mcp_servers_existing_project() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {"gateway": {"url": "http://..."}},
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {},
                "allowedTools": ["*"]
            }
        }
    }"#;

    let backup = serde_json::json!({
        "restored-service": {"command": "npx", "args": ["-y", "restored-mcp"]}
    });

    let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/project-a", &backup).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证 mcpServers 被恢复
    assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-service"].is_object());
    assert_eq!(
        parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-service"]["command"],
        "npx"
    );

    // 验证其他字段被保留
    assert_eq!(parsed["projects"]["/home/user/project-a"]["allowedTools"], serde_json::json!(["*"]));

    // 验证 user scope 不受影响
    assert!(parsed["mcpServers"]["gateway"].is_object());
}

#[test]
fn test_restore_local_scope_mcp_servers_new_project() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {"gateway": {"url": "http://..."}}
    }"#;

    let backup = serde_json::json!({
        "new-project-service": {"command": "new"}
    });

    let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/new-project", &backup).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 验证新项目被创建
    assert!(parsed["projects"]["/home/user/new-project"]["mcpServers"]["new-project-service"].is_object());
}

#[test]
fn test_restore_local_scope_does_not_affect_other_projects() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {}
            },
            "/home/user/project-b": {
                "mcpServers": {"existing": {"command": "existing"}}
            }
        }
    }"#;

    let backup = serde_json::json!({
        "restored-a": {"command": "a"}
    });

    let result = adapter.restore_local_scope_mcp_servers(content, "/home/user/project-a", &backup).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // project-a 被恢复
    assert!(parsed["projects"]["/home/user/project-a"]["mcpServers"]["restored-a"].is_object());

    // project-b 不受影响
    assert!(parsed["projects"]["/home/user/project-b"]["mcpServers"]["existing"].is_object());
}

#[test]
fn test_extract_local_scope_backup() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {
            "/home/user/project-a": {
                "mcpServers": {
                    "service-1": {"command": "cmd1", "args": ["arg1"]},
                    "service-2": {"url": "http://localhost:8080"}
                },
                "allowedTools": ["*"]
            }
        }
    }"#;

    let backup = adapter.extract_local_scope_backup(content, "/home/user/project-a").unwrap();

    // 验证备份内容
    assert!(backup["service-1"].is_object());
    assert_eq!(backup["service-1"]["command"], "cmd1");
    assert!(backup["service-2"].is_object());
    assert_eq!(backup["service-2"]["url"], "http://localhost:8080");
}

#[test]
fn test_extract_local_scope_backup_nonexistent() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "projects": {}
    }"#;

    let backup = adapter.extract_local_scope_backup(content, "/home/user/nonexistent").unwrap();

    // 不存在的项目返回空对象
    assert_eq!(backup, serde_json::json!({}));
}

#[test]
fn test_clear_local_scope_empty_projects() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {"user": {"command": "user"}}
    }"#;

    // 没有 projects 字段也能正常处理
    let result = adapter.clear_local_scope_mcp_servers(content).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // user scope 保持不变
    assert!(parsed["mcpServers"]["user"].is_object());
}

// ===== 测试 Gateway 启用/禁用列表处理 =====

#[test]
fn test_inject_gateway_removes_from_disabled_list() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {},
        "projects": {
            "/home/user/project": {
                "disabledMcpjsonServers": ["mantra-gateway", "other-server"],
                "mcpServers": {}
            }
        }
    }"#;

    let config = GatewayInjectionConfig {
        url: "http://127.0.0.1:39600/mcp".to_string(),
        token: "test-token".to_string(),
    };

    let result = adapter.inject_gateway(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // mantra-gateway 应该从 disabledMcpjsonServers 中移除
    let disabled = &parsed["projects"]["/home/user/project"]["disabledMcpjsonServers"];
    assert!(!disabled.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));
    // other-server 应该保留
    assert!(disabled.as_array().unwrap().iter().any(|v| v == "other-server"));
}

#[test]
fn test_inject_gateway_adds_to_enabled_list_if_nonempty() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {},
        "projects": {
            "/home/user/project": {
                "enabledMcpjsonServers": ["specific-server"],
                "mcpServers": {}
            }
        }
    }"#;

    let config = GatewayInjectionConfig {
        url: "http://127.0.0.1:39600/mcp".to_string(),
        token: "test-token".to_string(),
    };

    let result = adapter.inject_gateway(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 当 enabledMcpjsonServers 非空时，mantra-gateway 应该被添加进去
    let enabled = &parsed["projects"]["/home/user/project"]["enabledMcpjsonServers"];
    assert!(enabled.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));
    assert!(enabled.as_array().unwrap().iter().any(|v| v == "specific-server"));
}

#[test]
fn test_inject_gateway_skips_empty_enabled_list() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {},
        "projects": {
            "/home/user/project": {
                "enabledMcpjsonServers": [],
                "mcpServers": {}
            }
        }
    }"#;

    let config = GatewayInjectionConfig {
        url: "http://127.0.0.1:39600/mcp".to_string(),
        token: "test-token".to_string(),
    };

    let result = adapter.inject_gateway(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // 当 enabledMcpjsonServers 为空时，不应该添加 mantra-gateway
    let enabled = &parsed["projects"]["/home/user/project"]["enabledMcpjsonServers"];
    assert!(enabled.as_array().unwrap().is_empty());
}

#[test]
fn test_inject_gateway_handles_multiple_projects() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {},
        "projects": {
            "/project-a": {
                "disabledMcpjsonServers": ["mantra-gateway"],
                "enabledMcpjsonServers": [],
                "mcpServers": {}
            },
            "/project-b": {
                "disabledMcpjsonServers": [],
                "enabledMcpjsonServers": ["server-1"],
                "mcpServers": {}
            }
        }
    }"#;

    let config = GatewayInjectionConfig {
        url: "http://127.0.0.1:39600/mcp".to_string(),
        token: "test-token".to_string(),
    };

    let result = adapter.inject_gateway(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // project-a: mantra-gateway 应该从 disabled 中移除
    let disabled_a = &parsed["projects"]["/project-a"]["disabledMcpjsonServers"];
    assert!(!disabled_a.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));

    // project-b: mantra-gateway 应该添加到 enabled 中
    let enabled_b = &parsed["projects"]["/project-b"]["enabledMcpjsonServers"];
    assert!(enabled_b.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));
}

#[test]
fn test_inject_gateway_with_local_scope_clear_handles_enable_disable_lists() {
    let adapter = ClaudeAdapter;
    let content = r#"{
        "mcpServers": {"old": {"command": "old"}},
        "projects": {
            "/project": {
                "disabledMcpjsonServers": ["mantra-gateway"],
                "enabledMcpjsonServers": ["specific-server"],
                "mcpServers": {"local-service": {"command": "local"}}
            }
        }
    }"#;

    let config = GatewayInjectionConfig {
        url: "http://127.0.0.1:39600/mcp".to_string(),
        token: "test-token".to_string(),
    };

    let result = adapter.inject_gateway_with_local_scope_clear(content, &config).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // mcpServers 应该被清空
    assert!(parsed["projects"]["/project"]["mcpServers"].as_object().unwrap().is_empty());

    // mantra-gateway 应该从 disabled 中移除
    let disabled = &parsed["projects"]["/project"]["disabledMcpjsonServers"];
    assert!(!disabled.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));

    // mantra-gateway 应该添加到 enabled 中
    let enabled = &parsed["projects"]["/project"]["enabledMcpjsonServers"];
    assert!(enabled.as_array().unwrap().iter().any(|v| v == "mantra-gateway"));
}
