//! JSON-RPC 方法处理器
//!
//! Story 11.5: 上下文路由 - Task 4 & Task 5
//! Story 11.10: Project-Level Tool Management
//! Story 11.17: MCP 协议聚合器
//! Story 11.26: MCP Roots 机制
//! Story 11.28: MCP 严格模式服务过滤

use std::collections::HashSet;
use std::time::Duration;
use tokio::time::timeout;

use super::mcp_streamable::parse_roots_capability_from_params;
use super::{GatewayAppState, JsonRpcRequest, JsonRpcResponse};
use crate::gateway::state::SessionProjectContext;

/// 从会话中获取项目上下文
///
/// Story 11.28: MCP 严格模式服务过滤
///
/// 同时从 MCP Streamable HTTP 会话 (mcp_sessions) 和 legacy SSE 会话 (state) 中查找。
/// MCP Streamable HTTP 会话优先。
async fn get_session_project_context(
    app_state: &GatewayAppState,
    session_id: &str,
) -> Option<SessionProjectContext> {
    // 1. 首先尝试从 MCP Streamable HTTP 会话中获取 (mcp_sessions)
    let mcp_ctx = {
        let mcp_store = app_state.mcp_sessions.read().await;
        mcp_store
            .get_session(session_id)
            .and_then(|s| s.get_effective_project().cloned())
    };

    if mcp_ctx.is_some() {
        return mcp_ctx;
    }

    // 2. 如果没有找到，回退到 legacy SSE 会话 (state)
    let state = app_state.state.read().await;
    state
        .get_session(session_id)
        .and_then(|s| s.get_effective_project().cloned())
}

/// 处理 initialize 请求
///
/// Story 11.5: 上下文路由 - Task 4
/// Story 11.26: MCP Roots 机制 - Task 2
///
/// 1. 解析 capabilities.roots 检测 Client 是否支持 roots
/// 2. 保存 roots capability 到会话状态
/// 3. 返回 MCP 初始化响应
pub(super) async fn handle_initialize(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 1. 解析 roots capability (Story 11.26 AC1)
    let (supports_roots, roots_list_changed) = request
        .params
        .as_ref()
        .map(|p| parse_roots_capability_from_params(p))
        .unwrap_or((false, false));

    // 2. 保存 roots capability 到会话状态
    {
        let mut state = app_state.state.write().await;
        if let Some(session) = state.get_session_mut(session_id) {
            session.set_roots_capability(supports_roots, roots_list_changed);
        }
    }

    // 3. 记录日志 (Story 11.26 AC5)
    if supports_roots {
        eprintln!(
            "[Gateway] Session {} supports roots capability (listChanged: {})",
            session_id, roots_list_changed
        );
    } else {
        eprintln!(
            "[Gateway] Session {} does not support roots capability, using global services",
            session_id
        );
    }

    // 4. 返回 MCP 初始化响应
    // Story 11.17: 声明完整的 tools/resources/prompts capabilities
    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "subscribe": true, "listChanged": true },
                "prompts": { "listChanged": true }
            },
            "serverInfo": {
                "name": "mantra-gateway",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

/// 处理 tools/list 请求
///
/// Story 11.5: 上下文路由 - Task 5
/// Story 11.10: Project-Level Tool Management - AC 4 (Gateway 拦截 - tools/list 响应过滤)
/// Story 11.17: MCP 协议聚合器 - AC 1 (工具聚合)
/// Story 11.9 Phase 2: 工具策略完整实现 - AC 9 (Gateway 工具策略集成)
/// Story 11.28: MCP 严格模式服务过滤 - AC 1, AC 2, AC 4
///
/// 返回聚合的工具列表。根据项目的 Tool Policy 过滤返回的工具。
///
/// ## 严格模式过滤规则 (AC 1, AC 2)
/// - 有项目上下文: 仅返回关联服务的工具
/// - 无项目上下文: 返回全局启用服务的工具（回退）
///
/// ## Tool Policy 过滤规则 (AC 4)
/// - `mode = "allow_all"`: 返回所有工具（除了 deniedTools 中的）
/// - `mode = "deny_all"`: 返回空工具列表
/// - `mode = "custom"`: 仅返回 allowedTools 中且不在 deniedTools 中的工具
pub(super) async fn handle_tools_list(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // 获取会话的项目上下文
    // Story 11.28: 同时从 MCP Streamable HTTP 会话和 legacy SSE 会话中查找
    let project_context = get_session_project_context(app_state, session_id).await;

    // Story 11.28: 严格模式 - 查询项目关联的服务 ID 列表
    let filter_service_ids: Option<HashSet<String>> = match (&project_context, &app_state.project_services_client) {
        (Some(ctx), Some(client)) => {
            let service_ids = client.query_project_services(&ctx.project_id).await;
            if service_ids.is_empty() {
                eprintln!(
                    "[Gateway] No services linked to project {}, returning empty tools list",
                    ctx.project_id
                );
            } else {
                eprintln!(
                    "[Gateway] Strict mode: filtering tools to {} services for project {}",
                    service_ids.len(),
                    ctx.project_id
                );
            }
            Some(service_ids)
        }
        (Some(ctx), None) => {
            // 有项目上下文但没有查询客户端，使用全局服务（回退）
            eprintln!(
                "[Gateway] No project services client, using global services for project {}",
                ctx.project_id
            );
            None
        }
        (None, _) => {
            // 无项目上下文，使用全局服务（AC2 回退）
            eprintln!("[Gateway] No project context, using global services");
            None
        }
    };

    // Story 11.17: 从 Aggregator 获取聚合的工具列表
    let tools: Vec<serde_json::Value> = match &app_state.aggregator {
        Some(aggregator) => {
            // Story 11.9 Phase 2: 获取服务级 Tool Policy
            let policies = match &app_state.policy_resolver {
                Some(resolver) => {
                    // 获取所有已初始化服务的 ID 列表
                    let service_ids = aggregator.list_initialized_service_ids().await;

                    // 获取项目 ID（如果有）
                    let project_id = project_context.as_ref().map(|ctx| ctx.project_id.as_str());

                    // 批量获取所有服务的 Policy
                    let policies = resolver.get_policies(project_id, &service_ids).await;
                    Some(policies)
                }
                None => None,
            };

            // Story 11.28: 传递严格模式过滤参数
            let mcp_tools = aggregator.list_tools(policies.as_ref(), filter_service_ids.as_ref()).await;
            mcp_tools.iter().map(|t| t.to_mcp_format()).collect()
        }
        None => {
            // 没有 Aggregator，返回空列表
            Vec::new()
        }
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "tools": tools
        }),
    )
}

/// 处理 resources/list 请求
///
/// Story 11.17: MCP 协议聚合器 - AC 4 (资源聚合)
/// Story 11.28: MCP 严格模式服务过滤 - AC 6
///
/// 返回聚合的资源列表。
pub(super) async fn handle_resources_list(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // Story 11.28: 获取会话的项目上下文
    // 同时从 MCP Streamable HTTP 会话和 legacy SSE 会话中查找
    let project_context = get_session_project_context(app_state, session_id).await;

    // Story 11.28: 严格模式 - 查询项目关联的服务 ID 列表
    let filter_service_ids: Option<HashSet<String>> = match (&project_context, &app_state.project_services_client) {
        (Some(ctx), Some(client)) => {
            Some(client.query_project_services(&ctx.project_id).await)
        }
        _ => None,
    };

    // 从 Aggregator 获取聚合的资源列表
    let resources: Vec<serde_json::Value> = match &app_state.aggregator {
        Some(aggregator) => {
            // Story 11.28: 传递严格模式过滤参数
            let mcp_resources = aggregator.list_resources(filter_service_ids.as_ref()).await;
            mcp_resources.iter().map(|r| r.to_mcp_format()).collect()
        }
        None => {
            // 没有 Aggregator，返回空列表
            Vec::new()
        }
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "resources": resources
        }),
    )
}

/// 处理 resources/read 请求
///
/// Story 11.17: MCP 协议聚合器 - AC 5 (资源读取路由)
///
/// 读取指定资源的内容。根据 URI 前缀路由到对应的 MCP 服务。
pub(super) async fn handle_resources_read(
    app_state: &GatewayAppState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    use crate::models::mcp::McpTransportType;

    let uri = match request
        .params
        .as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|v| v.as_str())
    {
        Some(u) => u,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing uri parameter".to_string(),
            );
        }
    };

    // 解析 URI 格式: service_name://path
    let (service_name, original_uri) = match crate::gateway::aggregator::McpResource::parse_prefixed_uri(uri) {
        Some((svc, orig)) => (svc, orig),
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                format!("Invalid resource URI format: {}", uri),
            );
        }
    };

    // 检查是否有 Aggregator
    let aggregator = match &app_state.aggregator {
        Some(agg) => agg,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                "MCP Aggregator not initialized".to_string(),
            );
        }
    };

    // 获取服务 ID
    let service_id = match aggregator.get_service_id_by_name(&service_name).await {
        Some(id) => id,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // 获取服务配置
    let service = match aggregator.get_service(&service_id).await {
        Some(svc) => svc,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // 构造 MCP resources/read 请求（使用原始 URI）
    let mcp_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "method": "resources/read",
        "params": {
            "uri": original_uri
        }
    });

    // 根据传输类型转发请求（带超时控制）
    const RESOURCE_READ_TIMEOUT: Duration = Duration::from_secs(60);

    let forward_future = async {
        match service.transport_type {
            McpTransportType::Stdio => {
                aggregator.process_manager().send_request(&service_id, mcp_request).await
                    .map_err(|e| format!("Failed to read resource: {}", e))
            }
            McpTransportType::Http => {
                let http_client = aggregator.get_http_client(&service_id).await
                    .ok_or_else(|| format!("HTTP client not initialized for service: {}", service_name))?;
                http_client.send_request(mcp_request).await
                    .map_err(|e| format!("Failed to read resource: {}", e))
            }
        }
    };

    let response = match timeout(RESOURCE_READ_TIMEOUT, forward_future).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                e,
            );
        }
        Err(_) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                format!("Resource read timed out after {}s", RESOURCE_READ_TIMEOUT.as_secs()),
            );
        }
    };

    // 透传响应
    if let Some(result) = response.get("result") {
        JsonRpcResponse::success(request.id.clone(), result.clone())
    } else if let Some(error) = response.get("error") {
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603) as i32;
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        JsonRpcResponse::error(request.id.clone(), code, message.to_string())
    } else {
        JsonRpcResponse::error(
            request.id.clone(),
            -32603,
            "Invalid response from MCP service".to_string(),
        )
    }
}

/// 处理 prompts/list 请求
///
/// Story 11.17: MCP 协议聚合器 - AC 6 (提示聚合)
/// Story 11.28: MCP 严格模式服务过滤 - AC 6
///
/// 返回聚合的提示列表。
pub(super) async fn handle_prompts_list(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // Story 11.28: 获取会话的项目上下文
    // 同时从 MCP Streamable HTTP 会话和 legacy SSE 会话中查找
    let project_context = get_session_project_context(app_state, session_id).await;

    // Story 11.28: 严格模式 - 查询项目关联的服务 ID 列表
    let filter_service_ids: Option<HashSet<String>> = match (&project_context, &app_state.project_services_client) {
        (Some(ctx), Some(client)) => {
            Some(client.query_project_services(&ctx.project_id).await)
        }
        _ => None,
    };

    // 从 Aggregator 获取聚合的提示列表
    let prompts: Vec<serde_json::Value> = match &app_state.aggregator {
        Some(aggregator) => {
            // Story 11.28: 传递严格模式过滤参数
            let mcp_prompts = aggregator.list_prompts(filter_service_ids.as_ref()).await;
            mcp_prompts.iter().map(|p| p.to_mcp_format()).collect()
        }
        None => {
            // 没有 Aggregator，返回空列表
            Vec::new()
        }
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::json!({
            "prompts": prompts
        }),
    )
}

/// 处理 prompts/get 请求
///
/// Story 11.17: MCP 协议聚合器 - AC 6 (提示获取路由)
///
/// 获取指定提示的详情。根据提示名称前缀路由到对应的 MCP 服务。
pub(super) async fn handle_prompts_get(
    app_state: &GatewayAppState,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    use crate::models::mcp::McpTransportType;

    let prompt_name = match request
        .params
        .as_ref()
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
    {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing name parameter".to_string(),
            );
        }
    };

    let arguments = request
        .params
        .as_ref()
        .and_then(|p| p.get("arguments"))
        .cloned();

    // 解析提示名称格式: service_name/prompt_name
    let (service_name, original_name) = match crate::gateway::aggregator::McpAggregator::parse_tool_name(prompt_name) {
        Ok((svc, name)) => (svc, name),
        Err(_) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                format!("Invalid prompt name format: {}, expected: service_name/prompt_name", prompt_name),
            );
        }
    };

    // 检查是否有 Aggregator
    let aggregator = match &app_state.aggregator {
        Some(agg) => agg,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                "MCP Aggregator not initialized".to_string(),
            );
        }
    };

    // 获取服务 ID
    let service_id = match aggregator.get_service_id_by_name(&service_name).await {
        Some(id) => id,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // 获取服务配置
    let service = match aggregator.get_service(&service_id).await {
        Some(svc) => svc,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // 构造 MCP prompts/get 请求（使用原始提示名）
    let mut params = serde_json::json!({
        "name": original_name
    });
    if let Some(args) = arguments {
        params["arguments"] = args;
    }

    let mcp_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "method": "prompts/get",
        "params": params
    });

    // 根据传输类型转发请求（带超时控制）
    const PROMPT_GET_TIMEOUT: Duration = Duration::from_secs(60);

    let forward_future = async {
        match service.transport_type {
            McpTransportType::Stdio => {
                aggregator.process_manager().send_request(&service_id, mcp_request).await
                    .map_err(|e| format!("Failed to get prompt: {}", e))
            }
            McpTransportType::Http => {
                let http_client = aggregator.get_http_client(&service_id).await
                    .ok_or_else(|| format!("HTTP client not initialized for service: {}", service_name))?;
                http_client.send_request(mcp_request).await
                    .map_err(|e| format!("Failed to get prompt: {}", e))
            }
        }
    };

    let response = match timeout(PROMPT_GET_TIMEOUT, forward_future).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                e,
            );
        }
        Err(_) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                format!("Prompt get timed out after {}s", PROMPT_GET_TIMEOUT.as_secs()),
            );
        }
    };

    // 透传响应
    if let Some(result) = response.get("result") {
        JsonRpcResponse::success(request.id.clone(), result.clone())
    } else if let Some(error) = response.get("error") {
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603) as i32;
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        JsonRpcResponse::error(request.id.clone(), code, message.to_string())
    } else {
        JsonRpcResponse::error(
            request.id.clone(),
            -32603,
            "Invalid response from MCP service".to_string(),
        )
    }
}

/// 处理 tools/call 请求
///
/// Story 11.5: 上下文路由 - Task 7
/// Story 11.10: Project-Level Tool Management - AC 5 (Gateway 拦截 - tools/call 请求拦截)
/// Story 11.17: MCP 协议聚合器 - AC 2 (工具调用路由)
/// Story 11.28: MCP 严格模式服务过滤 - AC 5 (tools/call 路由一致性)
///
/// 1. 解析工具名称 (格式: service_name/tool_name)
/// 2. Story 11.28: 检查工具所属服务是否在当前项目上下文中可用
/// 3. 检查 Tool Policy 是否允许该工具
/// 4. 路由到对应的 MCP 服务
/// 5. 转发请求并透传响应
///
/// ## 严格模式验证 (AC 5)
/// 当 session 有项目上下文时，验证工具所属服务是否关联到该项目
/// 如果服务未关联，返回错误："Tool '{tool_name}' not available in current project context"
///
/// ## Tool Policy 拦截规则
/// 当工具被 Tool Policy 禁止时：
/// - 不转发请求到上游 MCP 服务
/// - 返回 JSON-RPC Error: `{"code": -32601, "message": "Tool not found: {tool_name}"}`
/// - 记录审计日志: `tool_blocked` 事件
pub(super) async fn handle_tools_call(
    app_state: &GatewayAppState,
    session_id: &str,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    use crate::models::mcp::McpTransportType;

    // 1. 解析工具名称和参数
    let params = match &request.params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing params".to_string(),
            );
        }
    };

    let tool_name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Missing tool name".to_string(),
            );
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    // 2. 解析工具名称格式: service_name/tool_name
    let (service_name, actual_tool_name) = match crate::gateway::aggregator::McpAggregator::parse_tool_name(tool_name) {
        Ok((svc, tool)) => (svc, tool),
        Err(_) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32602,
                "Invalid tool name format, expected: service_name/tool_name".to_string(),
            );
        }
    };

    // 3. 获取会话的项目上下文（用于 Tool Policy 和严格模式验证）
    // Story 11.28: 同时从 MCP Streamable HTTP 会话和 legacy SSE 会话中查找
    let project_context = get_session_project_context(app_state, session_id).await;

    // 4. 检查是否有 Aggregator
    let aggregator = match &app_state.aggregator {
        Some(agg) => agg,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                "MCP Aggregator not initialized".to_string(),
            );
        }
    };

    // 5. 获取服务 ID
    let service_id = match aggregator.get_service_id_by_name(&service_name).await {
        Some(id) => id,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // Story 11.28 AC5: 严格模式验证 - 检查服务是否关联到当前项目
    // TODO (Code Review M2): 考虑在 session 级别缓存 allowed_service_ids (短 TTL)，
    // 避免 tools/call 每次调用都查询 DB。当前实现保证实时性但可能成为性能瓶颈。
    if let (Some(ctx), Some(client)) = (&project_context, &app_state.project_services_client) {
        let allowed_service_ids = client.query_project_services(&ctx.project_id).await;
        if !allowed_service_ids.contains(&service_id) {
            eprintln!(
                "[Gateway] Tool '{}' blocked: service '{}' not linked to project {}",
                tool_name, service_name, ctx.project_id
            );
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Tool '{}' not available in current project context", tool_name),
            );
        }
    }

    // 6. 获取服务配置
    let service = match aggregator.get_service(&service_id).await {
        Some(svc) => svc,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32601,
                format!("Service not found: {}", service_name),
            );
        }
    };

    // 7. 构造 MCP tools/call 请求（使用原始工具名）
    let mcp_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request.id,
        "method": "tools/call",
        "params": {
            "name": actual_tool_name,
            "arguments": arguments
        }
    });

    // 8. 根据传输类型转发请求（带超时控制）
    const TOOL_CALL_TIMEOUT: Duration = Duration::from_secs(120);

    let forward_future = async {
        match service.transport_type {
            McpTransportType::Stdio => {
                aggregator.process_manager().send_request(&service_id, mcp_request).await
                    .map_err(|e| format!("Failed to call tool: {}", e))
            }
            McpTransportType::Http => {
                let http_client = aggregator.get_http_client(&service_id).await
                    .ok_or_else(|| format!("HTTP client not initialized for service: {}", service_name))?;
                http_client.send_request(mcp_request).await
                    .map_err(|e| format!("Failed to call tool: {}", e))
            }
        }
    };

    let response = match timeout(TOOL_CALL_TIMEOUT, forward_future).await {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                e,
            );
        }
        Err(_) => {
            return JsonRpcResponse::error(
                request.id.clone(),
                -32603,
                format!("Tool call timed out after {}s", TOOL_CALL_TIMEOUT.as_secs()),
            );
        }
    };

    // 9. 透传响应
    // 响应已经是完整的 JSON-RPC 格式，直接使用
    if let Some(result) = response.get("result") {
        JsonRpcResponse::success(request.id.clone(), result.clone())
    } else if let Some(error) = response.get("error") {
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603) as i32;
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        JsonRpcResponse::error(request.id.clone(), code, message.to_string())
    } else {
        JsonRpcResponse::error(
            request.id.clone(),
            -32603,
            "Invalid response from MCP service".to_string(),
        )
    }
}
