// kv-server-mt: 多线程键值存储服务器
// 用法: kv-server-mt [--port PORT] [--threads N]
//
// 特性:
// - 线程池处理多个客户端
// - RwLock 实现读写分离
// - 支持并发访问

mod thread_pool;

use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

use thread_pool::ThreadPool;

// Store 类型别名：原子引用计数 + 读写锁 + HashMap
// Arc: 允许多线程共享所有权
// RwLock: 读操作可并发，写操作独占
type Store = Arc<RwLock<HashMap<String, String>>>;

fn main() {
    let (port, thread_count) = parse_args();
    let addr = format!("127.0.0.1:{}", port);

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法绑定到 {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    println!("kv-server (多线程版) 启动");
    println!("监听地址: {}", addr);
    println!("线程池大小: {}", thread_count);
    println!("支持命令: SET key value | GET key | DEL key | KEYS | QUIT\n");

    // 共享存储
    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    // 创建线程池
    let pool = ThreadPool::new(thread_count);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 克隆 Arc，只增加引用计数
                let store = Arc::clone(&store);

                // 提交任务到线程池
                pool.execute(move || {
                    handle_client(stream, store);
                });
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }
}

/// 处理单个客户端连接
fn handle_client(stream: TcpStream, store: Store) {
    let peer = stream.peer_addr().ok();
    println!("[{:?}] 客户端连接", peer);

    // try_clone() 创建独立的写入句柄
    let mut writer = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };

    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let response = execute_command(&line, &store);

        if writer.write_all(response.as_bytes()).is_err() {
            break;
        }

        if line.trim().eq_ignore_ascii_case("QUIT") {
            break;
        }
    }

    println!("[{:?}] 客户端断开", peer);
}

/// 执行命令
fn execute_command(line: &str, store: &Store) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        // SET 需要写锁
        ["SET", key, value] | ["set", key, value] => {
            // write() 获取写锁，阻塞其他所有访问
            let mut store = store.write().unwrap();
            store.insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }

        // GET 只需要读锁
        ["GET", key] | ["get", key] => {
            // read() 获取读锁，允许多个读者并发
            let store = store.read().unwrap();
            match store.get(*key) {
                Some(value) => format!("VALUE {}\n", value),
                None => "NOT_FOUND\n".to_string(),
            }
        }

        // DEL 需要写锁
        ["DEL", key] | ["del", key] => {
            let mut store = store.write().unwrap();
            store.remove(*key);
            "OK\n".to_string()
        }

        // KEYS 只需要读锁
        ["KEYS"] | ["keys"] => {
            let store = store.read().unwrap();
            let keys: Vec<&String> = store.keys().collect();
            if keys.is_empty() {
                "KEYS (empty)\n".to_string()
            } else {
                format!(
                    "KEYS {}\n",
                    keys.iter()
                        .map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }

        ["QUIT"] | ["quit"] => "BYE\n".to_string(),

        _ => "ERROR unknown command\n".to_string(),
    }
}

/// 解析命令行参数
fn parse_args() -> (u16, usize) {
    let args: Vec<String> = env::args().collect();
    let mut port = 7878u16;
    let mut threads = 4usize;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or(7878);
                i += 2;
            }
            "--threads" if i + 1 < args.len() => {
                threads = args[i + 1].parse().unwrap_or(4);
                i += 2;
            }
            _ => i += 1,
        }
    }

    (port, threads)
}
