# 第 22 章：异步编程与 Tokio

## 本章目标

学完本章，你将能够：
- 理解 Rust 异步编程模型
- 掌握 async/await 语法
- 使用 Tokio 运行时构建异步应用
- 理解 Future trait 和异步执行原理
- 将 kv-server 升级为异步版本

---

## 前置知识

- 第 18 章：并发基础（线程、Arc、Mutex）
- 第 19 章：通道（消息传递）
- 第 20-21 章：网络编程（TCP）

---

## Tokio：Rust 异步生态的基石

### 背景介绍

**Tokio** 是 Rust 生态中最流行的异步运行时，由 Carl Lerche 创建并由 Tokio 团队维护。

**名字由来**：Tokyo（东京）的变体拼写，寓意"快速、高效"。

**生态地位**：
- crates.io 下载量前列，被 AWS、Discord、Cloudflare 等大规模使用
- 稳定维护超过 7 年（2017 年发布 0.1 版本）
- 1.0 版本于 2020 年发布，API 稳定
- Rust 异步 Web 框架（Axum、Actix-web、Warp）都基于 Tokio

**为什么需要 Tokio？** Rust 标准库只提供 async/await 语法和 Future trait，不包含运行时。Tokio 提供了：
- 异步任务调度器
- 异步 I/O（网络、文件）
- 定时器、同步原语等

---

### 设计理念

#### 问题：C10K 到 C10M

传统的"每连接一个线程"模型在高并发时面临瓶颈：
- 线程创建开销大（栈空间约 1-8 MB）
- 上下文切换成本高
- 10000 连接需要 10000 个线程，内存消耗巨大

**异步模型的解决方案**：
- 少量线程（通常等于 CPU 核心数）
- 每个线程可以处理成千上万个连接
- 任务在等待 I/O 时让出 CPU，执行其他任务

```
同步模型：
线程1: [等待IO.....................][处理]
线程2: [等待IO.....................][处理]
线程3: [等待IO.....................][处理]
...（需要很多线程）

异步模型：
线程1: [任务A等待][任务B处理][任务C等待][任务A处理][任务C处理]...
线程2: [任务D等待][任务E处理][任务F等待][任务D处理]...
（少量线程处理所有任务）
```

#### Tokio 的工作窃取调度器

Tokio 使用**工作窃取**（Work-Stealing）调度算法：
- 每个线程有自己的任务队列
- 空闲线程从忙碌线程"窃取"任务
- 自动负载均衡

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Worker 1  │  │   Worker 2  │  │   Worker 3  │
│ [任务A,B,C] │  │ [任务D]     │  │ [空]        │
└─────────────┘  └─────────────┘  └─────────────┘
                                        │
                                        │ 窃取
                                        ▼
                      ┌─────────────┐
                      │   Worker 1  │
                      │ [任务A,B]   │  ← Worker 3 窃取了任务 C
                      └─────────────┘
```

---

### 为什么 Rust 的异步很特别？

#### 零成本抽象

Rust 的 async/await 在**编译时**转换为状态机，运行时没有额外开销：

```rust
// 你写的代码
async fn fetch_data() -> String {
    let conn = connect().await;
    let data = conn.read().await;
    data
}

// 编译器生成的状态机（简化）
enum FetchDataState {
    Start,
    WaitingConnect(ConnectFuture),
    WaitingRead(Connection, ReadFuture),
    Done,
}
```

**对比其他语言**：
| 语言 | 实现方式 | 开销 |
|------|----------|------|
| JavaScript | 单线程事件循环 | 无法利用多核 |
| Python asyncio | 协程 + GIL | GIL 限制并发 |
| Go goroutine | 运行时调度 | 有栈协程，内存开销 |
| **Rust async** | 编译时状态机 | **零成本** |

#### 无栈协程

Rust 的 Future 是无栈的：
- 不需要预分配栈空间
- 每个 Future 只占用其实际需要的内存
- 可以创建数百万个 Future

---

## 核心概念

### 1. async/await 语法

```rust
// async fn 返回 impl Future
async fn fetch_user(id: u32) -> User {
    // .await 等待 Future 完成
    let response = http_get(&format!("/users/{}", id)).await;
    parse_user(response)
}

// 等价于
fn fetch_user(id: u32) -> impl Future<Output = User> {
    async move {
        let response = http_get(&format!("/users/{}", id)).await;
        parse_user(response)
    }
}
```

**关键理解**：
- `async fn` 定义异步函数，返回 `Future`
- `Future` 是惰性的，调用 async fn 不会立即执行
- `.await` 等待 Future 完成，期间可以执行其他任务

### 2. Future trait

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),    // 完成，返回结果
    Pending,     // 未完成，稍后再试
}
```

**poll 机制**：
1. 运行时调用 `poll`
2. 如果返回 `Pending`，Future 被挂起
3. 当 I/O 就绪时，运行时再次 `poll`
4. 返回 `Ready(T)` 表示完成

### 3. Tokio 运行时

```toml
# Cargo.toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
// 最简单的方式：#[tokio::main] 宏
#[tokio::main]
async fn main() {
    println!("Hello from async!");
}

// 等价于
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("Hello from async!");
    });
}
```

**运行时类型**：
```rust
// 多线程运行时（默认，适合生产）
#[tokio::main]
async fn main() { }

// 单线程运行时（适合测试和简单场景）
#[tokio::main(flavor = "current_thread")]
async fn main() { }

// 自定义配置
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .build()
    .unwrap();
```

### 4. tokio::spawn - 并发任务

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // spawn 创建并发任务
    let handle1 = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        println!("任务 1 完成");
        1
    });

    let handle2 = tokio::spawn(async {
        sleep(Duration::from_millis(500)).await;
        println!("任务 2 完成");
        2
    });

    // 等待任务完成
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    println!("结果: {} + {} = {}", result1, result2, result1 + result2);
}
```

**spawn vs thread::spawn**：
| `tokio::spawn` | `thread::spawn` |
|----------------|-----------------|
| 异步任务（轻量） | 操作系统线程（重量） |
| 可创建数百万个 | 通常限制在几千个 |
| 协作式调度 | 抢占式调度 |
| 需要 `Send` | 需要 `Send + 'static` |

### 5. 异步 I/O

```rust
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("监听端口 7878");

    loop {
        // accept 是异步的，等待时不阻塞线程
        let (socket, addr) = listener.accept().await.unwrap();
        println!("新连接: {}", addr);

        // 每个连接一个任务，而不是一个线程
        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}

async fn handle_connection(socket: tokio::net::TcpStream) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await.unwrap() > 0 {
        writer.write_all(b"OK\n").await.unwrap();
        line.clear();
    }
}
```

### 6. 异步同步原语

Tokio 提供了异步版本的同步原语：

```rust
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

// 异步 Mutex（可以跨 await 持有）
let data = Arc::new(Mutex::new(0));
let mut lock = data.lock().await;  // 异步等待锁
*lock += 1;

// 异步通道
let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    tx.send("hello").await.unwrap();
});

while let Some(msg) = rx.recv().await {
    println!("收到: {}", msg);
}
```

**std::sync vs tokio::sync**：
| 场景 | 使用 |
|------|------|
| 同步代码 | `std::sync::Mutex` |
| 锁持有时间短，不跨 await | `std::sync::Mutex` |
| 需要跨 await 持有锁 | `tokio::sync::Mutex` |

---

## 项目：async-kv（异步版 kv-server）

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

type Store = Arc<RwLock<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let store: Store = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    println!("async-kv 启动，监听 :7878");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("[{}] 连接", addr);

        let store = Arc::clone(&store);
        tokio::spawn(async move {
            handle_client(socket, store).await;
            println!("[{}] 断开", addr);
        });
    }
}

async fn handle_client(socket: TcpStream, store: Store) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await.unwrap() == 0 {
            break;
        }

        let response = process_command(line.trim(), &store).await;
        if writer.write_all(response.as_bytes()).await.is_err() {
            break;
        }
    }
}

async fn process_command(line: &str, store: &Store) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["SET", key, value] => {
            store.write().await.insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }
        ["GET", key] => {
            match store.read().await.get(*key) {
                Some(v) => format!("VALUE {}\n", v),
                None => "NOT_FOUND\n".to_string(),
            }
        }
        ["DEL", key] => {
            store.write().await.remove(*key);
            "OK\n".to_string()
        }
        _ => "ERROR\n".to_string(),
    }
}
```

---

## 同步 vs 异步对比

| 特性 | 同步（线程） | 异步（Tokio） |
|------|-------------|---------------|
| 并发模型 | 每连接一线程 | 多路复用 |
| 内存占用 | 高（每线程 1-8MB 栈） | 低（每任务几百字节） |
| 连接数上限 | 几千 | 几百万 |
| 代码复杂度 | 简单直观 | 需要理解 async |
| 适用场景 | CPU 密集、低并发 | I/O 密集、高并发 |

---

## 最佳实践

### 何时使用异步

| 场景 | 推荐 | 原因 |
|------|------|------|
| Web 服务器 | 异步 | 大量并发连接 |
| 数据库服务 | 异步 | I/O 密集 |
| CLI 工具 | 同步 | 简单，无需高并发 |
| 数值计算 | 同步 + 多线程 | CPU 密集 |
| 批处理任务 | 看情况 | 如果涉及大量 I/O 则异步 |

### 避免阻塞运行时

```rust
// 错误：在异步上下文中使用同步阻塞
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // 阻塞整个线程！
}

// 正确：使用异步版本
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// 如果必须使用阻塞操作，使用 spawn_blocking
async fn with_blocking() {
    let result = tokio::task::spawn_blocking(|| {
        // 阻塞操作在专门的线程池执行
        expensive_sync_computation()
    }).await.unwrap();
}
```

### 常见陷阱

| 陷阱 | 问题 | 解决 |
|------|------|------|
| 在 async 中用 `std::sync::Mutex` | 可能死锁 | 用 `tokio::sync::Mutex` 或缩小锁范围 |
| 忘记 `.await` | Future 不会执行 | 编译器会警告 |
| 阻塞操作 | 阻塞整个运行时 | 使用 `spawn_blocking` |
| 过多小任务 spawn | 调度开销 | 适当合并任务 |

---

## 要点回顾

1. **Tokio 是 Rust 最流行的异步运行时**
2. **async/await** 是语法糖，编译为状态机
3. **Future 是惰性的**，需要运行时驱动
4. **tokio::spawn** 创建轻量级并发任务
5. **异步适合 I/O 密集型高并发场景**

---

## 练习

1. **基础**：用 Tokio 实现一个异步的 echo 服务器
2. **进阶**：实现并发下载多个 URL（使用 `reqwest`）
3. **挑战**：给 async-kv 添加 EXPIRE 命令（键过期功能）

---

## 扩展阅读

- [Tokio 官方教程](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio 内部机制](https://tokio.rs/blog/2019-10-scheduler)
- [Alice Ryhl 的 Tokio 系列文章](https://ryhl.io/)

---

## 下一章预告

掌握了 Tokio，就可以学习基于它的 Web 框架。下一章学习 Axum，用类型安全的方式构建 REST API。
