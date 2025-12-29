# 第 20 章：网络编程基础

## 本章目标

学完本章，你将能够：
- 使用 std::net 进行 TCP 编程
- 理解连接的生命周期和资源管理
- 设计简单的文本协议
- 构建 kv-server 键值存储服务

---

## 前置知识

- 第 8 章：错误处理
- 第 11 章：闭包
- 第 18 章：并发基础（了解线程概念）

---

## Rust 网络编程概述

### 标准库的定位

Rust 标准库 `std::net` 提供了基础的同步网络 API：

| 类型 | 用途 |
|------|------|
| `TcpListener` | TCP 服务端监听 |
| `TcpStream` | TCP 连接流 |
| `UdpSocket` | UDP 套接字 |
| `SocketAddr` | 地址表示 |

**设计哲学**：标准库只提供最基础的能力，高级功能（异步、HTTP）由社区 crate 提供。

### 与 Java 的对比

```java
// Java - ServerSocket
ServerSocket server = new ServerSocket(8080);
Socket client = server.accept();
BufferedReader reader = new BufferedReader(
    new InputStreamReader(client.getInputStream())
);
```

```rust
// Rust - TcpListener
let listener = TcpListener::bind("127.0.0.1:8080")?;
let (stream, addr) = listener.accept()?;
let reader = BufReader::new(&stream);
```

**关键差异**：

| Java | Rust | 说明 |
|------|------|------|
| `try-catch` | `Result<T, E>` | 错误处理方式 |
| GC 管理资源 | RAII + Drop | 连接自动关闭 |
| `InputStream/OutputStream` | `Read/Write` trait | I/O 抽象 |
| `BufferedReader` | `BufReader` | 缓冲读取 |

---

## 项目：kv-server（单线程版）

### 功能概览

一个简单的键值存储服务器，支持通过 TCP 连接进行数据存取：

```bash
# 启动服务器
$ cargo run --bin kv-server
kv-server listening on 127.0.0.1:7878

# 客户端连接（使用 netcat）
$ nc localhost 7878
SET name Alice
OK
GET name
VALUE Alice
DEL name
OK
GET name
NOT_FOUND
```

### 为什么做这个项目？

1. **真实场景**：Redis、Memcached 都是类似的键值存储
2. **协议设计**：学习如何设计简单的文本协议
3. **I/O 操作**：实践网络读写和错误处理
4. **铺垫后续**：为多线程版本和异步版本做准备

---

## 核心概念

### 1. TCP 监听器 TcpListener

`TcpListener` 用于监听指定地址的 TCP 连接：

```rust
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    // bind() 绑定到地址，开始监听
    // 地址格式："IP:端口" 或 SocketAddr
    let listener = TcpListener::bind("127.0.0.1:7878")?;

    println!("服务器启动在 127.0.0.1:7878");

    // incoming() 返回一个迭代器，每次 yield 一个新连接
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("新连接: {}", stream.peer_addr()?);
                handle_client(stream)?;
            }
            Err(e) => {
                eprintln!("连接失败: {}", e);
            }
        }
    }

    Ok(())
}
```

**命名解释**：
- `bind`：绑定到地址（类似 Java 的 `ServerSocket(port)`）
- `incoming`：返回传入连接的迭代器
- `accept`：接受单个连接（incoming 内部调用）

### 2. TCP 流 TcpStream

`TcpStream` 代表一个已建立的 TCP 连接，实现了 `Read` 和 `Write` trait：

```rust
use std::io::{Read, Write};
use std::net::TcpStream;

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    // 获取连接信息
    println!("本地地址: {}", stream.local_addr()?);
    println!("对端地址: {}", stream.peer_addr()?);

    // 读取数据
    let mut buffer = [0u8; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    // 写入响应
    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nHello!")?;

    // stream 在此处自动关闭（Drop trait）
    Ok(())
}
```

### 3. 连接生命周期

```
客户端                           服务器
   |                                |
   |  1. connect() --------------> |  accept()
   |                                |
   |  2. write() ---------------> |  read()
   |                                |
   |  3. read() <---------------- |  write()
   |                                |
   |  4. close() ----------------- |  close()
   |                                |
```

**Rust 的优势**：连接资源通过 RAII 自动管理

```rust
fn handle_client(stream: TcpStream) {
    // stream 在函数结束时自动关闭
    // 无需 try-finally 或 try-with-resources
}
```

### 4. 缓冲读写

直接使用 `read`/`write` 效率低，每次调用都是系统调用。使用缓冲可以显著提升性能：

```rust
use std::io::{BufRead, BufReader, BufWriter, Write};

fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    // BufReader: 缓冲读取器，提供 read_line、lines 等便捷方法
    let reader = BufReader::new(&stream);

    // BufWriter: 缓冲写入器，减少系统调用次数
    let mut writer = BufWriter::new(&stream);

    for line in reader.lines() {
        let line = line?;
        writeln!(writer, "收到: {}", line)?;
        writer.flush()?;  // 确保数据发送出去
    }

    Ok(())
}
```

**命名解释**：
- `BufReader`：Buffered Reader，带缓冲的读取器
- `BufWriter`：Buffered Writer，带缓冲的写入器
- `lines()`：按行迭代（每行不含 `\n`）
- `flush()`：刷新缓冲区，确保数据写出

### 5. 借用 TcpStream

`TcpStream` 可以同时用于读和写，但 Rust 的借用规则需要特别处理：

```rust
// 方法 1：使用引用（推荐）
fn handle_v1(stream: TcpStream) -> std::io::Result<()> {
    let reader = BufReader::new(&stream);  // 不可变借用
    let mut writer = BufWriter::new(&stream);  // 不可变借用
    // 两个借用可以共存，因为 TcpStream 内部是线程安全的
    Ok(())
}

// 方法 2：使用 try_clone()
fn handle_v2(stream: TcpStream) -> std::io::Result<()> {
    let read_stream = stream.try_clone()?;  // 复制文件描述符
    let write_stream = stream;

    let reader = BufReader::new(read_stream);
    let mut writer = BufWriter::new(write_stream);
    Ok(())
}
```

---

## 协议设计

### 为什么需要协议？

TCP 是字节流协议，没有消息边界。我们需要定义：
- 消息如何开始和结束
- 命令格式是什么
- 响应格式是什么

### kv-server 协议

简单的文本协议，每行一个命令：

```
请求格式：
  SET key value\n   -> 设置键值
  GET key\n         -> 获取值
  DEL key\n         -> 删除键
  KEYS\n            -> 列出所有键

响应格式：
  OK\n              -> 操作成功
  VALUE value\n     -> 返回值
  NOT_FOUND\n       -> 键不存在
  ERROR message\n   -> 错误信息
```

**设计选择**：
- 使用 `\n` 作为消息分隔符（便于测试和调试）
- 命令全大写（便于解析）
- 响应有固定前缀（便于客户端解析）

### 协议解析

```rust
/// 解析命令
fn parse_command(line: &str) -> Command {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["SET", key, value] => Command::Set {
            key: key.to_string(),
            value: value.to_string(),
        },
        ["GET", key] => Command::Get {
            key: key.to_string(),
        },
        ["DEL", key] => Command::Del {
            key: key.to_string(),
        },
        ["KEYS"] => Command::Keys,
        _ => Command::Unknown(line.to_string()),
    }
}

enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Del { key: String },
    Keys,
    Unknown(String),
}
```

---

## kv-server 完整实现

```rust
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const ADDR: &str = "127.0.0.1:7878";

fn main() {
    // 启动服务器
    let listener = match TcpListener::bind(ADDR) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("无法绑定到 {}: {}", ADDR, e);
            return;
        }
    };

    println!("kv-server listening on {}", ADDR);

    // 存储（单线程版本，直接使用 HashMap）
    let mut store = HashMap::new();

    // 接受连接
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().map(|a| a.to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                println!("客户端连接: {}", addr);

                if let Err(e) = handle_client(stream, &mut store) {
                    eprintln!("处理客户端出错: {}", e);
                }

                println!("客户端断开: {}", addr);
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }
}

fn handle_client(
    stream: TcpStream,
    store: &mut HashMap<String, String>,
) -> std::io::Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let line = line?;

        if line.is_empty() {
            continue;
        }

        let response = process_command(&line, store);
        writer.write_all(response.as_bytes())?;
        writer.flush()?;
    }

    Ok(())
}

fn process_command(line: &str, store: &mut HashMap<String, String>) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["SET", key, value] => {
            store.insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }
        ["GET", key] => {
            match store.get(*key) {
                Some(value) => format!("VALUE {}\n", value),
                None => "NOT_FOUND\n".to_string(),
            }
        }
        ["DEL", key] => {
            store.remove(*key);
            "OK\n".to_string()
        }
        ["KEYS"] => {
            if store.is_empty() {
                "EMPTY\n".to_string()
            } else {
                let keys: Vec<&str> = store.keys().map(|s| s.as_str()).collect();
                format!("KEYS {}\n", keys.join(" "))
            }
        }
        _ => format!("ERROR unknown command: {}\n", line),
    }
}
```

---

## 错误处理模式

### 连接级错误 vs 请求级错误

```rust
fn handle_client(stream: TcpStream, store: &mut Store) -> std::io::Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        // 连接级错误：读取失败，退出循环
        let line = line?;

        // 请求级错误：命令无效，返回错误响应，继续处理
        let response = match parse_and_execute(&line, store) {
            Ok(resp) => resp,
            Err(e) => format!("ERROR {}\n", e),
        };

        // 连接级错误：写入失败，退出循环
        writer.write_all(response.as_bytes())?;
    }

    Ok(())
}
```

### 优雅处理断开连接

```rust
for line in reader.lines() {
    let line = match line {
        Ok(line) => line,
        Err(e) => {
            // 判断是正常断开还是真的错误
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                // 客户端正常关闭连接
                break;
            }
            return Err(e);
        }
    };

    // 处理命令...
}
```

---

## 单线程模型的局限

当前的单线程模型有明显问题：

```
客户端 A 连接 -----> 服务器处理 A
客户端 B 连接 -----> 等待...（被阻塞）
客户端 A 断开 -----> 服务器处理 B
```

**问题**：
1. 一次只能处理一个客户端
2. 慢客户端会阻塞其他客户端
3. 无法利用多核 CPU

**解决方案**（下一章）：
- 为每个连接创建新线程
- 使用线程池限制线程数量
- 共享状态需要同步原语保护

---

## 测试网络代码

### 使用 netcat 测试

```bash
# 连接服务器
nc localhost 7878

# 发送命令
SET foo bar
GET foo
DEL foo
```

### 使用 telnet 测试

```bash
telnet localhost 7878
```

### 编写测试客户端

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    let mut reader = BufReader::new(stream.try_clone()?);

    // 发送命令
    writeln!(stream, "SET test hello")?;
    stream.flush()?;

    // 读取响应
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("响应: {}", response.trim());

    Ok(())
}
```

---

## 最佳实践

### 服务器端

| 场景 | 推荐做法 |
|------|---------|
| 绑定地址 | 开发用 `127.0.0.1`，生产用 `0.0.0.0` |
| 缓冲读取 | 始终使用 `BufReader` |
| 错误处理 | 区分连接级和请求级错误 |
| 资源管理 | 依赖 RAII，不手动 close |
| 日志 | 记录连接/断开/错误事件 |

### 协议设计

| 场景 | 推荐做法 |
|------|---------|
| 消息边界 | 使用明确的分隔符（`\n`、长度前缀） |
| 编码 | 文本协议用 UTF-8 |
| 错误响应 | 包含错误类型和描述 |
| 版本兼容 | 考虑协议版本号 |

### 常见陷阱

| 陷阱 | 说明 | 解决方案 |
|------|------|---------|
| 忘记 flush | 数据在缓冲区没发出去 | 每次响应后 flush |
| TCP 粘包 | TCP 是流协议，没有消息边界 | 使用分隔符或长度前缀 |
| 阻塞读取 | read 会阻塞直到有数据 | 设置超时或使用非阻塞 I/O |
| 地址已占用 | bind 失败 | 检查端口是否被占用 |

---

## 要点回顾

1. **TcpListener::bind()** 创建监听器
2. **incoming()** 返回连接迭代器
3. **BufReader/BufWriter** 提供缓冲读写
4. **协议设计**需要明确消息边界
5. **单线程模型**一次只能处理一个连接
6. **RAII** 自动管理连接资源

---

## 练习

### 练习 1：KEYS 命令增强

让 KEYS 命令支持模式匹配：
- `KEYS *` - 返回所有键
- `KEYS user:*` - 返回以 `user:` 开头的键

### 练习 2：Echo 服务器

实现一个简单的 echo 服务器，返回客户端发送的每一行（加上前缀 "ECHO: "）。

### 练习 3：EXPIRE 命令

添加键过期功能：
- `SETEX key seconds value` - 设置带过期时间的键
- 过期后 GET 返回 NOT_FOUND

提示：可以存储 `(value, Option<Instant>)` 元组。

### 练习 4：连接超时

为服务器添加连接超时：如果客户端 30 秒没有发送命令，自动断开连接。

提示：使用 `TcpStream::set_read_timeout()`。

---

## 扩展阅读

- [The Rust Book - 构建多线程 Web 服务器](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html)
- [std::net 模块文档](https://doc.rust-lang.org/std/net/)
- [BufReader 和 BufWriter](https://doc.rust-lang.org/std/io/#bufreader-and-bufwriter)
- [TCP 粘包问题](https://en.wikipedia.org/wiki/Nagle%27s_algorithm)

---

## 下一章预告

单线程服务器一次只能处理一个客户端，这在实际应用中是不可接受的。下一章学习如何用多线程处理并发连接，实现真正可用的网络服务。
