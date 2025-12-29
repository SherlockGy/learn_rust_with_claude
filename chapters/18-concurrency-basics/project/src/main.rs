// parallel-hash: 并行计算多个文件的 SHA256 哈希
// 用法: parallel-hash <文件>...
// 示例: parallel-hash *.txt

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("用法: parallel-hash <文件>...");
        eprintln!("示例: parallel-hash *.txt");
        std::process::exit(1);
    }

    // 收集有效文件路径
    let paths: Vec<PathBuf> = args
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .collect();

    if paths.is_empty() {
        eprintln!("没有找到有效文件");
        std::process::exit(1);
    }

    let start = Instant::now();

    // 并行计算哈希
    let results = hash_files_parallel(paths);

    // 输出结果
    for (path, hash) in &results {
        println!("{}  sha256:{}", path.display(), hash);
    }

    let duration = start.elapsed();
    println!(
        "\n完成：{} 个文件，用时 {:.2} 秒",
        results.len(),
        duration.as_secs_f64()
    );
}

/// 并行计算多个文件的哈希值
///
/// 使用 Arc 共享文件列表，每个线程负责一个文件
fn hash_files_parallel(paths: Vec<PathBuf>) -> Vec<(PathBuf, String)> {
    // Arc: Atomic Reference Count，原子引用计数
    // 允许多个线程共享所有权
    let paths = Arc::new(paths);
    let mut handles = Vec::new();

    // 为每个文件创建一个线程
    for i in 0..paths.len() {
        // Arc::clone 只增加引用计数，不复制数据
        let paths = Arc::clone(&paths);

        // thread::spawn 需要 'static 生命周期
        // move 闭包将 paths 和 i 的所有权移入线程
        let handle = thread::spawn(move || {
            let path = &paths[i];
            let hash = hash_file(path);
            (path.clone(), hash)
        });

        handles.push(handle);
    }

    // 收集所有线程的结果
    // join() 等待线程完成并返回结果
    handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .collect()
}

/// 计算单个文件的 SHA256 哈希
fn hash_file(path: &PathBuf) -> String {
    match fs::read(path) {
        Ok(content) => {
            // Sha256::digest 返回 GenericArray
            // format!("{:x}", ...) 将其格式化为十六进制字符串
            let hash = Sha256::digest(&content);
            format!("{:x}", hash)
        }
        Err(e) => {
            format!("ERROR: {}", e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "hello world").unwrap();

        let hash = hash_file(&file.path().to_path_buf());
        // SHA256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_parallel_hash() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();

        write!(file1, "test1").unwrap();
        write!(file2, "test2").unwrap();

        let paths = vec![
            file1.path().to_path_buf(),
            file2.path().to_path_buf(),
        ];

        let results = hash_files_parallel(paths);
        assert_eq!(results.len(), 2);
    }
}
