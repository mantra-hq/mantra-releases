// 临时测试文件 - 用于调试解析问题
#[cfg(test)]
mod debug_tests {
    use crate::parsers::ClaudeParser;
    use crate::parsers::LogParser;

    #[test]
    fn test_parse_problematic_file() {
        let content = std::fs::read_to_string(
            "/home/decker/.claude/projects/-mnt-disk0-project-newx-nextalk-voice-capsule/4fe9325e-4c69-4633-ac6f-d879ca16d6c5.jsonl"
        ).expect("Failed to read file");

        println!("File content length: {} bytes", content.len());
        println!("Number of lines: {}", content.lines().count());

        let parser = ClaudeParser::new();
        let result = parser.parse_string(&content);

        match result {
            Ok(session) => {
                println!("\n=== 解析成功 ===");
                println!("Session ID: {}", session.id);
                println!("Messages count: {}", session.messages.len());

                for (i, msg) in session.messages.iter().enumerate() {
                    println!("\nMessage {}: role={:?}", i + 1, msg.role);
                    for (j, block) in msg.content_blocks.iter().enumerate() {
                        let block_type = match block {
                            crate::models::ContentBlock::Text { .. } => "text",
                            crate::models::ContentBlock::Thinking { .. } => "thinking",
                            crate::models::ContentBlock::ToolUse { .. } => "tool_use",
                            crate::models::ContentBlock::ToolResult { .. } => "tool_result",
                            _ => "other",
                        };
                        println!("  Block {}: {}", j + 1, block_type);
                    }
                }
            }
            Err(e) => {
                println!("解析失败: {:?}", e);
            }
        }
    }
}
