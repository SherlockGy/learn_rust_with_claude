// log-watcher: 多文件日志监控工具
// 用法: log-watcher <文件>... --pattern <匹配模式>
// 示例: log-watcher app.log web.log --pattern ERROR

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

/// 日志条目
struct LogEntry {
    /// 来源文件
    file: String,
    /// 日志内容
    line: String,
    /// 行号
    line_num: usize,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // 解析参数
    let (files, pattern) = match parse_args(&args) {
        Some(parsed) => parsed,
        None => {
            eprintln!("用法: log-watcher <文件>... --pattern <匹配模式>");
            eprintln!("示例: log-watcher app.log web.log --pattern ERROR");
            std::process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("没有指定要监控的文件");
        std::process::exit(1);
    }

    println!("开始监控 {} 个文件，匹配模式: \"{}\"", files.len(), pattern);
    println!("按 Ctrl+C 停止\n");

    // 创建通道
    // mpsc: Multiple Producer, Single Consumer
    // tx: transmitter (发送端), rx: receiver (接收端)
    let (tx, rx) = mpsc::channel::<LogEntry>();

    // 为每个文件创建监控线程
    for file in files {
        // clone() 创建发送端的副本
        // 每个生产者线程拥有自己的发送端
        let tx = tx.clone();
        let pattern = pattern.clone();

        thread::spawn(move || {
            watch_file(&file, &pattern, tx);
        });
    }

    // 重要：关闭原始发送端
    // 当所有发送端（包括克隆的）都关闭时，接收端的迭代才会结束
    drop(tx);

    // 统计匹配数
    let mut match_count = 0;

    // 接收并打印匹配的日志
    // rx 实现了 IntoIterator，可以直接 for 循环
    // 当所有发送端关闭时，迭代自动结束
    for entry in rx {
        println!(
            "[{} L{}] {}",
            entry.file, entry.line_num, entry.line
        );
        match_count += 1;
    }

    println!("\n监控结束，共匹配 {} 条", match_count);
}

/// 监控单个文件
fn watch_file(path: &str, pattern: &str, tx: mpsc::Sender<LogEntry>) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {}: {}", path, e);
            return;
        }
    };

    let reader = BufReader::new(file);

    for (line_num, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        // 检查是否匹配模式
        if line.contains(pattern) {
            let entry = LogEntry {
                file: path.to_string(),
                line,
                line_num: line_num + 1,
            };

            // send 可能失败（如果接收端已关闭）
            // 使用 ok() 忽略错误
            if tx.send(entry).is_err() {
                break;
            }
        }
    }
}

/// 解析命令行参数
fn parse_args(args: &[String]) -> Option<(Vec<String>, String)> {
    let mut files = Vec::new();
    let mut pattern = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--pattern" && i + 1 < args.len() {
            pattern = Some(args[i + 1].clone());
            i += 2;
        } else {
            files.push(args[i].clone());
            i += 1;
        }
    }

    Some((files, pattern?))
}
