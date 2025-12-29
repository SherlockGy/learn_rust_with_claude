// 综合项目：mini-redis
//
// 这是一个项目模板，选择以下方向之一深入实现：
//
// ## 选项 1: mini-redis（推荐）
// 更完整的 Redis 实现：
// - 支持更多命令（LPUSH、LRANGE、HSET、HGET、EXPIRE）
// - 过期时间管理
// - 持久化（AOF 或 RDB）
// - 发布/订阅功能
//
// ## 选项 2: file-sync
// 文件同步工具：
// - 监控文件变化（可使用 notify crate）
// - 增量同步
// - 冲突检测和解决
// - 网络传输（可使用已学的 TCP 知识）
//
// ## 选项 3: log-analyzer
// 日志分析管道：
// - 多源日志聚合
// - 正则表达式解析
// - 统计分析（错误率、延迟分布等）
// - 告警规则引擎

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

// 数据类型：支持字符串和列表
#[derive(Clone)]
enum Value {
    String(String),
    List(Vec<String>),
}

struct Store {
    data: RwLock<HashMap<String, Value>>,
    // TODO: 添加过期时间管理
    // expires: RwLock<HashMap<String, Instant>>,
}

impl Store {
    fn new() -> Self {
        Store {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("mini-redis 启动，监听 {}", addr);
    println!("\n已实现的命令:");
    println!("  SET key value");
    println!("  GET key");
    println!("  DEL key");
    println!("  LPUSH key value [value ...]");
    println!("  LRANGE key start stop");
    println!("\n待实现:");
    println!("  EXPIRE, HSET, HGET, PUBLISH, SUBSCRIBE...\n");

    let store = Arc::new(Store::new());

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let store = Arc::clone(&store);

        tokio::spawn(async move {
            handle_client(socket, store).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream, store: Arc<Store>) {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
            break;
        }

        let response = execute_command(line.trim(), &store).await;

        if writer.write_all(response.as_bytes()).await.is_err() {
            break;
        }
    }
}

async fn execute_command(line: &str, store: &Store) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return "ERROR empty command\n".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "SET" if parts.len() >= 3 => {
            let key = parts[1].to_string();
            let value = parts[2..].join(" ");
            store.data.write().await.insert(key, Value::String(value));
            "+OK\n".to_string()
        }

        "GET" if parts.len() == 2 => {
            let data = store.data.read().await;
            match data.get(parts[1]) {
                Some(Value::String(s)) => format!("${}\n", s),
                Some(Value::List(_)) => "-WRONGTYPE\n".to_string(),
                None => "$-1\n".to_string(),
            }
        }

        "DEL" if parts.len() >= 2 => {
            let mut data = store.data.write().await;
            let mut count = 0;
            for key in &parts[1..] {
                if data.remove(*key).is_some() {
                    count += 1;
                }
            }
            format!(":{}\n", count)
        }

        "LPUSH" if parts.len() >= 3 => {
            let key = parts[1].to_string();
            let values: Vec<String> = parts[2..].iter().map(|s| s.to_string()).collect();

            let mut data = store.data.write().await;
            let list = data
                .entry(key)
                .or_insert_with(|| Value::List(Vec::new()));

            if let Value::List(ref mut vec) = list {
                for v in values.into_iter().rev() {
                    vec.insert(0, v);
                }
                format!(":{}\n", vec.len())
            } else {
                "-WRONGTYPE\n".to_string()
            }
        }

        "LRANGE" if parts.len() == 4 => {
            let key = parts[1];
            let start: i64 = parts[2].parse().unwrap_or(0);
            let stop: i64 = parts[3].parse().unwrap_or(-1);

            let data = store.data.read().await;
            match data.get(key) {
                Some(Value::List(vec)) => {
                    let len = vec.len() as i64;
                    let start = if start < 0 { (len + start).max(0) } else { start.min(len) } as usize;
                    let stop = if stop < 0 { (len + stop).max(0) } else { stop.min(len - 1) } as usize;

                    if start > stop {
                        "*0\n".to_string()
                    } else {
                        let items: Vec<String> = vec[start..=stop]
                            .iter()
                            .map(|s| format!("${}", s))
                            .collect();
                        format!("*{}\n{}\n", items.len(), items.join("\n"))
                    }
                }
                Some(Value::String(_)) => "-WRONGTYPE\n".to_string(),
                None => "*0\n".to_string(),
            }
        }

        "PING" => "+PONG\n".to_string(),

        "QUIT" => "+OK\n".to_string(),

        _ => "-ERROR unknown command\n".to_string(),
    }
}
