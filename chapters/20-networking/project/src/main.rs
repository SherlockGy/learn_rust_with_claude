// kv-server: 简单的键值存储服务器（单线程版）
// 用法: kv-server [--port PORT]
//
// 协议:
//   SET key value\n  -> OK\n
//   GET key\n        -> VALUE value\n 或 NOT_FOUND\n
//   DEL key\n        -> OK\n
//   KEYS\n           -> KEYS key1 key2 ...\n
//   QUIT\n           -> 关闭连接

use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let port = parse_port();
    let addr = format!("127.0.0.1:{}", port);

    // TcpListener::bind 绑定到指定地址
    // 返回 Result<TcpListener>
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法绑定到 {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    println!("kv-server 启动，监听 {}", addr);
    println!("支持命令: SET key value | GET key | DEL key | KEYS | QUIT");

    // 存储使用 HashMap
    let mut store: HashMap<String, String> = HashMap::new();

    // listener.incoming() 返回连接迭代器
    // 每次迭代返回 Result<TcpStream>
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer = stream.peer_addr().ok();
                println!("\n客户端连接: {:?}", peer);

                handle_client(stream, &mut store);

                println!("客户端断开: {:?}", peer);
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }
}

/// 处理单个客户端连接
fn handle_client(stream: TcpStream, store: &mut HashMap<String, String>) {
    // try_clone() 创建一个独立的句柄
    // 这样读和写可以使用不同的句柄，避免借用冲突
    let mut writer = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };

    // BufReader 包装原始 stream 用于读取
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        println!("  收到: {}", line);

        // 解析并执行命令
        let response = execute_command(&line, store);

        println!("  响应: {}", response.trim());

        // 使用克隆的句柄发送响应
        if writer.write_all(response.as_bytes()).is_err() {
            break;
        }

        // QUIT 命令关闭连接
        if line.trim().eq_ignore_ascii_case("QUIT") {
            break;
        }
    }
}

/// 执行命令并返回响应
fn execute_command(line: &str, store: &mut HashMap<String, String>) -> String {
    // splitn(3, ' ') 最多分割成 3 部分
    // 这样 value 可以包含空格
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        // SET key value
        ["SET", key, value] | ["set", key, value] => {
            store.insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }

        // GET key
        ["GET", key] | ["get", key] => match store.get(*key) {
            Some(value) => format!("VALUE {}\n", value),
            None => "NOT_FOUND\n".to_string(),
        },

        // DEL key
        ["DEL", key] | ["del", key] => {
            store.remove(*key);
            "OK\n".to_string()
        }

        // KEYS - 列出所有键
        ["KEYS"] | ["keys"] => {
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

        // QUIT
        ["QUIT"] | ["quit"] => "BYE\n".to_string(),

        // 未知命令
        _ => "ERROR unknown command\n".to_string(),
    }
}

/// 解析端口参数
fn parse_port() -> u16 {
    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(port) = args[i + 1].parse() {
                return port;
            }
        }
    }

    7878 // 默认端口
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let mut store = HashMap::new();

        let response = execute_command("SET name Alice", &mut store);
        assert_eq!(response, "OK\n");

        let response = execute_command("GET name", &mut store);
        assert_eq!(response, "VALUE Alice\n");
    }

    #[test]
    fn test_get_not_found() {
        let mut store = HashMap::new();

        let response = execute_command("GET unknown", &mut store);
        assert_eq!(response, "NOT_FOUND\n");
    }

    #[test]
    fn test_del() {
        let mut store = HashMap::new();
        store.insert("key".to_string(), "value".to_string());

        let response = execute_command("DEL key", &mut store);
        assert_eq!(response, "OK\n");

        let response = execute_command("GET key", &mut store);
        assert_eq!(response, "NOT_FOUND\n");
    }

    #[test]
    fn test_value_with_spaces() {
        let mut store = HashMap::new();

        let response = execute_command("SET msg Hello World", &mut store);
        assert_eq!(response, "OK\n");

        let response = execute_command("GET msg", &mut store);
        assert_eq!(response, "VALUE Hello World\n");
    }
}
