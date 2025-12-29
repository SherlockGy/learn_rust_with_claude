// find-rs: 简化版 find 命令
// 用法: find-rs <目录> -name <模式>

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args[2] != "-name" {
        eprintln!("用法: find-rs <目录> -name <模式>");
        eprintln!("示例: find-rs . -name *.rs");
        std::process::exit(1);
    }

    let dir = &args[1];
    let pattern = &args[3];

    find_files(Path::new(dir), pattern);
}

/// 递归查找匹配模式的文件
///
/// # 参数
/// - dir: 起始目录
/// - pattern: 文件名模式（支持 * 通配符）
fn find_files(dir: &Path, pattern: &str) {
    // read_dir 返回 Result<ReadDir>
    // ReadDir 是一个迭代器，产出 Result<DirEntry>
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            // 某些目录可能没有权限访问，静默跳过
            if e.kind() != std::io::ErrorKind::PermissionDenied {
                eprintln!("无法读取目录 {}: {}", dir.display(), e);
            }
            return;
        }
    };

    for entry in entries {
        // 每个 entry 也是 Result
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.is_dir() {
            // 递归进入子目录
            find_files(&path, pattern);
        } else {
            // 检查文件名是否匹配
            if matches_pattern(&path, pattern) {
                println!("{}", path.display());
            }
        }
    }
}

/// 检查路径的文件名是否匹配模式
///
/// 支持简单的通配符匹配：
/// - *.rs 匹配所有 .rs 文件
/// - test* 匹配所有以 test 开头的文件
fn matches_pattern(path: &Path, pattern: &str) -> bool {
    // file_name() 返回 Option<&OsStr>
    // to_str() 将 OsStr 转换为 &str（可能失败，如非 UTF-8 文件名）
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // 简单的通配符匹配实现
    if pattern.starts_with('*') {
        // *.rs -> 匹配以 .rs 结尾
        let suffix = &pattern[1..];
        filename.ends_with(suffix)
    } else if pattern.ends_with('*') {
        // test* -> 匹配以 test 开头
        let prefix = &pattern[..pattern.len() - 1];
        filename.starts_with(prefix)
    } else if pattern.contains('*') {
        // a*b -> 匹配以 a 开头且以 b 结尾
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            filename.starts_with(parts[0]) && filename.ends_with(parts[1])
        } else {
            filename == pattern
        }
    } else {
        // 精确匹配
        filename == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_suffix_pattern() {
        assert!(matches_pattern(Path::new("main.rs"), "*.rs"));
        assert!(matches_pattern(Path::new("lib.rs"), "*.rs"));
        assert!(!matches_pattern(Path::new("main.txt"), "*.rs"));
    }

    #[test]
    fn test_prefix_pattern() {
        assert!(matches_pattern(Path::new("test_main.rs"), "test*"));
        assert!(!matches_pattern(Path::new("main_test.rs"), "test*"));
    }

    #[test]
    fn test_exact_pattern() {
        assert!(matches_pattern(Path::new("Cargo.toml"), "Cargo.toml"));
        assert!(!matches_pattern(Path::new("Cargo.lock"), "Cargo.toml"));
    }
}
