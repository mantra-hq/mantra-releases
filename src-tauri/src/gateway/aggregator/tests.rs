use super::*;

#[test]
fn test_mcp_tool_new() {
    let tool = McpTool::new(
        "service-123",
        "playwright",
        "browser_click",
        Some("Click".to_string()),
        Some("Click an element".to_string()),
        Some(serde_json::json!({"type": "object"})),
        None,
    );

    assert_eq!(tool.name, "playwright/browser_click");
    assert_eq!(tool.original_name, "browser_click");
    assert_eq!(tool.service_id, "service-123");
    assert_eq!(tool.service_name, "playwright");
}

#[test]
fn test_mcp_tool_from_mcp_tool() {
    let mcp_tool = serde_json::json!({
        "name": "read_file",
        "title": "Read File",
        "description": "Read contents of a file",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            }
        }
    });

    let tool = McpTool::from_mcp_tool("svc-1", "filesystem", &mcp_tool).unwrap();

    assert_eq!(tool.name, "filesystem/read_file");
    assert_eq!(tool.original_name, "read_file");
    assert_eq!(tool.title, Some("Read File".to_string()));
    assert_eq!(tool.description, Some("Read contents of a file".to_string()));
    assert!(tool.input_schema.is_some());
}

#[test]
fn test_mcp_tool_to_mcp_format() {
    let tool = McpTool::new(
        "svc-1",
        "git",
        "list_commits",
        None,
        Some("List commits".to_string()),
        None,
        None,
    );

    let format = tool.to_mcp_format();

    assert_eq!(format["name"], "git/list_commits");
    assert_eq!(format["description"], "List commits");
    assert!(format.get("title").is_none());
}

#[test]
fn test_mcp_resource_new() {
    let resource = McpResource::new(
        "svc-1",
        "filesystem",
        "file:///home/user/test.txt",
        Some("test.txt".to_string()),
        None,
        Some("text/plain".to_string()),
    );

    // 使用 ::: 分隔符
    assert_eq!(resource.uri, "filesystem:::file:///home/user/test.txt");
    assert_eq!(resource.original_uri, "file:///home/user/test.txt");
}

#[test]
fn test_mcp_resource_parse_prefixed_uri() {
    let (service, uri) =
        McpResource::parse_prefixed_uri("filesystem:::file:///home/user/test.txt").unwrap();

    assert_eq!(service, "filesystem");
    assert_eq!(uri, "file:///home/user/test.txt");
}

#[test]
fn test_mcp_prompt_new() {
    let prompt = McpPrompt::new(
        "svc-1",
        "prompts",
        "code_review",
        Some("Code review prompt".to_string()),
        None,
    );

    assert_eq!(prompt.name, "prompts/code_review");
    assert_eq!(prompt.original_name, "code_review");
}

#[test]
fn test_parse_tool_name() {
    let (service, tool) = McpAggregator::parse_tool_name("playwright/browser_click").unwrap();
    assert_eq!(service, "playwright");
    assert_eq!(tool, "browser_click");
}

#[test]
fn test_parse_tool_name_invalid() {
    let result = McpAggregator::parse_tool_name("invalid_name");
    assert!(result.is_err());
}

#[test]
fn test_service_capabilities_from_response() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "capabilities": {
                "tools": {"listChanged": true},
                "resources": {"subscribe": true, "listChanged": true},
                "prompts": {}
            }
        }
    });

    let caps = ServiceCapabilities::from_initialize_response(&response);

    assert!(caps.tools);
    assert!(caps.tools_list_changed);
    assert!(caps.resources);
    assert!(caps.resources_subscribe);
    assert!(caps.resources_list_changed);
    assert!(caps.prompts);
    assert!(!caps.prompts_list_changed);
}

#[test]
fn test_service_capabilities_empty() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "capabilities": {}
        }
    });

    let caps = ServiceCapabilities::from_initialize_response(&response);

    assert!(!caps.tools);
    assert!(!caps.resources);
    assert!(!caps.prompts);
}

#[tokio::test]
async fn test_aggregator_new() {
    let services = vec![McpService {
        id: "svc-1".to_string(),
        name: "test-service".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "echo".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: crate::models::mcp::McpServiceSource::Manual,
        source_file: None,
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        default_tool_policy: None,
    }];

    let aggregator = McpAggregator::new(services);

    let service_id = aggregator.get_service_id_by_name("test-service").await;
    assert_eq!(service_id, Some("svc-1".to_string()));
}

#[tokio::test]
async fn test_aggregator_list_tools_empty() {
    let aggregator = McpAggregator::new(vec![]);
    let tools = aggregator.list_tools(None, None).await;
    assert!(tools.is_empty());
}

#[tokio::test]
async fn test_aggregator_update_and_remove_service() {
    let aggregator = McpAggregator::new(vec![]);

    let service = McpService {
        id: "svc-new".to_string(),
        name: "new-service".to_string(),
        transport_type: McpTransportType::Stdio,
        command: "test".to_string(),
        args: None,
        env: None,
        url: None,
        headers: None,
        source: crate::models::mcp::McpServiceSource::Manual,
        source_file: None,
        source_adapter_id: None,
        source_scope: None,
        enabled: true,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        default_tool_policy: None,
    };

    aggregator.update_service(service).await;
    assert!(aggregator.get_service("svc-new").await.is_some());

    aggregator.remove_service("svc-new").await;
    assert!(aggregator.get_service("svc-new").await.is_none());
}

/// Story 11.9 Phase 2: 测试服务级 Tool Policy 过滤
#[tokio::test]
async fn test_aggregator_list_tools_with_service_policies() {
    use crate::models::mcp::ToolPolicy;

    let aggregator = McpAggregator::new(vec![]);

    // 手动向缓存添加两个服务的工具数据
    {
        let mut cache = aggregator.cache.write().await;

        // 服务 1 有两个工具: read_file, write_file
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "read_file", None, None, None, None),
                    McpTool::new("svc-1", "service-a", "write_file", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        // 服务 2 有两个工具: list_dir, delete_file
        cache.insert(
            "svc-2".to_string(),
            ServiceCache {
                service_id: "svc-2".to_string(),
                service_name: "service-b".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-2", "service-b", "list_dir", None, None, None, None),
                    McpTool::new("svc-2", "service-b", "delete_file", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 测试 1: 无 Policy，返回所有工具
    let all_tools = aggregator.list_tools(None, None).await;
    assert_eq!(all_tools.len(), 4);

    // 测试 2: 服务 1 使用不可能匹配的 custom policy 模拟 "deny all" 行为
    // 新模型中没有 DenyAll，使用 custom 策略配合不存在的工具名来阻止所有工具
    let mut policies = HashMap::new();
    policies.insert(
        "svc-1".to_string(),
        ToolPolicy::custom(vec!["__none__".to_string()]),
    );

    let tools_with_policy = aggregator.list_tools(Some(&policies), None).await;
    assert_eq!(tools_with_policy.len(), 2); // 只有服务 2 的工具
    assert!(tools_with_policy.iter().all(|t| t.service_id == "svc-2"));

    // 测试 3: 服务 1 使用 Custom Policy（只允许 read_file）
    policies.insert(
        "svc-1".to_string(),
        ToolPolicy::custom(vec!["read_file".to_string()]),
    );

    let tools_custom = aggregator.list_tools(Some(&policies), None).await;
    assert_eq!(tools_custom.len(), 3); // 服务 1 的 read_file + 服务 2 的两个工具
    assert!(tools_custom.iter().any(|t| t.original_name == "read_file" && t.service_id == "svc-1"));
    assert!(tools_custom.iter().all(|t| t.original_name != "write_file" || t.service_id != "svc-1"));

    // 测试 4: 两个服务都有 Policy
    policies.insert(
        "svc-2".to_string(),
        ToolPolicy::custom(vec!["list_dir".to_string()]),
    );

    let tools_both = aggregator.list_tools(Some(&policies), None).await;
    assert_eq!(tools_both.len(), 2); // 服务 1 的 read_file + 服务 2 的 list_dir
}

/// Story 11.9 Phase 2: Custom Policy 部分选过滤
///
/// 注意：新模型不再支持 denied_tools，改用部分选模式
#[tokio::test]
async fn test_aggregator_list_tools_custom_partial_select() {
    use crate::models::mcp::ToolPolicy;

    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "read_file", None, None, None, None),
                    McpTool::new("svc-1", "service-a", "write_file", None, None, None, None),
                    McpTool::new("svc-1", "service-a", "delete_file", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 使用 custom 策略只允许 read_file 和 write_file（不包括 delete_file）
    let mut policies = HashMap::new();
    policies.insert(
        "svc-1".to_string(),
        ToolPolicy::custom(vec![
            "read_file".to_string(),
            "write_file".to_string(),
        ]),
    );

    let tools = aggregator.list_tools(Some(&policies), None).await;
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().all(|t| t.original_name != "delete_file"));
}

/// Story 11.9 Phase 2: 空 policies HashMap 应返回所有工具
#[tokio::test]
async fn test_aggregator_list_tools_empty_policies_map() {
    use crate::models::mcp::ToolPolicy;

    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "read_file", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 空 policies map（有 map 但 service 不在其中）
    let policies: HashMap<String, ToolPolicy> = HashMap::new();
    let tools = aggregator.list_tools(Some(&policies), None).await;
    assert_eq!(tools.len(), 1); // 无匹配 policy，返回所有工具
}

/// Story 11.9 Phase 2: 未初始化服务不返回工具
#[tokio::test]
async fn test_aggregator_list_tools_uninitialised_service_excluded() {
    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;
        // 已初始化
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "tool_a", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
        // 未初始化
        cache.insert(
            "svc-2".to_string(),
            ServiceCache {
                service_id: "svc-2".to_string(),
                service_name: "service-b".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-2", "service-b", "tool_b", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: false,
                last_updated: None,
                error: None,
            },
        );
    }

    let tools = aggregator.list_tools(None, None).await;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].service_id, "svc-1");
}

// ============================================================
// Story 11.28: MCP 严格模式服务过滤测试
// ============================================================

/// Story 11.28 Task 5.1: 测试有项目上下文时严格模式服务过滤
///
/// 当提供 filter_service_ids 时，只返回属于指定服务的工具
#[tokio::test]
async fn test_list_tools_with_strict_mode_filtering() {
    use crate::models::mcp::ToolPolicy;

    let aggregator = McpAggregator::new(vec![]);

    // 设置三个服务的缓存
    {
        let mut cache = aggregator.cache.write().await;

        // 服务 1: 项目关联
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "linked-service".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "linked-service", "read_file", None, None, None, None),
                    McpTool::new("svc-1", "linked-service", "write_file", None, None, None, None),
                ],
                resources: vec![
                    McpResource::new("svc-1", "linked-service", "file:///test", None, None, None),
                ],
                prompts: vec![
                    McpPrompt::new("svc-1", "linked-service", "code_review", None, None),
                ],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        // 服务 2: 全局启用但项目未关联
        cache.insert(
            "svc-2".to_string(),
            ServiceCache {
                service_id: "svc-2".to_string(),
                service_name: "global-service".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-2", "global-service", "list_dir", None, None, None, None),
                ],
                resources: vec![
                    McpResource::new("svc-2", "global-service", "dir:///", None, None, None),
                ],
                prompts: vec![
                    McpPrompt::new("svc-2", "global-service", "explain_code", None, None),
                ],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        // 服务 3: 另一个项目关联
        cache.insert(
            "svc-3".to_string(),
            ServiceCache {
                service_id: "svc-3".to_string(),
                service_name: "another-linked".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-3", "another-linked", "git_commit", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 严格模式过滤：只包含 svc-1 和 svc-3
    let mut filter = std::collections::HashSet::new();
    filter.insert("svc-1".to_string());
    filter.insert("svc-3".to_string());

    // 测试 tools 过滤
    let tools = aggregator.list_tools(None, Some(&filter)).await;
    assert_eq!(tools.len(), 3); // svc-1 有 2 个工具 + svc-3 有 1 个工具
    assert!(tools.iter().all(|t| t.service_id == "svc-1" || t.service_id == "svc-3"));
    assert!(tools.iter().all(|t| t.service_id != "svc-2"));

    // 测试 resources 过滤
    let resources = aggregator.list_resources(Some(&filter)).await;
    assert_eq!(resources.len(), 1); // 只有 svc-1 有资源
    assert!(resources.iter().all(|r| r.service_id == "svc-1"));

    // 测试 prompts 过滤
    let prompts = aggregator.list_prompts(Some(&filter)).await;
    assert_eq!(prompts.len(), 1); // 只有 svc-1 有提示
    assert!(prompts.iter().all(|p| p.service_id == "svc-1"));
}

/// Story 11.28 Task 5.2: 测试无 filter_service_ids 时返回全局列表
///
/// 当不提供 filter_service_ids 时，返回所有已初始化服务的资源
#[tokio::test]
async fn test_list_tools_without_strict_mode() {
    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;

        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "tool_a", None, None, None, None),
                ],
                resources: vec![
                    McpResource::new("svc-1", "service-a", "res://a", None, None, None),
                ],
                prompts: vec![
                    McpPrompt::new("svc-1", "service-a", "prompt_a", None, None),
                ],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        cache.insert(
            "svc-2".to_string(),
            ServiceCache {
                service_id: "svc-2".to_string(),
                service_name: "service-b".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-2", "service-b", "tool_b", None, None, None, None),
                ],
                resources: vec![
                    McpResource::new("svc-2", "service-b", "res://b", None, None, None),
                ],
                prompts: vec![
                    McpPrompt::new("svc-2", "service-b", "prompt_b", None, None),
                ],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 无 filter_service_ids，返回所有
    let tools = aggregator.list_tools(None, None).await;
    assert_eq!(tools.len(), 2);

    let resources = aggregator.list_resources(None).await;
    assert_eq!(resources.len(), 2);

    let prompts = aggregator.list_prompts(None).await;
    assert_eq!(prompts.len(), 2);
}

/// Story 11.28 Task 5.3: 测试 Tool Policy 与严格模式叠加
///
/// 当同时提供 policies 和 filter_service_ids 时，两种过滤条件叠加
#[tokio::test]
async fn test_list_tools_policy_and_strict_mode_combined() {
    use crate::models::mcp::ToolPolicy;

    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;

        // 服务 1: 有 3 个工具
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "read_file", None, None, None, None),
                    McpTool::new("svc-1", "service-a", "write_file", None, None, None, None),
                    McpTool::new("svc-1", "service-a", "delete_file", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        // 服务 2: 不在严格模式过滤中
        cache.insert(
            "svc-2".to_string(),
            ServiceCache {
                service_id: "svc-2".to_string(),
                service_name: "service-b".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-2", "service-b", "git_commit", None, None, None, None),
                ],
                resources: vec![],
                prompts: vec![],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 严格模式: 只包含 svc-1
    let mut filter = std::collections::HashSet::new();
    filter.insert("svc-1".to_string());

    // Tool Policy: svc-1 只允许 read_file 和 write_file
    let mut policies = HashMap::new();
    policies.insert(
        "svc-1".to_string(),
        ToolPolicy::custom(vec!["read_file".to_string(), "write_file".to_string()]),
    );

    // 叠加结果: 只有 svc-1 的 read_file 和 write_file (delete_file 被 policy 过滤, svc-2 被严格模式过滤)
    let tools = aggregator.list_tools(Some(&policies), Some(&filter)).await;
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().all(|t| t.service_id == "svc-1"));
    assert!(tools.iter().any(|t| t.original_name == "read_file"));
    assert!(tools.iter().any(|t| t.original_name == "write_file"));
    assert!(tools.iter().all(|t| t.original_name != "delete_file"));
}

/// Story 11.28 Task 5.4: 测试空 filter_service_ids 集合
///
/// 当 filter_service_ids 为空集合时，应返回空列表（项目没有关联任何服务）
#[tokio::test]
async fn test_list_tools_empty_filter_returns_empty() {
    let aggregator = McpAggregator::new(vec![]);

    {
        let mut cache = aggregator.cache.write().await;
        cache.insert(
            "svc-1".to_string(),
            ServiceCache {
                service_id: "svc-1".to_string(),
                service_name: "service-a".to_string(),
                capabilities: ServiceCapabilities::default(),
                tools: vec![
                    McpTool::new("svc-1", "service-a", "tool_a", None, None, None, None),
                ],
                resources: vec![
                    McpResource::new("svc-1", "service-a", "res://a", None, None, None),
                ],
                prompts: vec![
                    McpPrompt::new("svc-1", "service-a", "prompt_a", None, None),
                ],
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );
    }

    // 空过滤集合 = 项目没有关联任何服务
    let empty_filter = std::collections::HashSet::new();

    let tools = aggregator.list_tools(None, Some(&empty_filter)).await;
    assert!(tools.is_empty());

    let resources = aggregator.list_resources(Some(&empty_filter)).await;
    assert!(resources.is_empty());

    let prompts = aggregator.list_prompts(Some(&empty_filter)).await;
    assert!(prompts.is_empty());
}
