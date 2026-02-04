//! MCP Inspector 运行时命令（capabilities/call/resource/stop）

use std::sync::Arc;

use tauri::State;

use crate::error::AppError;
use crate::gateway::McpHttpClient;
use crate::models::mcp::{McpService, McpTransportType};

use super::{resolve_service_env, McpCapabilities, McpProcessState, McpState, McpResourceInfo, McpToolInfo};

// ===== Story 11.11: MCP Inspector 直接调用命令 =====

/// 启动 MCP 服务并获取其工具和资源列表
///
/// Story 11.11: MCP Inspector - Task 实现
///
/// 此命令会：
/// 1. 根据服务配置启动 MCP 子进程（stdio）或连接 HTTP 端点（http）
/// 2. 发送 initialize 请求
/// 3. 发送 tools/list 和 resources/list 请求
/// 4. 返回服务的完整能力列表
#[tauri::command]
pub async fn mcp_get_service_capabilities(
    service_id: String,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    // 1. 从数据库获取服务配置
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    // 根据传输类型选择不同的处理路径
    match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输模式：使用 McpHttpClient
            get_http_service_capabilities(&service, &process_state).await
        }
        McpTransportType::Stdio => {
            // stdio 传输模式：使用子进程
            get_stdio_service_capabilities(&service, &mcp_state, &process_state).await
        }
    }
}

/// 获取 HTTP 传输类型服务的能力
///
/// 创建并缓存 HTTP 客户端，后续 tools/call 和 resources/read 可复用
pub(super) async fn get_http_service_capabilities(
    service: &McpService,
    process_state: &State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    let url = service.url.as_ref().ok_or_else(|| {
        AppError::internal(format!(
            "HTTP service '{}' has no URL configured",
            service.name
        ))
    })?;

    // 创建 HTTP 客户端
    let client = McpHttpClient::new(url.clone(), service.headers.clone());

    // 1. 发送 initialize 请求
    client.initialize().await.map_err(|e| {
        AppError::internal(format!(
            "Initialize failed for HTTP service '{}' ({}): {}",
            service.name, url, e
        ))
    })?;

    // 2. 发送 initialized 通知
    let _ = client.send_initialized().await;

    // 3. 获取工具列表
    let tools: Vec<McpToolInfo> = match client.list_tools().await {
        Ok(response) => response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // 4. 获取资源列表
    let resources: Vec<McpResourceInfo> = match client.list_resources().await {
        Ok(response) => response
            .get("result")
            .and_then(|r| r.get("resources"))
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // 5. 缓存已初始化的客户端供后续 tools/call 复用
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.insert(service.id.clone(), Arc::new(client));
    }

    Ok(McpCapabilities { tools, resources })
}

/// 获取 stdio 传输类型服务的能力
pub(super) async fn get_stdio_service_capabilities(
    service: &McpService,
    mcp_state: &State<'_, McpState>,
    process_state: &State<'_, McpProcessState>,
) -> Result<McpCapabilities, AppError> {
    // 2. 解析环境变量
    let env = resolve_service_env(service, mcp_state)?;

    // 3. 启动或获取进程
    {
        let manager = process_state.manager.read().await;
        if !manager.is_running(&service.id).await {
            drop(manager);
            let manager = process_state.manager.write().await;
            manager
                .get_or_spawn(service, env.clone())
                .await
                .map_err(|e| AppError::internal(e.to_string()))?;
        }
    }

    // 4. 等待进程准备就绪（特别是对于需要网络连接的服务如 mcp-remote）
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 检查进程是否仍在运行
    {
        let manager = process_state.manager.read().await;
        if !manager.is_running(&service.id).await {
            return Err(AppError::internal(format!(
                "MCP service '{}' process exited before initialization. \
                 Command: {} {:?}. \
                 Please check if the service is correctly configured and all dependencies are installed.",
                service.name,
                service.command,
                service.args
            )));
        }
    }

    // 5. 发送 initialize 请求
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "mantra-inspector",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    {
        let manager = process_state.manager.read().await;
        let _ = manager
            .send_request(&service.id, init_request)
            .await
            .map_err(|e| {
                AppError::internal(format!(
                    "Initialize failed for '{}': {}. \
                     Command: {} {:?}",
                    service.name, e, service.command, service.args
                ))
            })?;
    }

    // 6. 发送 notifications/initialized
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    {
        let manager = process_state.manager.read().await;
        // 通知不需要响应，忽略错误
        let _ = manager
            .send_request(&service.id, initialized_notification)
            .await;
    }

    // 7. 获取工具列表
    let tools_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let tools: Vec<McpToolInfo> = {
        let manager = process_state.manager.read().await;
        match manager.send_request(&service.id, tools_request).await {
            Ok(response) => response
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| serde_json::from_value(t.clone()).ok())
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    };

    // 8. 获取资源列表
    let resources_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list",
        "params": {}
    });

    let resources: Vec<McpResourceInfo> = {
        let manager = process_state.manager.read().await;
        match manager
            .send_request(&service.id, resources_request)
            .await
        {
            Ok(response) => response
                .get("result")
                .and_then(|r| r.get("resources"))
                .and_then(|t| serde_json::from_value(t.clone()).ok())
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    };

    Ok(McpCapabilities { tools, resources })
}

/// 调用 MCP 工具
///
/// Story 11.11: MCP Inspector
/// 支持 stdio 和 HTTP 两种传输类型
#[tauri::command]
pub async fn mcp_call_tool(
    service_id: String,
    tool_name: String,
    arguments: serde_json::Value,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<serde_json::Value, AppError> {
    // 查询服务配置以确定传输类型
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": chrono::Utc::now().timestamp_millis(),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });

    let response = match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输：从缓存获取客户端或新建
            let client = get_or_create_http_client(&service, &process_state).await?;
            client
                .send_request(request)
                .await
                .map_err(|e| AppError::internal(format!("HTTP tool call failed: {}", e)))?
        }
        McpTransportType::Stdio => {
            // stdio 传输：通过进程管理器
            let manager = process_state.manager.read().await;
            manager
                .send_request(&service_id, request)
                .await
                .map_err(|e| AppError::internal(e.to_string()))?
        }
    };

    // 检查是否有错误
    if let Some(error) = response.get("error") {
        return Err(AppError::internal(format!(
            "Tool call failed: {}",
            error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
        )));
    }

    Ok(response
        .get("result")
        .cloned()
        .unwrap_or(serde_json::json!(null)))
}

/// 读取 MCP 资源
///
/// Story 11.11: MCP Inspector
/// 支持 stdio 和 HTTP 两种传输类型
#[tauri::command]
pub async fn mcp_read_resource(
    service_id: String,
    uri: String,
    mcp_state: State<'_, McpState>,
    process_state: State<'_, McpProcessState>,
) -> Result<serde_json::Value, AppError> {
    // 查询服务配置以确定传输类型
    let service = {
        let db = mcp_state.db.lock().map_err(|_| AppError::LockError)?;
        db.get_mcp_service(&service_id)?
    };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": chrono::Utc::now().timestamp_millis(),
        "method": "resources/read",
        "params": {
            "uri": uri
        }
    });

    let response = match service.transport_type {
        McpTransportType::Http => {
            // HTTP 传输：从缓存获取客户端或新建
            let client = get_or_create_http_client(&service, &process_state).await?;
            client
                .send_request(request)
                .await
                .map_err(|e| AppError::internal(format!("HTTP resource read failed: {}", e)))?
        }
        McpTransportType::Stdio => {
            // stdio 传输：通过进程管理器
            let manager = process_state.manager.read().await;
            manager
                .send_request(&service_id, request)
                .await
                .map_err(|e| AppError::internal(e.to_string()))?
        }
    };

    // 检查是否有错误
    if let Some(error) = response.get("error") {
        return Err(AppError::internal(format!(
            "Resource read failed: {}",
            error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
        )));
    }

    Ok(response
        .get("result")
        .cloned()
        .unwrap_or(serde_json::json!(null)))
}

/// 获取或创建 HTTP 客户端
///
/// 优先从缓存中获取已初始化的客户端，如果不存在则创建新客户端并初始化
async fn get_or_create_http_client(
    service: &McpService,
    process_state: &State<'_, McpProcessState>,
) -> Result<Arc<McpHttpClient>, AppError> {
    // 先尝试从缓存获取
    {
        let http_clients = process_state.http_clients.read().await;
        if let Some(client) = http_clients.get(&service.id) {
            return Ok(Arc::clone(client));
        }
    }

    // 缓存中没有，创建新客户端
    let url = service.url.as_ref().ok_or_else(|| {
        AppError::internal(format!(
            "HTTP service '{}' has no URL configured",
            service.name
        ))
    })?;

    let client = McpHttpClient::new(url.clone(), service.headers.clone());

    // 初始化连接
    client.initialize().await.map_err(|e| {
        AppError::internal(format!(
            "Initialize failed for HTTP service '{}' ({}): {}",
            service.name, url, e
        ))
    })?;
    let _ = client.send_initialized().await;

    let client = Arc::new(client);

    // 缓存客户端
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.insert(service.id.clone(), Arc::clone(&client));
    }

    Ok(client)
}

/// 停止 MCP 服务进程
///
/// Story 11.11: MCP Inspector
/// 清理 stdio 进程和 HTTP 客户端缓存
#[tauri::command]
pub async fn mcp_stop_service(
    service_id: String,
    process_state: State<'_, McpProcessState>,
) -> Result<(), AppError> {
    // 停止 stdio 进程
    let manager = process_state.manager.read().await;
    manager.stop_process(&service_id).await;

    // 清理 HTTP 客户端缓存
    {
        let mut http_clients = process_state.http_clients.write().await;
        http_clients.remove(&service_id);
    }

    Ok(())
}

/// 获取运行中的 MCP 服务列表
///
/// Story 11.11: MCP Inspector
#[tauri::command]
pub async fn mcp_list_running_services(
    process_state: State<'_, McpProcessState>,
) -> Result<Vec<String>, AppError> {
    let manager = process_state.manager.read().await;
    let running = manager.list_running().await;
    Ok(running.into_iter().map(|p| p.service_id).collect())
}
