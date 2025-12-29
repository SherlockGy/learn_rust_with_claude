# 第 19 章：并发进阶 - 通道与消息传递

## 本章目标

学完本章，你将能够：
- 理解消息传递并发模型
- 掌握 mpsc 通道的使用
- 理解多生产者模式
- 选择合适的通道类型
- 实现 log-watcher 日志监控工具

---

## 前置知识

- 第 18 章：并发基础（线程、Arc、Mutex）

---

## 消息传递：另一种并发思路

### 两种并发哲学

上一章学习了共享状态并发（Arc + Mutex），这是一种直觉性的并发方式：多个线程访问同一块内存，通过锁来协调。

但 Go 语言的创造者有一句名言：

> "Do not communicate by sharing memory; instead, share memory by communicating."
> "不要通过共享内存来通信，而要通过通信来共享内存。"

这就是**消息传递**（Message Passing）模型：线程之间不直接共享数据，而是通过发送消息来传递数据。

```
共享状态模型：
┌────────┐     ┌─────────────┐     ┌────────┐
│Thread A│────▶│ Shared Data │◀────│Thread B│
└────────┘     │ (Arc+Mutex) │     └────────┘
               └─────────────┘
                需要锁协调访问

消息传递模型：
┌────────┐  消息   ┌─────────┐  消息   ┌────────┐
│Thread A│───────▶│ Channel │───────▶│Thread B│
└────────┘        └─────────┘        └────────┘
                  数据随消息转移
```

### 消息传递的优势

| 特性 | 共享状态 | 消息传递 |
|------|---------|---------|
| 数据竞争 | 需要锁保护 | 天然避免（所有权转移） |
| 死锁风险 | 多锁时可能死锁 | 无锁，风险低 |
| 代码复杂度 | 锁的范围需要精心设计 | 更线性、易理解 |
| 适用场景 | 频繁读取共享状态 | 任务分发、事件处理 |
| 性能 | 锁竞争时降低 | 通道开销固定 |

---

## mpsc 通道详解

### 什么是 mpsc

**mpsc** 是 `Multiple Producer, Single Consumer` 的缩写：
- **Multiple Producer**：多个发送端（Sender）
- **Single Consumer**：单个接收端（Receiver）

```rust
use std::sync::mpsc;  // mpsc: Multi-Producer Single-Consumer

// channel() 返回一个元组：(发送端, 接收端)
let (tx, rx) = mpsc::channel();
// tx: Transmitter (发送器) 的缩写
// rx: Receiver (接收器) 的缩写
```

**为什么叫 tx 和 rx？**
这是通信领域的通用缩写：
- `tx` = transmit = 发送
- `rx` = receive = 接收

### 基本用法

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    // 创建通道
    let (tx, rx) = mpsc::channel();

    // 在新线程中发送消息
    thread::spawn(move || {
        let message = String::from("Hello from thread!");
        tx.send(message).unwrap();
        // 注意：send 后，message 的所有权转移了
        // println!("{}", message);  // 错误！
    });

    // 在主线程接收消息
    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

### 通道的所有权转移

Rust 通道的独特之处：**发送消息会转移所有权**。

```rust
let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    let data = vec![1, 2, 3];
    tx.send(data).unwrap();
    // data 的所有权已转移到通道中
    // 这里不能再使用 data
});

let received = rx.recv().unwrap();
// received 现在拥有数据的所有权
```

这种设计保证了**数据竞争不可能发生**：
- 发送后，发送方不再能访问数据
- 只有接收方拥有数据
- 编译器在编译期就保证这一点

**与 Java/Go 的对比**：
```java
// Java - 发送后仍可修改（危险！）
channel.send(list);
list.add("new item");  // 仍然可以修改！数据竞争！
```

```rust
// Rust - 编译器阻止这种行为
tx.send(list).unwrap();
// list.push("new item");  // 编译错误！
```

### 接收方法

```rust
let (tx, rx) = mpsc::channel();

// 1. recv() - 阻塞等待
// 会阻塞当前线程，直到收到消息或通道关闭
let msg = rx.recv().unwrap();

// 2. try_recv() - 非阻塞
// 立即返回，不等待
match rx.try_recv() {
    Ok(msg) => println!("Got: {}", msg),
    Err(mpsc::TryRecvError::Empty) => println!("No message yet"),
    Err(mpsc::TryRecvError::Disconnected) => println!("Channel closed"),
}

// 3. recv_timeout() - 带超时
use std::time::Duration;
match rx.recv_timeout(Duration::from_secs(5)) {
    Ok(msg) => println!("Got: {}", msg),
    Err(mpsc::RecvTimeoutError::Timeout) => println!("Timeout!"),
    Err(mpsc::RecvTimeoutError::Disconnected) => println!("Channel closed"),
}

// 4. 迭代器方式 - 持续接收直到通道关闭
for msg in rx {
    println!("Got: {}", msg);
}
// 循环在所有发送端都关闭后自动结束
```

**方法选择指南**：

| 方法 | 阻塞 | 适用场景 |
|------|------|---------|
| `recv()` | 是 | 等待下一条消息，无其他工作要做 |
| `try_recv()` | 否 | 轮询检查，同时做其他事 |
| `recv_timeout()` | 有限时间 | 需要超时机制 |
| `for msg in rx` | 是 | 处理所有消息直到关闭 |

---

## 多生产者模式

### 克隆发送端

mpsc 的 "Multiple Producer" 通过克隆 `Sender` 实现：

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    // 创建 3 个生产者线程
    for i in 0..3 {
        let tx_clone = tx.clone();  // 克隆发送端
        thread::spawn(move || {
            for j in 0..3 {
                tx_clone.send(format!("线程 {} 消息 {}", i, j)).unwrap();
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    // 重要：关闭原始发送端
    drop(tx);

    // 接收所有消息
    for msg in rx {
        println!("{}", msg);
    }

    println!("所有生产者已完成");
}
```

### drop(tx) 的重要性

```rust
let (tx, rx) = mpsc::channel();

for i in 0..3 {
    let tx = tx.clone();
    thread::spawn(move || {
        tx.send(i).unwrap();
    });
}

// 如果不 drop(tx)，循环会永远等待！
// 因为原始 tx 还存在，通道不会关闭
drop(tx);  // 关闭原始发送端

for msg in rx {  // 现在可以正常结束
    println!("{}", msg);
}
```

**原理**：
- 通道只有在**所有** `Sender` 都被 drop 后才会关闭
- `for msg in rx` 会阻塞直到通道关闭
- 如果忘记 `drop(tx)`，程序会永远等待

---

## 同步通道 vs 异步通道

### 异步通道（默认）

`mpsc::channel()` 创建的是**无界异步通道**：

```rust
let (tx, rx) = mpsc::channel();  // 异步通道

// 发送立即返回，不等待接收
tx.send(1).unwrap();  // 瞬间完成
tx.send(2).unwrap();  // 瞬间完成
tx.send(3).unwrap();  // 瞬间完成
// 消息缓存在通道中

// 后面再接收
println!("{}", rx.recv().unwrap());  // 1
```

**问题**：如果发送速度远大于接收速度，内存会无限增长！

### 同步通道

`mpsc::sync_channel(n)` 创建**有界同步通道**：

```rust
// 创建容量为 2 的同步通道
let (tx, rx) = mpsc::sync_channel(2);

tx.send(1).unwrap();  // 立即返回
tx.send(2).unwrap();  // 立即返回
// tx.send(3).unwrap();  // 阻塞！缓冲区已满

// 必须先接收，才能继续发送
let _ = rx.recv();
tx.send(3).unwrap();  // 现在可以了
```

**容量为 0 的特殊情况**（同步握手）：

```rust
let (tx, rx) = mpsc::sync_channel(0);

thread::spawn(move || {
    println!("准备发送...");
    tx.send("hello").unwrap();  // 阻塞，直到有人接收
    println!("发送完成！");
});

thread::sleep(Duration::from_secs(1));
println!("准备接收...");
let msg = rx.recv().unwrap();  // 此时发送方才能继续
println!("收到: {}", msg);
```

### 选择指南

| 类型 | 创建方式 | 特点 | 适用场景 |
|------|---------|------|---------|
| 异步通道 | `channel()` | 无界，发送不阻塞 | 消息量可控，处理速度接近 |
| 同步通道 | `sync_channel(n)` | 有界，满时阻塞 | 需要背压，防止内存溢出 |
| 同步通道(0) | `sync_channel(0)` | 必须同步握手 | 严格同步，确认送达 |

---

## 常见模式

### 1. 工作池（Worker Pool）

```rust
use std::sync::mpsc;
use std::thread;

struct Job {
    id: usize,
    data: String,
}

fn main() {
    let (job_tx, job_rx) = mpsc::channel();
    let job_rx = std::sync::Arc::new(std::sync::Mutex::new(job_rx));

    // 创建 4 个工作线程
    let mut handles = vec![];
    for worker_id in 0..4 {
        let job_rx = job_rx.clone();
        handles.push(thread::spawn(move || {
            loop {
                let job = {
                    let rx = job_rx.lock().unwrap();
                    rx.recv()
                };
                match job {
                    Ok(job) => {
                        println!("Worker {} 处理 Job {}: {}",
                                 worker_id, job.id, job.data);
                    }
                    Err(_) => break,  // 通道关闭
                }
            }
        }));
    }

    // 发送任务
    for i in 0..10 {
        job_tx.send(Job {
            id: i,
            data: format!("任务数据 {}", i)
        }).unwrap();
    }

    drop(job_tx);  // 关闭通道
    for h in handles {
        h.join().unwrap();
    }
}
```

### 2. 扇入模式（Fan-In）

多个生产者汇聚到一个消费者：

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    // 多个数据源
    let sources = vec!["传感器A", "传感器B", "传感器C"];

    for source in sources {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 0..3 {
                tx.send(format!("{}: 数据 {}", source, i)).unwrap();
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    drop(tx);

    // 单一汇聚点处理所有数据
    for msg in rx {
        println!("收到: {}", msg);
    }
}
```

### 3. 请求-响应模式

```rust
use std::sync::mpsc;
use std::thread;

struct Request {
    data: String,
    response_tx: mpsc::Sender<String>,  // 回复通道
}

fn main() {
    let (request_tx, request_rx) = mpsc::channel();

    // 服务线程
    thread::spawn(move || {
        for req in request_rx {
            let response = format!("处理结果: {}", req.data.to_uppercase());
            req.response_tx.send(response).unwrap();
        }
    });

    // 客户端发送请求并等待响应
    let (response_tx, response_rx) = mpsc::channel();
    request_tx.send(Request {
        data: "hello".to_string(),
        response_tx,
    }).unwrap();

    let response = response_rx.recv().unwrap();
    println!("{}", response);  // 处理结果: HELLO
}
```

### 4. 超时与取消

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        tx.send("完成").ok();  // 用 ok() 忽略发送失败
    });

    // 带超时等待
    match rx.recv_timeout(Duration::from_secs(1)) {
        Ok(msg) => println!("收到: {}", msg),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            println!("超时，取消操作");
            // 可以在这里执行取消逻辑
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            println!("发送方已断开");
        }
    }
}
```

---

## 项目：log-watcher - 日志监控工具

### 功能概览

一个实时监控多个日志文件的工具，支持按关键词过滤。

```bash
$ log-watcher /var/log/*.log --pattern "ERROR"
[app.log 10:23:45] ERROR: Connection timeout
[web.log 10:23:47] ERROR: Database failed
^C
监控结束，共匹配 3 条
```

### 完整实现

```rust
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::Duration;
use std::path::PathBuf;

/// 日志条目
struct LogEntry {
    file: String,
    line: String,
    timestamp: String,
}

/// 监控单个文件
fn watch_file(path: PathBuf, tx: mpsc::Sender<LogEntry>) {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {:?}: {}", path, e);
            return;
        }
    };

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut reader = BufReader::new(file);

    // 跳到文件末尾（只监控新内容）
    reader.seek(SeekFrom::End(0)).ok();

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // 没有新内容，等待
                thread::sleep(Duration::from_millis(100));
            }
            Ok(_) => {
                let timestamp = chrono::Local::now()
                    .format("%H:%M:%S")
                    .to_string();

                if tx.send(LogEntry {
                    file: file_name.clone(),
                    line: line.trim().to_string(),
                    timestamp,
                }).is_err() {
                    // 接收端关闭，退出
                    break;
                }
            }
            Err(e) => {
                eprintln!("读取错误: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("用法: log-watcher <file1> [file2...] [--pattern <pattern>]");
        std::process::exit(1);
    }

    // 解析参数
    let mut files = Vec::new();
    let mut pattern = None;
    let mut i = 1;

    while i < args.len() {
        if args[i] == "--pattern" && i + 1 < args.len() {
            pattern = Some(args[i + 1].clone());
            i += 2;
        } else {
            files.push(PathBuf::from(&args[i]));
            i += 1;
        }
    }

    if files.is_empty() {
        eprintln!("请指定至少一个文件");
        std::process::exit(1);
    }

    let (tx, rx) = mpsc::channel();
    let mut match_count = 0;

    println!("开始监控 {} 个文件...", files.len());
    if let Some(ref p) = pattern {
        println!("过滤模式: {}", p);
    }
    println!("按 Ctrl+C 停止\n");

    // 为每个文件启动监控线程
    for path in files {
        let tx = tx.clone();
        thread::spawn(move || watch_file(path, tx));
    }

    // 关闭原始发送端
    drop(tx);

    // 设置 Ctrl+C 处理（简化版）
    // 实际项目中应使用 ctrlc crate

    // 接收并处理日志
    for entry in rx {
        let should_print = match &pattern {
            Some(p) => entry.line.contains(p),
            None => true,
        };

        if should_print {
            println!("[{} {}] {}", entry.file, entry.timestamp, entry.line);
            match_count += 1;
        }
    }

    println!("\n监控结束，共匹配 {} 条", match_count);
}
```

### 简化版（不使用 chrono）

```rust
use std::sync::mpsc;
use std::thread;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

struct LogEntry {
    file: String,
    line: String,
}

fn watch_file(path: &str, tx: mpsc::Sender<LogEntry>) {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            if tx.send(LogEntry {
                file: path.to_string(),
                line,
            }).is_err() {
                break;
            }
        }
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let files = vec!["app.log", "web.log"];

    for path in files {
        let tx = tx.clone();
        let path = path.to_string();
        thread::spawn(move || watch_file(&path, tx));
    }

    drop(tx);

    let pattern = "ERROR";
    let mut count = 0;

    for entry in rx {
        if entry.line.contains(pattern) {
            println!("[{}] {}", entry.file, entry.line);
            count += 1;
        }
    }

    println!("共匹配 {} 条", count);
}
```

---

## 最佳实践

### 何时使用通道 vs 共享状态

| 场景 | 推荐方案 | 原因 |
|------|---------|------|
| 任务分发给工作线程 | 通道 | 自然的生产者-消费者模式 |
| 多线程更新同一个计数器 | Arc + Mutex | 简单，状态共享合理 |
| 事件通知 | 通道 | 解耦发送方和接收方 |
| 频繁读取的缓存 | Arc + RwLock | 读多写少，通道开销不值得 |
| 线程间传递大量数据 | 通道 | 所有权转移，避免克隆 |
| 配置热更新 | Arc + RwLock | 全局状态，随时读取 |

### 通道类型选择

| 需求 | 选择 | 示例 |
|------|------|------|
| 简单消息传递 | `mpsc::channel()` | 日志收集 |
| 限制内存使用 | `mpsc::sync_channel(n)` | 任务队列 |
| 确认消息送达 | `sync_channel(0)` | 关键操作 |
| 多消费者 | crossbeam-channel | 工作池 |
| 高性能 | crossbeam-channel | 高吞吐场景 |

### 常见陷阱

| 陷阱 | 问题 | 解决方案 |
|------|------|---------|
| 忘记 drop(tx) | for 循环永远等待 | 在循环前 drop 原始发送端 |
| 无界通道 | 内存溢出 | 使用 sync_channel 限制 |
| 单线程 recv | 死锁 | 确保发送和接收在不同线程 |
| send 后使用数据 | 编译错误 | 理解所有权转移 |
| 忽略 send 错误 | 静默失败 | 处理 SendError |

### 错误处理

```rust
use std::sync::mpsc::{SendError, RecvError};

// 发送可能失败（接收端已关闭）
match tx.send(data) {
    Ok(()) => println!("发送成功"),
    Err(SendError(data)) => {
        // 数据会返回，可以重试或记录
        eprintln!("发送失败，接收端已关闭");
    }
}

// 接收可能失败（发送端全部关闭）
match rx.recv() {
    Ok(msg) => println!("收到: {}", msg),
    Err(RecvError) => println!("所有发送端已关闭"),
}
```

---

## 与 Java/Go 对比

### Java BlockingQueue

```java
// Java
BlockingQueue<String> queue = new LinkedBlockingQueue<>();

// 生产者
queue.put("message");

// 消费者
String msg = queue.take();  // 阻塞
```

**Rust 的优势**：
- **编译期安全**：发送后不能再使用数据
- **所有权转移**：无数据竞争可能
- **无需 GC**：数据随通道移动，自动释放

### Go Channel

```go
// Go
ch := make(chan string, 10)  // 缓冲通道

// 发送
ch <- "message"

// 接收
msg := <-ch
```

**对比**：

| 特性 | Rust mpsc | Go channel |
|------|----------|------------|
| 类型安全 | 编译期 | 运行期（interface{} 时） |
| 多消费者 | 需要额外包装 | 原生支持 |
| 关闭检测 | 通过错误返回 | ok 语法 |
| Select | 需要 crossbeam | 原生 select |
| 所有权 | 强制转移 | 可能共享引用 |

---

## crossbeam-channel 简介

标准库的 mpsc 功能有限，生产环境常用 crossbeam-channel：

```toml
[dependencies]
crossbeam-channel = "0.5"
```

```rust
use crossbeam_channel::{bounded, select, unbounded};

// 创建通道
let (tx, rx) = bounded(10);  // 有界
let (tx, rx) = unbounded();  // 无界

// 多消费者支持
let rx2 = rx.clone();  // 可以克隆接收端！

// select! 宏 - 同时等待多个通道
let (tx1, rx1) = unbounded();
let (tx2, rx2) = unbounded();

select! {
    recv(rx1) -> msg => println!("从 rx1 收到: {:?}", msg),
    recv(rx2) -> msg => println!("从 rx2 收到: {:?}", msg),
    default => println!("没有消息"),
}
```

---

## 要点回顾

1. **mpsc** = Multiple Producer, Single Consumer
2. **发送转移所有权**：编译期保证无数据竞争
3. **drop(tx) 很重要**：否则 for 循环永远等待
4. **sync_channel** 提供背压，防止内存溢出
5. **选择合适的并发模型**：任务分发用通道，共享状态用锁

---

## 练习

1. **基础**：实现一个简单的聊天室，多个用户（线程）发送消息到一个通道，主线程显示
2. **进阶**：用工作池模式处理一批文件的 MD5 计算
3. **挑战**：实现带优先级的任务队列（高优先级任务先处理）

---

## 扩展阅读

- [The Rust Book - Message Passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html)
- [crossbeam-channel 文档](https://docs.rs/crossbeam-channel)
- [Rust 异步通道 vs 同步通道](https://tokio.rs/tokio/tutorial/channels)
- [Go 并发模式](https://go.dev/blog/pipelines)

---

## 下一章预告

学会了线程间通信，下一章开始网络编程。我们将用标准库实现一个简单的 TCP 服务器，为后续的异步网络编程打下基础。
