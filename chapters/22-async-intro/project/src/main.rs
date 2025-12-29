// async-kv: 异步键值存储服务器
// 使用 Tokio 运行时
//
// 特性:
// - 异步 I/O，少量线程处理大量连接
// - tokio::spawn 并发处理请求
// - 使用 tokio::sync::RwLock 代替 std::sync::RwLock

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

// 异步版本的 Store
// 注意：tokio::sync::RwLock 而不是 std::sync::RwLock
// tokio 的锁是异步感知的，可以跨 await 点持有
type Store = Arc<RwLock<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:7878";

    // TcpListener::bind 是异步的，返回 Future
    // .await 等待 Future 完成
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("async-kv 启动，监听 {}", addr);
    println!("使用 Tokio 异步运行时\n");

    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    loop {
        // accept() 异步等待新连接
        let (socket, peer) = listener.accept().await.unwrap();

        println!("[{:?}] 客户端连接", peer);

        // 克隆共享状态
        let store = Arc::clone(&store);

        // tokio::spawn 创建异步任务
        // 类似 thread::spawn，但是是轻量级的绿色线程
        tokio::spawn(async move {
            handle_client(socket, store).await;
            println!("[{:?}] 客户端断开", peer);
        });
    }
}

/// 处理单个客户端（异步版本）
async fn handle_client(mut socket: TcpStream, store: Store) {
    // split 将 socket 分成读写两半
    let (reader, mut writer) = socket.split();

    // 使用异步 BufReader
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        // read_line 是异步的
        let bytes_read = reader.read_line(&mut line).await.unwrap_or(0);

        if bytes_read == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let response = execute_command(line, &store).await;

        // write_all 也是异步的
        if writer.write_all(response.as_bytes()).await.is_err() {
            break;
        }

        if line.eq_ignore_ascii_case("QUIT") {
            break;
        }
    }
}

/// 执行命令（异步版本）
async fn execute_command(line: &str, store: &Store) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["SET", key, value] | ["set", key, value] => {
            // .await 获取写锁
            let mut store = store.write().await;
            store.insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }

        ["GET", key] | ["get", key] => {
            // .await 获取读锁
            let store = store.read().await;
            match store.get(*key) {
                Some(value) => format!("VALUE {}\n", value),
                None => "NOT_FOUND\n".to_string(),
            }
        }

        ["DEL", key] | ["del", key] => {
            let mut store = store.write().await;
            store.remove(*key);
            "OK\n".to_string()
        }

        ["KEYS"] | ["keys"] => {
            let store = store.read().await;
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
