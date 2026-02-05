//! MCP 协议聚合器
//!
//! Story 11.17: MCP 协议聚合器
//! Story 11.28: MCP 严格模式服务过滤
//!
//! 负责聚合所有启用 MCP 服务的 tools/resources/prompts，
//! 并统一暴露给客户端。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::models::mcp::{McpService, McpTransportType, ToolPolicy};

use super::http_transport::McpHttpClient;
use super::process_manager::{McpProcessManager, ProcessError};

// ===== 错误类型 =====

/// 聚合器错误
#[derive(Debug, Error)]
pub enum AggregatorError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Process error: {0}")]
    ProcessError(#[from] ProcessError),

    #[error("HTTP transport error: {0}")]
    HttpTransportError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Invalid tool name format: {0}")]
    InvalidToolName(String),

    #[error("Service not initialized: {0}")]
    ServiceNotInitialized(String),

    #[error("Timeout")]
    Timeout,
}

// ===== MCP 数据结构 =====

/// 聚合后的 MCP 工具
///
/// 工具名称格式为 `{service_name}/{original_tool_name}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    /// 聚合后的工具名称: `{service_name}/{tool_name}`
    pub name: String,
    /// 原始工具名称
    pub original_name: String,
    /// 所属服务 ID
    pub service_id: String,
    /// 所属服务名称
    pub service_name: String,
    /// 工具标题
    pub title: Option<String>,
    /// 工具描述
    pub description: Option<String>,
    /// 输入参数 JSON Schema
    pub input_schema: Option<serde_json::Value>,
    /// 输出参数 JSON Schema
    pub output_schema: Option<serde_json::Value>,
}

impl McpTool {
    /// 创建聚合工具
    pub fn new(
        service_id: &str,
        service_name: &str,
        original_name: &str,
        title: Option<String>,
        description: Option<String>,
        input_schema: Option<serde_json::Value>,
        output_schema: Option<serde_json::Value>,
    ) -> Self {
        Self {
            name: format!("{}/{}", service_name, original_name),
            original_name: original_name.to_string(),
            service_id: service_id.to_string(),
            service_name: service_name.to_string(),
            title,
            description,
            input_schema,
            output_schema,
        }
    }

    /// 从 MCP 响应中的工具定义创建
    pub fn from_mcp_tool(
        service_id: &str,
        service_name: &str,
        tool: &serde_json::Value,
    ) -> Option<Self> {
        let original_name = tool.get("name")?.as_str()?;
        Some(Self::new(
            service_id,
            service_name,
            original_name,
            tool.get("title").and_then(|v| v.as_str()).map(String::from),
            tool.get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            tool.get("inputSchema").cloned(),
            tool.get("outputSchema").cloned(),
        ))
    }

    /// 转换为 MCP 规范格式
    pub fn to_mcp_format(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "name": self.name,
        });

        if let Some(title) = &self.title {
            obj["title"] = serde_json::Value::String(title.clone());
        }
        if let Some(desc) = &self.description {
            obj["description"] = serde_json::Value::String(desc.clone());
        }
        if let Some(schema) = &self.input_schema {
            obj["inputSchema"] = schema.clone();
        }
        if let Some(schema) = &self.output_schema {
            obj["outputSchema"] = schema.clone();
        }

        obj
    }
}

/// 聚合后的 MCP 资源
///
/// 资源 URI 格式为 `{service_name}://{original_uri}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpResource {
    /// 聚合后的 URI: `{service_name}://{path}`
    pub uri: String,
    /// 原始 URI
    pub original_uri: String,
    /// 所属服务 ID
    pub service_id: String,
    /// 所属服务名称
    pub service_name: String,
    /// 资源名称
    pub name: Option<String>,
    /// 资源描述
    pub description: Option<String>,
    /// MIME 类型
    pub mime_type: Option<String>,
}

impl McpResource {
    /// 创建聚合资源
    pub fn new(
        service_id: &str,
        service_name: &str,
        original_uri: &str,
        name: Option<String>,
        description: Option<String>,
        mime_type: Option<String>,
    ) -> Self {
        // 为 URI 添加服务前缀以确保唯一性
        let prefixed_uri = Self::add_service_prefix(service_name, original_uri);
        Self {
            uri: prefixed_uri,
            original_uri: original_uri.to_string(),
            service_id: service_id.to_string(),
            service_name: service_name.to_string(),
            name,
            description,
            mime_type,
        }
    }

    /// 从 MCP 响应中的资源定义创建
    pub fn from_mcp_resource(
        service_id: &str,
        service_name: &str,
        resource: &serde_json::Value,
    ) -> Option<Self> {
        let original_uri = resource.get("uri")?.as_str()?;
        Some(Self::new(
            service_id,
            service_name,
            original_uri,
            resource
                .get("name")
                .and_then(|v| v.as_str())
                .map(String::from),
            resource
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            resource
                .get("mimeType")
                .and_then(|v| v.as_str())
                .map(String::from),
        ))
    }

    /// 为 URI 添加服务前缀
    ///
    /// 将原始 URI (如 file:///path, https://example.com) 转换为
    /// 聚合 URI: {service_name}:::{original_uri}
    fn add_service_prefix(service_name: &str, uri: &str) -> String {
        // 使用 ::: 作为分隔符，避免与 URI scheme 中的 :// 冲突
        format!("{}:::{}", service_name, uri)
    }

    /// 从聚合 URI 解析出服务名和原始 URI
    ///
    /// 输入: {service_name}:::{original_uri}
    /// 输出: (service_name, original_uri)
    pub fn parse_prefixed_uri(prefixed_uri: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = prefixed_uri.splitn(2, ":::").collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// 转换为 MCP 规范格式
    pub fn to_mcp_format(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "uri": self.uri,
        });

        if let Some(name) = &self.name {
            obj["name"] = serde_json::Value::String(name.clone());
        }
        if let Some(desc) = &self.description {
            obj["description"] = serde_json::Value::String(desc.clone());
        }
        if let Some(mime) = &self.mime_type {
            obj["mimeType"] = serde_json::Value::String(mime.clone());
        }

        obj
    }
}

/// 聚合后的 MCP 提示
///
/// 提示名称格式为 `{service_name}/{original_name}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPrompt {
    /// 聚合后的名称: `{service_name}/{prompt_name}`
    pub name: String,
    /// 原始名称
    pub original_name: String,
    /// 所属服务 ID
    pub service_id: String,
    /// 所属服务名称
    pub service_name: String,
    /// 提示描述
    pub description: Option<String>,
    /// 参数列表
    pub arguments: Option<Vec<serde_json::Value>>,
}

impl McpPrompt {
    /// 创建聚合提示
    pub fn new(
        service_id: &str,
        service_name: &str,
        original_name: &str,
        description: Option<String>,
        arguments: Option<Vec<serde_json::Value>>,
    ) -> Self {
        Self {
            name: format!("{}/{}", service_name, original_name),
            original_name: original_name.to_string(),
            service_id: service_id.to_string(),
            service_name: service_name.to_string(),
            description,
            arguments,
        }
    }

    /// 从 MCP 响应中的提示定义创建
    pub fn from_mcp_prompt(
        service_id: &str,
        service_name: &str,
        prompt: &serde_json::Value,
    ) -> Option<Self> {
        let original_name = prompt.get("name")?.as_str()?;
        Some(Self::new(
            service_id,
            service_name,
            original_name,
            prompt
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            prompt
                .get("arguments")
                .and_then(|v| v.as_array())
                .cloned(),
        ))
    }

    /// 转换为 MCP 规范格式
    pub fn to_mcp_format(&self) -> serde_json::Value {
        let mut obj = serde_json::json!({
            "name": self.name,
        });

        if let Some(desc) = &self.description {
            obj["description"] = serde_json::Value::String(desc.clone());
        }
        if let Some(args) = &self.arguments {
            obj["arguments"] = serde_json::Value::Array(args.clone());
        }

        obj
    }
}

// ===== 服务能力 =====

/// MCP 服务能力
#[derive(Debug, Clone, Default)]
pub struct ServiceCapabilities {
    /// 是否支持工具
    pub tools: bool,
    /// 是否支持工具列表变更通知
    pub tools_list_changed: bool,
    /// 是否支持资源
    pub resources: bool,
    /// 是否支持资源订阅
    pub resources_subscribe: bool,
    /// 是否支持资源列表变更通知
    pub resources_list_changed: bool,
    /// 是否支持提示
    pub prompts: bool,
    /// 是否支持提示列表变更通知
    pub prompts_list_changed: bool,
}

impl ServiceCapabilities {
    /// 从 MCP initialize 响应中解析能力
    pub fn from_initialize_response(response: &serde_json::Value) -> Self {
        let capabilities = response
            .get("result")
            .and_then(|r| r.get("capabilities"))
            .unwrap_or(&serde_json::Value::Null);

        let tools = capabilities.get("tools");
        let resources = capabilities.get("resources");
        let prompts = capabilities.get("prompts");

        Self {
            tools: tools.is_some(),
            tools_list_changed: tools
                .and_then(|t| t.get("listChanged"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            resources: resources.is_some(),
            resources_subscribe: resources
                .and_then(|r| r.get("subscribe"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            resources_list_changed: resources
                .and_then(|r| r.get("listChanged"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            prompts: prompts.is_some(),
            prompts_list_changed: prompts
                .and_then(|p| p.get("listChanged"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}

// ===== 服务缓存 =====

/// 单个服务的缓存数据
#[derive(Debug, Clone, Default)]
pub struct ServiceCache {
    /// 服务 ID
    pub service_id: String,
    /// 服务名称
    pub service_name: String,
    /// 服务能力
    pub capabilities: ServiceCapabilities,
    /// 工具列表
    pub tools: Vec<McpTool>,
    /// 资源列表
    pub resources: Vec<McpResource>,
    /// 提示列表
    pub prompts: Vec<McpPrompt>,
    /// 是否已初始化
    pub initialized: bool,
    /// 最后更新时间
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// 初始化错误（如果有）
    pub error: Option<String>,
}

// ===== 聚合器 =====

/// MCP 协议聚合器
///
/// 负责聚合所有启用 MCP 服务的 tools/resources/prompts
pub struct McpAggregator {
    /// stdio 进程管理器
    process_manager: Arc<McpProcessManager>,
    /// HTTP 客户端缓存 (service_id -> McpHttpClient)
    http_clients: RwLock<HashMap<String, Arc<McpHttpClient>>>,
    /// 服务配置 (service_id -> McpService)
    services: RwLock<HashMap<String, McpService>>,
    /// 服务缓存 (service_id -> ServiceCache)
    pub(crate) cache: RwLock<HashMap<String, ServiceCache>>,
    /// 服务名称到 ID 的映射
    name_to_id: RwLock<HashMap<String, String>>,
}

impl McpAggregator {
    /// 创建新的聚合器
    ///
    /// # Arguments
    /// * `services` - 预加载的 MCP 服务配置列表
    pub fn new(services: Vec<McpService>) -> Self {
        let mut services_map = HashMap::new();
        let mut name_to_id = HashMap::new();

        for service in services {
            name_to_id.insert(service.name.clone(), service.id.clone());
            services_map.insert(service.id.clone(), service);
        }

        Self {
            process_manager: Arc::new(McpProcessManager::new()),
            http_clients: RwLock::new(HashMap::new()),
            services: RwLock::new(services_map),
            cache: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(name_to_id),
        }
    }

    /// 创建带自定义进程管理器的聚合器
    pub fn with_process_manager(
        services: Vec<McpService>,
        process_manager: Arc<McpProcessManager>,
    ) -> Self {
        let mut services_map = HashMap::new();
        let mut name_to_id = HashMap::new();

        for service in services {
            name_to_id.insert(service.name.clone(), service.id.clone());
            services_map.insert(service.id.clone(), service);
        }

        Self {
            process_manager,
            http_clients: RwLock::new(HashMap::new()),
            services: RwLock::new(services_map),
            cache: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(name_to_id),
        }
    }

    /// 预热所有服务
    ///
    /// 遍历所有启用的服务，初始化并获取工具/资源/提示列表
    pub async fn warmup(&self, env_resolver: impl Fn(&str) -> Option<String>) -> WarmupResult {
        let services = self.services.read().await;
        let enabled_services: Vec<_> = services
            .values()
            .filter(|s| s.enabled)
            .cloned()
            .collect();
        drop(services);

        let total = enabled_services.len();
        let mut succeeded = 0;
        let mut failed = 0;
        let mut errors: Vec<(String, String)> = Vec::new();

        for service in enabled_services {
            eprintln!(
                "[aggregator] Warming up service: {} ({})",
                service.name, service.id
            );

            match self.initialize_service(&service, &env_resolver).await {
                Ok(_) => {
                    succeeded += 1;
                    eprintln!("[aggregator] Service {} initialized successfully", service.name);
                }
                Err(e) => {
                    failed += 1;
                    let error_msg = e.to_string();
                    eprintln!(
                        "[aggregator] Service {} initialization failed: {}",
                        service.name, error_msg
                    );
                    errors.push((service.name.clone(), error_msg.clone()));

                    // 记录错误到缓存
                    let mut cache = self.cache.write().await;
                    cache.insert(
                        service.id.clone(),
                        ServiceCache {
                            service_id: service.id.clone(),
                            service_name: service.name.clone(),
                            error: Some(error_msg),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        WarmupResult {
            total,
            succeeded,
            failed,
            errors,
        }
    }

    /// 初始化单个服务
    async fn initialize_service(
        &self,
        service: &McpService,
        env_resolver: &impl Fn(&str) -> Option<String>,
    ) -> Result<(), AggregatorError> {
        // 根据传输类型选择初始化方式
        match service.transport_type {
            McpTransportType::Stdio => {
                self.initialize_stdio_service(service, env_resolver).await
            }
            McpTransportType::Http => self.initialize_http_service(service).await,
        }
    }

    /// 初始化 stdio 类型的服务
    async fn initialize_stdio_service(
        &self,
        service: &McpService,
        env_resolver: &impl Fn(&str) -> Option<String>,
    ) -> Result<(), AggregatorError> {
        // 解析环境变量
        let env = self.resolve_service_env(service, env_resolver);

        // 启动进程
        self.process_manager.get_or_spawn(service, env).await?;

        // 发送 initialize 请求
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "mantra-gateway",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let init_response = self
            .process_manager
            .send_request(&service.id, init_request)
            .await?;

        // 发送 notifications/initialized 通知（MCP 协议要求）
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        // 通知不需要响应，忽略发送失败
        let _ = self
            .process_manager
            .send_request(&service.id, initialized_notification)
            .await;

        // 解析能力
        let capabilities = ServiceCapabilities::from_initialize_response(&init_response);

        // 获取工具列表
        let tools = if capabilities.tools {
            self.fetch_tools_stdio(&service.id, &service.name).await?
        } else {
            Vec::new()
        };

        // 获取资源列表
        let resources = if capabilities.resources {
            self.fetch_resources_stdio(&service.id, &service.name)
                .await?
        } else {
            Vec::new()
        };

        // 获取提示列表
        let prompts = if capabilities.prompts {
            self.fetch_prompts_stdio(&service.id, &service.name).await?
        } else {
            Vec::new()
        };

        // 更新缓存
        let mut cache = self.cache.write().await;
        cache.insert(
            service.id.clone(),
            ServiceCache {
                service_id: service.id.clone(),
                service_name: service.name.clone(),
                capabilities,
                tools,
                resources,
                prompts,
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        Ok(())
    }

    /// 初始化 HTTP 类型的服务
    async fn initialize_http_service(&self, service: &McpService) -> Result<(), AggregatorError> {
        let url = service
            .url
            .as_ref()
            .ok_or_else(|| AggregatorError::ProtocolError("HTTP service missing URL".to_string()))?;

        // 为此服务创建专用的 HTTP 客户端
        let http_client = Arc::new(McpHttpClient::new(
            url.clone(),
            service.headers.clone(),
        ));

        // 保存客户端供后续使用
        {
            let mut clients = self.http_clients.write().await;
            clients.insert(service.id.clone(), http_client.clone());
        }

        // 发送 initialize 请求
        let init_response = http_client
            .initialize()
            .await
            .map_err(|e| AggregatorError::HttpTransportError(e.to_string()))?;

        // 发送 notifications/initialized 通知（MCP 协议要求）
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        // 通知不需要响应，忽略发送失败
        let _ = http_client.send_request(initialized_notification).await;

        // 解析能力
        let capabilities = ServiceCapabilities::from_initialize_response(&init_response);

        // 获取工具列表
        let tools = if capabilities.tools {
            self.fetch_tools_http(&http_client, &service.id, &service.name)
                .await?
        } else {
            Vec::new()
        };

        // 获取资源列表
        let resources = if capabilities.resources {
            self.fetch_resources_http(&http_client, &service.id, &service.name)
                .await?
        } else {
            Vec::new()
        };

        // 获取提示列表
        let prompts = if capabilities.prompts {
            self.fetch_prompts_http(&http_client, &service.id, &service.name)
                .await?
        } else {
            Vec::new()
        };

        // 更新缓存
        let mut cache = self.cache.write().await;
        cache.insert(
            service.id.clone(),
            ServiceCache {
                service_id: service.id.clone(),
                service_name: service.name.clone(),
                capabilities,
                tools,
                resources,
                prompts,
                initialized: true,
                last_updated: Some(chrono::Utc::now()),
                error: None,
            },
        );

        Ok(())
    }

    /// 通过 stdio 获取工具列表
    async fn fetch_tools_stdio(
        &self,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpTool>, AggregatorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response = self.process_manager.send_request(service_id, request).await?;
        Self::parse_tools_response(&response, service_id, service_name)
    }

    /// 通过 HTTP 获取工具列表
    async fn fetch_tools_http(
        &self,
        client: &McpHttpClient,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpTool>, AggregatorError> {
        let response = client
            .list_tools()
            .await
            .map_err(|e| AggregatorError::HttpTransportError(e.to_string()))?;

        Self::parse_tools_response(&response, service_id, service_name)
    }

    /// 解析工具列表响应
    fn parse_tools_response(
        response: &serde_json::Value,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpTool>, AggregatorError> {
        let tools = response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .ok_or_else(|| AggregatorError::ProtocolError("Invalid tools/list response".to_string()))?;

        Ok(tools
            .iter()
            .filter_map(|t| McpTool::from_mcp_tool(service_id, service_name, t))
            .collect())
    }

    /// 通过 stdio 获取资源列表
    async fn fetch_resources_stdio(
        &self,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpResource>, AggregatorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "resources/list",
            "params": {}
        });

        let response = self.process_manager.send_request(service_id, request).await?;
        Self::parse_resources_response(&response, service_id, service_name)
    }

    /// 通过 HTTP 获取资源列表
    async fn fetch_resources_http(
        &self,
        client: &McpHttpClient,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpResource>, AggregatorError> {
        let response = client
            .list_resources()
            .await
            .map_err(|e| AggregatorError::HttpTransportError(e.to_string()))?;

        Self::parse_resources_response(&response, service_id, service_name)
    }

    /// 解析资源列表响应
    fn parse_resources_response(
        response: &serde_json::Value,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpResource>, AggregatorError> {
        let resources = response
            .get("result")
            .and_then(|r| r.get("resources"))
            .and_then(|r| r.as_array())
            .ok_or_else(|| {
                AggregatorError::ProtocolError("Invalid resources/list response".to_string())
            })?;

        Ok(resources
            .iter()
            .filter_map(|r| McpResource::from_mcp_resource(service_id, service_name, r))
            .collect())
    }

    /// 通过 stdio 获取提示列表
    async fn fetch_prompts_stdio(
        &self,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpPrompt>, AggregatorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "prompts/list",
            "params": {}
        });

        let response = self.process_manager.send_request(service_id, request).await?;
        Self::parse_prompts_response(&response, service_id, service_name)
    }

    /// 通过 HTTP 获取提示列表
    async fn fetch_prompts_http(
        &self,
        client: &McpHttpClient,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpPrompt>, AggregatorError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "prompts/list",
            "params": {}
        });

        let response = client
            .send_request(request)
            .await
            .map_err(|e| AggregatorError::HttpTransportError(e.to_string()))?;

        Self::parse_prompts_response(&response, service_id, service_name)
    }

    /// 解析提示列表响应
    fn parse_prompts_response(
        response: &serde_json::Value,
        service_id: &str,
        service_name: &str,
    ) -> Result<Vec<McpPrompt>, AggregatorError> {
        let prompts = response
            .get("result")
            .and_then(|r| r.get("prompts"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| {
                AggregatorError::ProtocolError("Invalid prompts/list response".to_string())
            })?;

        Ok(prompts
            .iter()
            .filter_map(|p| McpPrompt::from_mcp_prompt(service_id, service_name, p))
            .collect())
    }

    /// 解析服务环境变量
    fn resolve_service_env(
        &self,
        service: &McpService,
        env_resolver: &impl Fn(&str) -> Option<String>,
    ) -> HashMap<String, String> {
        let mut resolved = HashMap::new();

        if let Some(env) = &service.env {
            if let Some(obj) = env.as_object() {
                for (key, value) in obj {
                    if let Some(val_str) = value.as_str() {
                        // 检查是否是环境变量引用 ($VAR_NAME)
                        if let Some(var_name) = val_str.strip_prefix('$') {
                            if let Some(resolved_value) = env_resolver(var_name) {
                                resolved.insert(key.clone(), resolved_value);
                            }
                        } else {
                            resolved.insert(key.clone(), val_str.to_string());
                        }
                    }
                }
            }
        }

        resolved
    }

    // ===== 公共查询接口 =====

    /// 获取聚合的工具列表
    ///
    /// # Arguments
    /// * `policies` - 可选的服务级 Tool Policy 映射，key 为 service_id
    /// * `filter_service_ids` - 可选的服务 ID 过滤集合（严格模式）
    ///
    /// Story 11.9 Phase 2: 支持服务级独立 Tool Policy
    /// Story 11.28: 支持严格模式服务 ID 过滤
    pub async fn list_tools(
        &self,
        policies: Option<&HashMap<String, ToolPolicy>>,
        filter_service_ids: Option<&HashSet<String>>,
    ) -> Vec<McpTool> {
        let cache = self.cache.read().await;
        let mut all_tools: Vec<McpTool> = Vec::new();

        for service_cache in cache.values() {
            if !service_cache.initialized {
                continue;
            }

            // Story 11.28: 严格模式过滤 - 只保留关联服务的工具
            if let Some(filter_ids) = filter_service_ids {
                if !filter_ids.contains(&service_cache.service_id) {
                    continue;
                }
            }

            // 获取该服务的 Policy（如果有）
            let service_policy = policies.and_then(|p| p.get(&service_cache.service_id));

            // 过滤该服务的工具
            for tool in &service_cache.tools {
                if let Some(policy) = service_policy {
                    // 使用原始工具名进行 Policy 检查
                    if policy.is_tool_allowed(&tool.original_name) {
                        all_tools.push(tool.clone());
                    }
                } else {
                    // 无 Policy，允许所有工具
                    all_tools.push(tool.clone());
                }
            }
        }

        all_tools
    }

    /// 获取聚合的资源列表
    ///
    /// # Arguments
    /// * `filter_service_ids` - 可选的服务 ID 过滤集合（严格模式）
    ///
    /// Story 11.28: 支持严格模式服务 ID 过滤
    pub async fn list_resources(&self, filter_service_ids: Option<&HashSet<String>>) -> Vec<McpResource> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|c| c.initialized)
            .filter(|c| {
                // Story 11.28: 严格模式过滤
                if let Some(filter_ids) = filter_service_ids {
                    filter_ids.contains(&c.service_id)
                } else {
                    true
                }
            })
            .flat_map(|c| c.resources.clone())
            .collect()
    }

    /// 获取聚合的提示列表
    ///
    /// # Arguments
    /// * `filter_service_ids` - 可选的服务 ID 过滤集合（严格模式）
    ///
    /// Story 11.28: 支持严格模式服务 ID 过滤
    pub async fn list_prompts(&self, filter_service_ids: Option<&HashSet<String>>) -> Vec<McpPrompt> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|c| c.initialized)
            .filter(|c| {
                // Story 11.28: 严格模式过滤
                if let Some(filter_ids) = filter_service_ids {
                    filter_ids.contains(&c.service_id)
                } else {
                    true
                }
            })
            .flat_map(|c| c.prompts.clone())
            .collect()
    }

    /// 解析工具名称
    ///
    /// 从 `{service_name}/{tool_name}` 格式解析出服务名和工具名
    pub fn parse_tool_name(tool_name: &str) -> Result<(String, String), AggregatorError> {
        let parts: Vec<&str> = tool_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(AggregatorError::InvalidToolName(format!(
                "Invalid tool name format: {}, expected: service_name/tool_name",
                tool_name
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 根据服务名获取服务 ID
    pub async fn get_service_id_by_name(&self, service_name: &str) -> Option<String> {
        let name_to_id = self.name_to_id.read().await;
        name_to_id.get(service_name).cloned()
    }

    /// 获取已初始化服务的 ID 列表
    ///
    /// Story 11.9 Phase 2: 用于 Tool Policy 过滤
    pub async fn list_initialized_service_ids(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|c| c.initialized)
            .map(|c| c.service_id.clone())
            .collect()
    }

    /// 获取服务配置
    pub async fn get_service(&self, service_id: &str) -> Option<McpService> {
        let services = self.services.read().await;
        services.get(service_id).cloned()
    }

    /// 获取进程管理器
    pub fn process_manager(&self) -> &Arc<McpProcessManager> {
        &self.process_manager
    }

    /// 获取指定服务的 HTTP 客户端
    pub async fn get_http_client(&self, service_id: &str) -> Option<Arc<McpHttpClient>> {
        let clients = self.http_clients.read().await;
        clients.get(service_id).cloned()
    }

    /// 刷新单个服务的缓存
    pub async fn refresh_service(
        &self,
        service_id: &str,
        env_resolver: impl Fn(&str) -> Option<String>,
    ) -> Result<(), AggregatorError> {
        let service = {
            let services = self.services.read().await;
            services
                .get(service_id)
                .cloned()
                .ok_or_else(|| AggregatorError::ServiceNotFound(service_id.to_string()))?
        };

        self.initialize_service(&service, &env_resolver).await
    }

    /// 刷新所有服务的缓存
    pub async fn refresh_all(
        &self,
        env_resolver: impl Fn(&str) -> Option<String>,
    ) -> WarmupResult {
        self.warmup(env_resolver).await
    }

    /// 添加或更新服务配置
    pub async fn update_service(&self, service: McpService) {
        let mut services = self.services.write().await;
        let mut name_to_id = self.name_to_id.write().await;

        name_to_id.insert(service.name.clone(), service.id.clone());
        services.insert(service.id.clone(), service);
    }

    /// 移除服务
    pub async fn remove_service(&self, service_id: &str) {
        let mut services = self.services.write().await;
        let mut name_to_id = self.name_to_id.write().await;
        let mut cache = self.cache.write().await;
        let mut http_clients = self.http_clients.write().await;

        if let Some(service) = services.remove(service_id) {
            name_to_id.remove(&service.name);
        }
        cache.remove(service_id);
        http_clients.remove(service_id);

        // 停止进程
        self.process_manager.stop_process(service_id).await;
    }

    /// 停止所有进程
    pub async fn shutdown(&self) {
        self.process_manager.stop_all().await;
    }
}

/// 预热结果
#[derive(Debug, Clone)]
pub struct WarmupResult {
    /// 总服务数
    pub total: usize,
    /// 成功初始化数
    pub succeeded: usize,
    /// 失败数
    pub failed: usize,
    /// 错误列表 (服务名, 错误信息)
    pub errors: Vec<(String, String)>,
}

/// 线程安全的聚合器包装
pub type SharedMcpAggregator = Arc<McpAggregator>;

#[cfg(test)]
mod tests;
