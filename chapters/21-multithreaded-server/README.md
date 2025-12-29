# 第 21 章：多线程网络服务

## 本章目标

学完本章，你将能够：
- 理解线程池的设计原理和实现
- 使用 Arc、Mutex、RwLock 实现并发安全的共享状态
- 升级 kv-server 支持多客户端并发
- 实现线程池的优雅关闭

---

## 前置知识

- 第 18 章：并发基础（线程、Send/Sync）
- 第 19 章：消息传递（Channel）
- 第 20 章：网络编程基础

---

## 项目：kv-server（多线程版）

### 功能概览

升级单线程 kv-server，支持多个客户端同时连接：

```bash
# 多个客户端可以同时连接和操作
$ nc localhost 7878 &   # 客户端 1
$ nc localhost 7878 &   # 客户端 2
$ nc localhost 7878 &   # 客户端 3

# 所有客户端共享同一个数据存储
# 客户端 1 设置的值，客户端 2 可以读取
```

### 架构变化

```
单线程版本：                    多线程版本：
┌─────────────┐               ┌─────────────┐
│  Listener   │               │  Listener   │
└──────┬──────┘               └──────┬──────┘
       │                             │
       ▼                             ▼
┌─────────────┐               ┌─────────────┐
│  处理连接    │               │  ThreadPool │
│  (阻塞)     │               └──────┬──────┘
└──────┬──────┘                      │
       │                    ┌────────┼────────┐
       ▼                    ▼        ▼        ▼
┌─────────────┐         Worker1  Worker2  Worker3
│  HashMap    │              │        │        │
│  (独占)     │              └────────┼────────┘
└─────────────┘                      ▼
                             ┌─────────────┐
                             │ Arc<RwLock> │
                             │  HashMap    │
                             └─────────────┘
```

---

## 核心概念

### 1. 为什么需要线程池？

**方案对比**：

| 方案 | 描述 | 问题 |
|------|------|------|
| 单线程 | 顺序处理 | 无法并发 |
| 每连接一线程 | 为每个连接创建线程 | 线程创建开销大，可能耗尽资源 |
| 线程池 | 固定数量线程复用 | 平衡并发和资源 |

**线程池的优势**：
- 避免频繁创建/销毁线程的开销
- 限制最大并发数，防止资源耗尽
- 线程复用，提高响应速度

### 2. 线程池设计

一个基本的线程池包含：
- **Worker 线程**：执行任务的工作线程
- **任务队列**：存放待执行任务的队列
- **分发器**：将任务分发给空闲 Worker

```
                    ┌─────────────────────────────────────┐
                    │            ThreadPool               │
                    │  ┌─────────────────────────────┐   │
    execute(job) ──►│  │     Channel (Job Queue)     │   │
                    │  └──────────────┬──────────────┘   │
                    │                 │                   │
                    │     ┌───────────┼───────────┐      │
                    │     ▼           ▼           ▼      │
                    │ ┌────────┐ ┌────────┐ ┌────────┐  │
                    │ │Worker 1│ │Worker 2│ │Worker 3│  │
                    │ └────────┘ └────────┘ └────────┘  │
                    └─────────────────────────────────────┘
```

### 3. 完整的线程池实现

```rust
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// Job 类型：可以跨线程传递的闭包
// FnOnce: 只执行一次
// Send: 可以跨线程传递
// 'static: 不依赖任何借用
type Job = Box<dyn FnOnce() + Send + 'static>;

// Worker：工作线程
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // 从队列获取任务
                // lock() 获取锁，recv() 阻塞等待任务
                let job = receiver.lock().unwrap().recv();

                match job {
                    Ok(job) => {
                        println!("Worker {} 收到任务", id);
                        job();  // 执行任务
                    }
                    Err(_) => {
                        // channel 关闭，退出循环
                        println!("Worker {} 退出", id);
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// 线程池
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// 创建线程池
    ///
    /// size: 线程数量
    ///
    /// # Panics
    /// 如果 size 为 0 则 panic
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "线程池大小必须大于 0");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// 提交任务到线程池
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(sender) = &self.sender {
            sender.send(job).unwrap();
        }
    }
}

// 优雅关闭
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 1. 关闭 channel，让 worker 退出循环
        drop(self.sender.take());

        // 2. 等待所有 worker 完成
        for worker in &mut self.workers {
            println!("关闭 Worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
```

### 4. 关键类型解释

**`type Job = Box<dyn FnOnce() + Send + 'static>`**：

| 部分 | 含义 |
|------|------|
| `Box<dyn ...>` | 动态分发的 trait object |
| `FnOnce()` | 只能调用一次的闭包（任务执行后就完成） |
| `Send` | 可以安全地跨线程传递 |
| `'static` | 不包含任何短生命周期的引用 |

**为什么需要 `Arc<Mutex<Receiver>>`**：

```rust
// Receiver 不实现 Clone，无法直接给多个线程
// 需要 Arc 共享所有权 + Mutex 保证同步访问

let receiver = Arc::new(Mutex::new(receiver));

// 每个 Worker 持有一个 Arc 克隆
let r = Arc::clone(&receiver);
```

---

## 共享状态

### 1. 选择合适的同步原语

| 同步原语 | 适用场景 | 特点 |
|---------|---------|------|
| `Mutex<T>` | 读写频率相当 | 简单，独占访问 |
| `RwLock<T>` | 读多写少 | 允许多读或单写 |
| 原子类型 | 简单计数器 | 无锁，最快 |

对于 kv-server：
- GET（读）操作可能比 SET（写）多
- 选择 `RwLock` 允许多个读取并发

### 2. RwLock 使用模式

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

// 定义共享存储类型
type Store = Arc<RwLock<HashMap<String, String>>>;

fn handle_get(store: &Store, key: &str) -> Option<String> {
    // read() 获取读锁，多个线程可以同时持有
    let guard = store.read().unwrap();
    guard.get(key).cloned()
}   // guard 离开作用域，自动释放读锁

fn handle_set(store: &Store, key: String, value: String) {
    // write() 获取写锁，独占访问
    let mut guard = store.write().unwrap();
    guard.insert(key, value);
}   // guard 离开作用域，自动释放写锁
```

### 3. 锁的作用域

**重要**：尽量缩小锁的作用域

```rust
// 不好：锁持有时间过长
fn handle_client_bad(store: &Store, stream: TcpStream) {
    let mut guard = store.write().unwrap();  // 获取写锁
    // ... 整个连接期间都持有锁 ...
    // 其他线程完全无法访问 store
}

// 好：只在需要时获取锁
fn handle_client_good(store: &Store, stream: TcpStream) {
    // ... 解析命令 ...

    // 只在访问数据时获取锁
    {
        let guard = store.read().unwrap();
        let value = guard.get(&key).cloned();
    }  // 立即释放锁

    // ... 发送响应 ...
}
```

---

## 完整的多线程 kv-server

```rust
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

mod thread_pool;
use thread_pool::ThreadPool;

// 共享存储类型
type Store = Arc<RwLock<HashMap<String, String>>>;

const ADDR: &str = "127.0.0.1:7878";
const POOL_SIZE: usize = 4;

fn main() {
    let listener = TcpListener::bind(ADDR).unwrap();
    println!("kv-server (多线程版) listening on {}", ADDR);

    // 创建共享存储
    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    // 创建线程池
    let pool = ThreadPool::new(POOL_SIZE);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 克隆 Arc，增加引用计数
                let store = Arc::clone(&store);

                // 将连接处理任务提交到线程池
                pool.execute(move || {
                    if let Err(e) = handle_client(stream, store) {
                        eprintln!("处理客户端出错: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }
}

fn handle_client(stream: TcpStream, store: Store) -> std::io::Result<()> {
    let addr = stream.peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("[{}] 连接", addr);

    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        println!("[{}] 命令: {}", addr, line);

        let response = process_command(&line, &store);
        writer.write_all(response.as_bytes())?;
        writer.flush()?;
    }

    println!("[{}] 断开", addr);
    Ok(())
}

fn process_command(line: &str, store: &Store) -> String {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["SET", key, value] => {
            // 获取写锁
            store.write().unwrap()
                .insert(key.to_string(), value.to_string());
            "OK\n".to_string()
        }
        ["GET", key] => {
            // 获取读锁
            match store.read().unwrap().get(*key) {
                Some(value) => format!("VALUE {}\n", value),
                None => "NOT_FOUND\n".to_string(),
            }
        }
        ["DEL", key] => {
            store.write().unwrap().remove(*key);
            "OK\n".to_string()
        }
        ["KEYS"] => {
            let guard = store.read().unwrap();
            if guard.is_empty() {
                "EMPTY\n".to_string()
            } else {
                let keys: Vec<&str> = guard.keys()
                    .map(|s| s.as_str())
                    .collect();
                format!("KEYS {}\n", keys.join(" "))
            }
        }
        ["COUNT"] => {
            let count = store.read().unwrap().len();
            format!("COUNT {}\n", count)
        }
        _ => format!("ERROR unknown command: {}\n", line),
    }
}
```

---

## 优雅关闭

### 为什么需要优雅关闭？

直接终止程序可能导致：
- 正在处理的请求中断
- 数据未保存
- 资源未释放

### 实现信号处理

```rust
use std::sync::atomic::{AtomicBool, Ordering};

// 全局关闭标志
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

fn main() {
    // 设置 Ctrl+C 处理
    ctrlc::set_handler(|| {
        println!("\n收到关闭信号...");
        SHUTDOWN.store(true, Ordering::SeqCst);
    }).expect("设置信号处理失败");

    let listener = TcpListener::bind(ADDR).unwrap();
    // 设置为非阻塞，这样可以定期检查关闭标志
    listener.set_nonblocking(true).unwrap();

    loop {
        // 检查关闭标志
        if SHUTDOWN.load(Ordering::SeqCst) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                // 处理连接...
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 没有新连接，等待一会儿
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                eprintln!("接受连接失败: {}", e);
            }
        }
    }

    println!("服务器正在关闭...");
    // ThreadPool 的 Drop 会等待所有任务完成
}
```

---

## 性能考量

### 1. 线程池大小选择

```rust
// 获取 CPU 核心数
let num_cpus = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4);

// 对于 I/O 密集型任务，可以多于 CPU 核心数
let pool_size = num_cpus * 2;

// 对于 CPU 密集型任务，等于或略小于核心数
let pool_size = num_cpus;
```

### 2. 锁争用优化

```rust
// 方案 1：分片锁（减少争用）
type ShardedStore = Vec<Arc<RwLock<HashMap<String, String>>>>;

fn get_shard(key: &str, shards: &ShardedStore) -> &Arc<RwLock<HashMap<String, String>>> {
    let hash = hash(key);
    let index = hash % shards.len();
    &shards[index]
}

// 方案 2：读写分离（使用无锁数据结构）
// 例如 crossbeam 的 SkipMap
```

### 3. 避免死锁

```rust
// 危险：可能死锁
fn transfer(from: &Store, to: &Store, key: &str) {
    let mut from_guard = from.write().unwrap();
    let mut to_guard = to.write().unwrap();  // 如果另一个线程以相反顺序获取锁...
    // ...
}

// 安全：固定获取锁的顺序
fn transfer_safe(stores: &[Store], from_idx: usize, to_idx: usize, key: &str) {
    let (first, second) = if from_idx < to_idx {
        (from_idx, to_idx)
    } else {
        (to_idx, from_idx)
    };

    let _guard1 = stores[first].write().unwrap();
    let _guard2 = stores[second].write().unwrap();
    // ...
}
```

---

## 与 Java 对比

### 线程池

```java
// Java - ExecutorService
ExecutorService pool = Executors.newFixedThreadPool(4);
pool.submit(() -> handleClient(socket));
pool.shutdown();
pool.awaitTermination(30, TimeUnit.SECONDS);
```

```rust
// Rust - 自定义 ThreadPool
let pool = ThreadPool::new(4);
pool.execute(move || handle_client(stream, store));
// Drop 时自动关闭
```

### 共享状态

```java
// Java - ConcurrentHashMap
ConcurrentHashMap<String, String> store = new ConcurrentHashMap<>();
store.put("key", "value");
String value = store.get("key");
```

```rust
// Rust - Arc<RwLock<HashMap>>
let store: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
store.write().unwrap().insert("key".to_string(), "value".to_string());
let value = store.read().unwrap().get("key").cloned();
```

**关键差异**：

| 方面 | Java | Rust |
|------|------|------|
| 线程安全 | 运行时检查 | 编译时保证 |
| 死锁检测 | 需要额外工具 | 某些情况编译器可检测 |
| 资源管理 | GC | RAII（Drop） |
| 闭包限制 | 可以捕获任意引用 | 必须满足 Send + 'static |

---

## 最佳实践

### 线程池

| 场景 | 推荐 |
|------|------|
| I/O 密集型 | 线程数 = CPU 核心 * 2 |
| CPU 密集型 | 线程数 = CPU 核心 |
| 混合负载 | 使用多个专用线程池 |

### 共享状态

| 场景 | 推荐 |
|------|------|
| 读多写少 | `RwLock` |
| 读写均衡 | `Mutex`（更简单） |
| 简单计数 | 原子类型 |
| 高并发场景 | 分片锁或无锁数据结构 |

### 常见陷阱

| 陷阱 | 说明 | 解决方案 |
|------|------|---------|
| 锁持有时间过长 | 阻塞其他线程 | 缩小临界区 |
| 死锁 | 循环等待锁 | 固定获取顺序 |
| 饥饿 | 某些线程一直得不到锁 | 使用公平锁或调整设计 |
| 忘记释放锁 | Rust 不存在（RAII） | - |
| 毒化的锁 | panic 时锁被毒化 | 使用 `.lock().unwrap()` 或处理毒化 |

---

## 要点回顾

1. **线程池**避免频繁创建销毁线程
2. **`Arc<Mutex<T>>`** 在线程间共享可变数据
3. **`Arc<RwLock<T>>`** 适合读多写少场景
4. **`FnOnce + Send + 'static`** 是闭包跨线程传递的约束
5. **优雅关闭**需要信号处理和等待任务完成
6. **锁作用域**应该尽可能小

---

## 练习

### 练习 1：线程池 Builder

为 ThreadPool 实现 Builder 模式，支持配置：
- 线程数量
- 线程名称前缀
- 栈大小

```rust
let pool = ThreadPool::builder()
    .num_threads(4)
    .thread_name("kv-worker")
    .build();
```

### 练习 2：优雅关闭

实现完整的优雅关闭：
- 捕获 Ctrl+C 信号
- 停止接受新连接
- 等待现有连接处理完成
- 保存数据到文件

### 练习 3：连接限制

限制最大并发连接数，超过时返回错误：
- 使用 Semaphore 或 AtomicUsize 计数
- 超过限制时立即关闭新连接并返回错误消息

### 练习 4：统计信息

添加统计端点，返回：
- 当前连接数
- 总请求数
- 键值对数量
- 线程池状态

---

## 扩展阅读

- [The Rust Book - 构建多线程 Web 服务器](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html)
- [crossbeam - 并发工具库](https://docs.rs/crossbeam)
- [rayon - 数据并行库](https://docs.rs/rayon)
- [parking_lot - 更快的同步原语](https://docs.rs/parking_lot)

---

## 下一章预告

多线程模型需要为每个连接分配一个线程（或复用线程池中的线程）。当连接数很大时，即使是线程池也会成为瓶颈。下一章学习异步编程，用更少的资源处理更多并发连接——这就是 Tokio 的世界。
