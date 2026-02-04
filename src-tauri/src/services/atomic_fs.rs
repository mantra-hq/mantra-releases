//! 原子文件操作模块
//!
//! Story 11.22: 原子性备份恢复机制 - Task 1
//!
//! 提供原子性文件复制和写入操作，确保在操作失败时不会产生损坏的文件。
//! 使用 tempfile + SHA256 hash 验证 + atomic rename 模式。

use std::fs;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// 计算文件的 SHA256 hash
///
/// # Arguments
/// * `path` - 文件路径
///
/// # Returns
/// 十六进制编码的 SHA256 hash 字符串
pub fn calculate_file_hash(path: &Path) -> io::Result<String> {
    let content = fs::read(path)?;
    Ok(calculate_content_hash(&content))
}

/// 计算字节内容的 SHA256 hash
///
/// # Arguments
/// * `content` - 字节内容
///
/// # Returns
/// 十六进制编码的 SHA256 hash 字符串
pub fn calculate_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// 计算字符串内容的 SHA256 hash (Story 11.22 - Task 9)
///
/// 便捷函数，用于直接计算字符串的 hash 而无需先写入文件
///
/// # Arguments
/// * `content` - 字符串内容
///
/// # Returns
/// 十六进制编码的 SHA256 hash 字符串
pub fn calculate_string_hash(content: &str) -> String {
    calculate_content_hash(content.as_bytes())
}

/// 验证文件完整性
///
/// # Arguments
/// * `path` - 文件路径
/// * `expected_hash` - 期望的 SHA256 hash
///
/// # Returns
/// 验证通过返回 Ok(true)，hash 不匹配返回 Ok(false)，IO 错误返回 Err
pub fn verify_file_integrity(path: &Path, expected_hash: &str) -> io::Result<bool> {
    let actual_hash = calculate_file_hash(path)?;
    Ok(actual_hash == expected_hash)
}

/// 原子复制文件
///
/// 使用 tempfile + hash verify + atomic rename 模式：
/// 1. 计算源文件 hash
/// 2. 复制到同目录的临时文件
/// 3. 验证临时文件 hash
/// 4. 原子重命名为目标路径
///
/// # Arguments
/// * `src` - 源文件路径
/// * `dst` - 目标文件路径
///
/// # Returns
/// 源文件的 SHA256 hash
pub fn atomic_copy(src: &Path, dst: &Path) -> io::Result<String> {
    // 1. 计算源文件 hash
    let src_hash = calculate_file_hash(src)?;

    // 2. 确保目标目录存在
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    // 3. 在目标目录创建临时文件
    let dst_parent = dst.parent().unwrap_or(Path::new("."));
    let temp_file = tempfile::NamedTempFile::new_in(dst_parent)?;

    // 4. 复制内容到临时文件
    // persist 后手动管理文件生命周期
    let temp_path = temp_file.into_temp_path();
    fs::copy(src, &temp_path)?;

    // 5. 验证临时文件 hash
    let temp_hash = calculate_file_hash(&temp_path)?;
    if src_hash != temp_hash {
        // 清理临时文件（TempPath drop 会自动清理）
        let _ = fs::remove_file(&temp_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Hash mismatch after copy: expected {}, got {}",
                src_hash, temp_hash
            ),
        ));
    }

    // 6. 原子重命名
    // 注意：rename 在同一文件系统上是原子操作
    if let Err(e) = fs::rename(&temp_path, dst) {
        // rename 可能因为跨文件系统失败，回退到 copy + delete
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(src_hash)
}

/// 原子写入文件
///
/// 使用 tempfile + hash verify + atomic rename 模式：
/// 1. 计算内容 hash
/// 2. 写入同目录的临时文件
/// 3. 验证写入内容的 hash
/// 4. 原子重命名为目标路径
///
/// # Arguments
/// * `path` - 目标文件路径
/// * `content` - 要写入的字节内容
///
/// # Returns
/// 内容的 SHA256 hash
pub fn atomic_write(path: &Path, content: &[u8]) -> io::Result<String> {
    // 1. 计算内容 hash
    let content_hash = calculate_content_hash(content);

    // 2. 确保目标目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 3. 在目标目录创建临时文件
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp_file = tempfile::NamedTempFile::new_in(parent)?;
    let temp_path = temp_file.into_temp_path();

    // 4. 写入内容到临时文件
    fs::write(&temp_path, content)?;

    // 5. 验证写入内容的 hash
    let written_hash = calculate_file_hash(&temp_path)?;
    if content_hash != written_hash {
        let _ = fs::remove_file(&temp_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Hash mismatch after write: expected {}, got {}",
                content_hash, written_hash
            ),
        ));
    }

    // 6. 原子重命名
    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(content_hash)
}

/// 原子写入字符串内容
///
/// 便捷函数，将字符串编码为 UTF-8 后调用 atomic_write
pub fn atomic_write_str(path: &Path, content: &str) -> io::Result<String> {
    atomic_write(path, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    /// 测试计算文件 hash
    #[test]
    fn test_calculate_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 写入已知内容
        let content = b"Hello, World!";
        fs::write(&file_path, content).unwrap();

        // 计算 hash
        let hash = calculate_file_hash(&file_path).unwrap();

        // 验证 hash 格式（64 个十六进制字符）
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // 验证相同内容产生相同 hash
        let content_hash = calculate_content_hash(content);
        assert_eq!(hash, content_hash);
    }

    /// 测试计算内容 hash
    #[test]
    fn test_calculate_content_hash() {
        let content = b"Test content";
        let hash = calculate_content_hash(content);

        // 验证 hash 格式
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // 验证确定性：相同内容相同 hash
        let hash2 = calculate_content_hash(content);
        assert_eq!(hash, hash2);

        // 验证不同内容不同 hash
        let different_hash = calculate_content_hash(b"Different content");
        assert_ne!(hash, different_hash);
    }

    /// 测试空内容的 hash
    #[test]
    fn test_empty_content_hash() {
        let empty_hash = calculate_content_hash(b"");

        // SHA256 of empty string is a known value
        // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            empty_hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// 测试验证文件完整性 - 成功
    #[test]
    fn test_verify_file_integrity_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = b"Test content for integrity";
        fs::write(&file_path, content).unwrap();

        let expected_hash = calculate_content_hash(content);
        let result = verify_file_integrity(&file_path, &expected_hash).unwrap();

        assert!(result);
    }

    /// 测试验证文件完整性 - 失败
    #[test]
    fn test_verify_file_integrity_failure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, b"Actual content").unwrap();

        let wrong_hash = calculate_content_hash(b"Different content");
        let result = verify_file_integrity(&file_path, &wrong_hash).unwrap();

        assert!(!result);
    }

    /// 测试验证不存在的文件
    #[test]
    fn test_verify_file_integrity_not_found() {
        let result = verify_file_integrity(Path::new("/nonexistent/file.txt"), "somehash");
        assert!(result.is_err());
    }

    /// 测试原子复制 - 成功
    #[test]
    fn test_atomic_copy_success() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let dst_path = temp_dir.path().join("dest.txt");

        let content = b"Content to copy atomically";
        fs::write(&src_path, content).unwrap();

        let hash = atomic_copy(&src_path, &dst_path).unwrap();

        // 验证目标文件存在且内容正确
        assert!(dst_path.exists());
        let dst_content = fs::read(&dst_path).unwrap();
        assert_eq!(dst_content, content);

        // 验证返回的 hash 正确
        let expected_hash = calculate_content_hash(content);
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子复制 - 自动创建目标目录
    #[test]
    fn test_atomic_copy_creates_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let dst_path = temp_dir.path().join("subdir/nested/dest.txt");

        let content = b"Content for nested copy";
        fs::write(&src_path, content).unwrap();

        let hash = atomic_copy(&src_path, &dst_path).unwrap();

        // 验证目标文件存在
        assert!(dst_path.exists());

        // 验证 hash
        let expected_hash = calculate_content_hash(content);
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子复制 - 源文件不存在
    #[test]
    fn test_atomic_copy_source_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("nonexistent.txt");
        let dst_path = temp_dir.path().join("dest.txt");

        let result = atomic_copy(&src_path, &dst_path);
        assert!(result.is_err());

        // 确保没有创建目标文件
        assert!(!dst_path.exists());
    }

    /// 测试原子复制 - 覆盖已存在的文件
    #[test]
    fn test_atomic_copy_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("source.txt");
        let dst_path = temp_dir.path().join("dest.txt");

        // 创建源文件
        let new_content = b"New content";
        fs::write(&src_path, new_content).unwrap();

        // 创建已存在的目标文件
        fs::write(&dst_path, b"Old content").unwrap();

        // 原子复制应覆盖
        atomic_copy(&src_path, &dst_path).unwrap();

        let dst_content = fs::read(&dst_path).unwrap();
        assert_eq!(dst_content, new_content);
    }

    /// 测试原子写入 - 成功
    #[test]
    fn test_atomic_write_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = b"Content to write atomically";
        let hash = atomic_write(&file_path, content).unwrap();

        // 验证文件存在且内容正确
        assert!(file_path.exists());
        let written_content = fs::read(&file_path).unwrap();
        assert_eq!(written_content, content);

        // 验证返回的 hash 正确
        let expected_hash = calculate_content_hash(content);
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子写入 - 自动创建目标目录
    #[test]
    fn test_atomic_write_creates_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("deep/nested/dir/test.txt");

        let content = b"Nested write content";
        let hash = atomic_write(&file_path, content).unwrap();

        // 验证文件存在
        assert!(file_path.exists());

        // 验证 hash
        let expected_hash = calculate_content_hash(content);
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子写入 - 覆盖已存在的文件
    #[test]
    fn test_atomic_write_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 创建已存在的文件
        fs::write(&file_path, b"Old content").unwrap();

        // 原子写入新内容
        let new_content = b"New content";
        atomic_write(&file_path, new_content).unwrap();

        let written_content = fs::read(&file_path).unwrap();
        assert_eq!(written_content, new_content);
    }

    /// 测试原子写入字符串
    #[test]
    fn test_atomic_write_str() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = "String content with unicode: 你好世界";
        let hash = atomic_write_str(&file_path, content).unwrap();

        // 验证文件内容
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);

        // 验证 hash
        let expected_hash = calculate_content_hash(content.as_bytes());
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子写入空内容
    #[test]
    fn test_atomic_write_empty() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");

        let hash = atomic_write(&file_path, b"").unwrap();

        // 验证文件存在且为空
        assert!(file_path.exists());
        let content = fs::read(&file_path).unwrap();
        assert!(content.is_empty());

        // 验证 hash 是空内容的 hash
        let expected_hash = calculate_content_hash(b"");
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子写入大文件
    #[test]
    fn test_atomic_write_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        // 创建 1MB 内容
        let content: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();

        let hash = atomic_write(&file_path, &content).unwrap();

        // 验证文件内容
        let written_content = fs::read(&file_path).unwrap();
        assert_eq!(written_content, content);

        // 验证 hash
        let expected_hash = calculate_content_hash(&content);
        assert_eq!(hash, expected_hash);
    }

    /// 测试原子操作不留下临时文件
    #[test]
    fn test_atomic_operations_no_temp_files_left() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 执行原子写入
        atomic_write(&file_path, b"Test content").unwrap();

        // 列出目录中的所有文件
        let entries: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        // 应该只有一个文件（目标文件）
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name().to_string_lossy(), "test.txt");
    }

    /// 测试原子复制保留文件内容完整性
    #[test]
    fn test_atomic_copy_preserves_binary_content() {
        let temp_dir = TempDir::new().unwrap();
        let src_path = temp_dir.path().join("binary.bin");
        let dst_path = temp_dir.path().join("binary_copy.bin");

        // 创建包含所有字节值的二进制内容
        let content: Vec<u8> = (0..=255).collect();
        fs::write(&src_path, &content).unwrap();

        atomic_copy(&src_path, &dst_path).unwrap();

        let copied_content = fs::read(&dst_path).unwrap();
        assert_eq!(copied_content, content);
    }
}
