//! 公共工具函数
//!
//! Story 11.8: MCP Gateway Architecture Refactor
//!
//! 提供 JSON 和 TOML 配置文件的非破坏性合并功能。

use serde_json::Value as JsonValue;
use super::AdapterError;

/// JSON 配置非破坏性合并
///
/// 将 Gateway 配置合并到原始 JSON 配置中，仅更新 `mcpServers` 字段，
/// 保留其他所有顶层字段（如 `autoApprove`, `permissions` 等）。
///
/// # Arguments
/// * `original` - 原始 JSON 配置内容
/// * `mcp_servers_key` - MCP 服务器配置的键名（通常是 "mcpServers"）
/// * `gateway_config` - Gateway 配置 JSON 值
///
/// # Returns
/// 合并后的 JSON 字符串（格式化输出）
///
/// # Example
/// ```ignore
/// let original = r#"{"autoApprove": ["read"], "mcpServers": {"old": {}}}"#;
/// let gateway = serde_json::json!({"mantra-gateway": {"url": "...", "headers": {}}});
/// let merged = merge_json_config(original, "mcpServers", gateway)?;
/// // merged 保留 autoApprove，mcpServers 被更新
/// ```
pub fn merge_json_config(
    original: &str,
    mcp_servers_key: &str,
    gateway_config: JsonValue,
) -> Result<String, AdapterError> {
    // 尝试解析原始内容为 JSON
    let mut root: JsonValue = if original.trim().is_empty() {
        JsonValue::Object(serde_json::Map::new())
    } else {
        // 移除 JSONC 注释
        let stripped = strip_json_comments(original);
        serde_json::from_str(&stripped)?
    };

    // 确保根是对象
    let obj = root.as_object_mut().ok_or_else(|| {
        AdapterError::InvalidFormat("Root must be a JSON object".to_string())
    })?;

    // 更新 mcpServers 字段
    obj.insert(mcp_servers_key.to_string(), gateway_config);

    // 格式化输出
    serde_json::to_string_pretty(&root).map_err(AdapterError::Json)
}

/// TOML 配置非破坏性合并（保留注释和格式）
///
/// 将 Gateway 配置合并到原始 TOML 配置中，仅更新 `[mcp_servers]` 表，
/// 保留文件中的所有注释、空行和原始格式。
///
/// # Arguments
/// * `original` - 原始 TOML 配置内容
/// * `mcp_servers_section` - MCP 服务器配置的节名（如 "mcp_servers"）
/// * `gateway_toml` - Gateway 配置 TOML 字符串
///
/// # Returns
/// 合并后的 TOML 字符串（保留注释和格式）
///
/// # Example
/// ```ignore
/// let original = r#"
/// # My config
/// model = "gpt-4"
///
/// [mcp_servers]
/// old_server = { ... }
/// "#;
/// let gateway = r#"mantra-gateway = { url = "...", http_headers = {...} }"#;
/// let merged = merge_toml_config(original, "mcp_servers", gateway)?;
/// // merged 保留 model 和注释，[mcp_servers] 被更新
/// ```
pub fn merge_toml_config(
    original: &str,
    mcp_servers_section: &str,
    gateway_toml: &str,
) -> Result<String, AdapterError> {
    use toml_edit::{DocumentMut, Item, Table};

    // 解析原始文档
    let mut doc: DocumentMut = if original.trim().is_empty() {
        DocumentMut::new()
    } else {
        original.parse().map_err(|e: toml_edit::TomlError| {
            AdapterError::Toml(e.to_string())
        })?
    };

    // 解析 gateway 配置
    let gateway_doc: DocumentMut = gateway_toml.parse().map_err(|e: toml_edit::TomlError| {
        AdapterError::Toml(format!("Failed to parse gateway config: {}", e))
    })?;

    // 创建或更新 [mcp_servers] 节
    let mcp_table = doc
        .entry(mcp_servers_section)
        .or_insert(Item::Table(Table::new()));

    // 确保是表
    if let Some(table) = mcp_table.as_table_mut() {
        // 清空现有内容
        table.clear();

        // 复制 gateway 配置
        if let Some(gateway_table) = gateway_doc.as_table().get(mcp_servers_section) {
            if let Some(gt) = gateway_table.as_table() {
                for (key, value) in gt.iter() {
                    table.insert(key, value.clone());
                }
            }
        } else {
            // 如果 gateway_toml 没有 section wrapper，直接作为顶层表处理
            for (key, value) in gateway_doc.as_table().iter() {
                table.insert(key, value.clone());
            }
        }
    }

    Ok(doc.to_string())
}

/// 移除 JSON 注释（支持 JSONC）
///
/// 支持移除 // 单行注释和 /* */ 块注释
pub fn strip_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            result.push(c);
            escape_next = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escape_next = true;
            continue;
        }

        if c == '"' && !escape_next {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if in_string {
            result.push(c);
            continue;
        }

        // 不在字符串中，检查注释
        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // 单行注释，跳过到行末
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        if nc == '\n' {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    // 块注释，跳过到 */
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== JSON 合并测试 =====

    #[test]
    fn test_merge_json_config_empty_original() {
        let gateway = serde_json::json!({
            "mantra-gateway": {
                "url": "http://127.0.0.1:8080/mcp",
                "headers": {
                    "Authorization": "Bearer token123"
                }
            }
        });

        let result = merge_json_config("", "mcpServers", gateway).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(
            parsed["mcpServers"]["mantra-gateway"]["url"],
            "http://127.0.0.1:8080/mcp"
        );
    }

    #[test]
    fn test_merge_json_config_preserve_other_fields() {
        let original = r#"{
            "autoApprove": ["read", "write"],
            "permissions": {"allowedPaths": ["/home"]},
            "mcpServers": {
                "old-server": {"command": "old"}
            }
        }"#;

        let gateway = serde_json::json!({
            "mantra-gateway": {
                "url": "http://127.0.0.1:8080/mcp",
                "headers": {"Authorization": "Bearer token"}
            }
        });

        let result = merge_json_config(original, "mcpServers", gateway).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 验证其他字段保留
        assert_eq!(parsed["autoApprove"], serde_json::json!(["read", "write"]));
        assert!(parsed["permissions"]["allowedPaths"].is_array());

        // 验证 mcpServers 被更新
        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert!(parsed["mcpServers"]["old-server"].is_null()); // 旧服务被移除
    }

    #[test]
    fn test_merge_json_config_with_comments() {
        let original = r#"{
            // This is a comment
            "autoApprove": ["read"],
            /* Block comment */
            "mcpServers": {}
        }"#;

        let gateway = serde_json::json!({
            "mantra-gateway": {"url": "http://localhost/mcp"}
        });

        let result = merge_json_config(original, "mcpServers", gateway).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        assert!(parsed["mcpServers"]["mantra-gateway"].is_object());
        assert_eq!(parsed["autoApprove"], serde_json::json!(["read"]));
    }

    #[test]
    fn test_merge_json_config_invalid_root() {
        let original = "[]"; // 数组而非对象
        let gateway = serde_json::json!({"test": {}});

        let result = merge_json_config(original, "mcpServers", gateway);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::InvalidFormat(_)));
    }

    // ===== TOML 合并测试 =====

    #[test]
    fn test_merge_toml_config_empty_original() {
        let gateway = r#"
[mcp_servers]
mantra-gateway = { url = "http://127.0.0.1:8080/mcp", http_headers = { Authorization = "Bearer token" } }
"#;

        let result = merge_toml_config("", "mcp_servers", gateway).unwrap();
        assert!(result.contains("mantra-gateway"));
        assert!(result.contains("http://127.0.0.1:8080/mcp"));
    }

    #[test]
    fn test_merge_toml_config_preserve_other_sections() {
        let original = r#"
# Configuration file
model = "gpt-4"
temperature = 0.7

# MCP servers
[mcp_servers]
old_server = { command = "old" }

[other_section]
key = "value"
"#;

        let gateway = r#"
[mcp_servers]
mantra-gateway = { url = "http://127.0.0.1:8080/mcp" }
"#;

        let result = merge_toml_config(original, "mcp_servers", gateway).unwrap();

        // 验证其他配置保留
        assert!(result.contains("model = \"gpt-4\""));
        assert!(result.contains("temperature = 0.7"));
        assert!(result.contains("[other_section]"));
        assert!(result.contains("key = \"value\""));

        // 验证 mcp_servers 被更新
        assert!(result.contains("mantra-gateway"));
        assert!(!result.contains("old_server")); // 旧服务被移除
    }

    #[test]
    fn test_merge_toml_config_preserve_comments() {
        let original = r#"
# This is my config file
# Very important settings

model = "gpt-4" # Use GPT-4

# Server configuration below
[mcp_servers]
test = { command = "test" }
"#;

        let gateway = r#"
[mcp_servers]
mantra-gateway = { url = "http://127.0.0.1:8080/mcp" }
"#;

        let result = merge_toml_config(original, "mcp_servers", gateway).unwrap();

        // 验证注释保留
        assert!(result.contains("# This is my config file"));
        assert!(result.contains("# Very important settings"));
        assert!(result.contains("# Use GPT-4"));
    }

    // ===== strip_json_comments 测试 =====

    #[test]
    fn test_strip_single_line_comment() {
        let input = r#"{"key": "value" // comment}"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("// comment"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_block_comment() {
        let input = r#"{"key": /* comment */ "value"}"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("/* comment */"));
        assert!(result.contains("\"key\":"));
        assert!(result.contains("\"value\""));
    }

    #[test]
    fn test_preserve_comment_in_string() {
        let input = r#"{"url": "http://example.com // not a comment"}"#;
        let result = strip_json_comments(input);
        assert!(result.contains("// not a comment"));
    }

    #[test]
    fn test_strip_multiline_block_comment() {
        let input = r#"{
            /*
             * Multi-line
             * block comment
             */
            "key": "value"
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("Multi-line"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_comment_at_end_of_line() {
        let input = r#"{
            "key1": "value1", // first comment
            "key2": "value2" // second comment
        }"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("// first comment"));
        assert!(!result.contains("// second comment"));
        assert!(result.contains("\"key1\": \"value1\""));
        assert!(result.contains("\"key2\": \"value2\""));
    }
}
