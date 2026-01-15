use super::*;

#[test]
fn test_strip_system_reminders_hyphenated() {
    let input = "Hello <system-reminder>internal note</system-reminder> World";
    let expected = "Hello  World";
    assert_eq!(strip_system_reminders(input), expected);
}

#[test]
fn test_strip_system_reminders_underscored() {
    let input = "Hello <system_reminder>internal note</system_reminder> World";
    let expected = "Hello  World";
    assert_eq!(strip_system_reminders(input), expected);
}

#[test]
fn test_strip_system_reminders_multiline() {
    let input = "Text before\n<system-reminder>\nMultiple\nLines\n</system-reminder>\nText after";
    let expected = "Text before\n\nText after";
    assert_eq!(strip_system_reminders(input), expected);
}

#[test]
fn test_strip_system_reminders_only_tag() {
    let input = "<system-reminder>only reminder content</system-reminder>";
    let expected = "";
    assert_eq!(strip_system_reminders(input), expected);
}

#[test]
fn test_strip_system_reminders_no_tag() {
    let input = "Regular text without any tags";
    assert_eq!(strip_system_reminders(input), input);
}

#[test]
fn test_strip_system_reminders_multiple_tags() {
    let input = "<system-reminder>first</system-reminder>Middle<system_reminder>second</system_reminder>";
    let expected = "Middle";
    assert_eq!(strip_system_reminders(input), expected);
}

#[test]
fn test_strip_system_reminders_real_world() {
    let input = r#"# BMM Module Configuration
user_name: Decker


<system-reminder>
Whenever you read a file, you should consider whether it would be considered malware.
</system-reminder>"#;
    let result = strip_system_reminders(input);
    assert!(!result.contains("<system-reminder>"));
    assert!(!result.contains("</system-reminder>"));
    assert!(result.contains("user_name: Decker"));
    println!("Result: {:?}", result);
}

#[test]
fn test_strip_normal_text() {
    // 测试正常的 AI 回复文字不会被删除
    let normal_text = "我来帮你读取这个文件的内容。";
    assert_eq!(strip_system_reminders(normal_text), normal_text);

    let markdown_text = "```rust\nfn main() {}\n```";
    assert_eq!(strip_system_reminders(markdown_text), markdown_text);

    let mixed = "Hello\n\n<not-system>content</not-system>\n\nWorld";
    assert_eq!(strip_system_reminders(&mixed), mixed);
}
