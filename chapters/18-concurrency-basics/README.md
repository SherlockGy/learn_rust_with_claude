# 第 18 章：并发基础 - 线程与共享状态

## 本章目标

学完本章，你将能够：
- 创建和管理线程
- 理解 Rust 的"无畏并发"设计理念
- 使用 Arc 在线程间共享数据
- 使用 Mutex/RwLock 实现同步
- 理解 Send 和 Sync trait
- 实现 parallel-hash 并行哈希计算工具

---

## 前置知识

- 第 9 章：Trait
- 第 10 章：泛型
- 第 11 章：闭包（move 关键字）

---

## 项目：parallel-hash

### 功能概览

一个并行计算多个文件哈希值的工具，利用多核 CPU 加速计算。

### 为什么需要并发？

假设你要计算 100 个文件的 SHA256：
- **串行**：逐个计算，总时间 = 100 × 单个时间
- **并行**：同时计算，总时间 ≈ 单个时间（理想情况）

### 最终效果

```bash
$ parallel-hash *.txt
file1.txt  sha256:a1b2c3d4...
file2.txt  sha256:e5f6g7h8...
file3.txt  sha256:i9j0k1l2...

完成：3 个文件，用时 0.12 秒（串行需 0.35 秒）
```

---

## Rust 的"无畏并发"

### 并发编程的噩梦

在 C/C++/Java 中，并发编程充满陷阱：

```java
// Java - 经典的数据竞争
class Counter {
    private int count = 0;

    public void increment() {
        count++;  // 不是原子操作！
    }
}

// 多线程同时调用 increment()，结果不可预测
```

**常见问题**：
- **数据竞争**：多线程同时读写同一数据
- **死锁**：线程互相等待对方释放锁
- **悬垂指针**：线程访问已释放的内存

### Rust 的解决方案

Rust 在**编译时**阻止这些问题：

```rust
let mut data = vec![1, 2, 3];

// 尝试在线程中使用 data
thread::spawn(|| {
    println!("{:?}", data);  // 编译错误！
});

// Rust 阻止了可能的数据竞争
```

**编译器错误信息**：
```
error: closure may outlive the current function, but it borrows `data`
```

Rust 强制你显式处理所有权和同步，消除了运行时的不确定性。

---

## 核心概念

### 1. 创建线程

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // thread::spawn 创建新线程
    // 参数是一个闭包，定义线程要执行的代码
    let handle = thread::spawn(|| {
        for i in 1..5 {
            println!("子线程: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // 主线程继续执行
    for i in 1..3 {
        println!("主线程: {}", i);
        thread::sleep(Duration::from_millis(100));
    }

    // join() 等待子线程完成
    // 返回 Result，unwrap() 处理可能的 panic
    handle.join().unwrap();
}
```

**命名解释**：
- `spawn`：产生、孵化（新线程从主线程"孵化"出来）
- `join`：加入、汇合（等待子线程"汇合"到主线程）
- `JoinHandle`：连接句柄，用于等待和获取线程结果

### 2. move 闭包

线程闭包默认借用外部变量，但线程可能比变量活得更久：

```rust
let data = vec![1, 2, 3];

// 错误！data 可能在线程结束前被释放
let handle = thread::spawn(|| {
    println!("{:?}", data);
});
```

使用 `move` 强制转移所有权：

```rust
let data = vec![1, 2, 3];

let handle = thread::spawn(move || {
    // data 的所有权已转移到线程内
    println!("{:?}", data);
});

// println!("{:?}", data);  // 错误！data 已移动
```

### 3. Arc - 原子引用计数

`Arc`（Atomic Reference Counted）允许多个线程共享数据的所有权：

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    // Arc::new 创建原子引用计数的智能指针
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..3 {
        // Arc::clone 增加引用计数（不复制数据）
        let data = Arc::clone(&data);

        let handle = thread::spawn(move || {
            // 每个线程都有自己的 Arc，指向同一数据
            let sum: i32 = data.iter().sum();
            println!("线程 {} 计算和: {}", i, sum);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Arc vs Rc**：

| 特性 | Rc | Arc |
|------|-----|-----|
| 全称 | Reference Counted | Atomic Reference Counted |
| 线程安全 | 否 | 是 |
| 性能 | 更快 | 稍慢（原子操作开销） |
| 使用场景 | 单线程共享 | 多线程共享 |

```rust
// 单线程用 Rc
use std::rc::Rc;
let data = Rc::new(vec![1, 2, 3]);

// 多线程用 Arc
use std::sync::Arc;
let data = Arc::new(vec![1, 2, 3]);
```

### 4. Mutex - 互斥锁

`Arc` 只能共享不可变数据。要共享可变数据，需要 `Mutex`：

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Mutex 保护可变数据
    // Arc 让多个线程共享 Mutex
    let counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            // lock() 获取锁，返回 MutexGuard
            // MutexGuard 实现了 Deref，可以像普通引用一样使用
            let mut num = counter.lock().unwrap();
            *num += 1;
            // MutexGuard 离开作用域时自动释放锁
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("最终结果: {}", *counter.lock().unwrap());  // 10
}
```

**命名解释**：
- `Mutex`：Mutual Exclusion（互斥），同一时间只有一个线程能访问
- `lock()`：获取锁，如果锁被占用则阻塞等待
- `MutexGuard`：锁的守卫，离开作用域自动释放

**Mutex 的陷阱**：

```rust
// 死锁示例
let a = Arc::new(Mutex::new(1));
let b = Arc::new(Mutex::new(2));

// 线程 1：先锁 a，再锁 b
// 线程 2：先锁 b，再锁 a
// 可能死锁！

// 避免方法：始终按相同顺序获取锁
```

### 5. RwLock - 读写锁

`Mutex` 不区分读写，即使多个读操作也会互斥。`RwLock` 允许多读单写：

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));

    let mut handles = vec![];

    // 多个读线程可以并发
    for i in 0..3 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let read_guard = data.read().unwrap();
            println!("读线程 {}: {:?}", i, *read_guard);
        }));
    }

    // 写线程需要独占
    {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let mut write_guard = data.write().unwrap();
            write_guard.push(4);
            println!("写线程完成");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Mutex vs RwLock**：

| 特性 | Mutex | RwLock |
|------|-------|--------|
| 读-读 | 互斥 | 并发 |
| 读-写 | 互斥 | 互斥 |
| 写-写 | 互斥 | 互斥 |
| 适用场景 | 写多读少 | 读多写少 |
| 开销 | 较低 | 较高 |

### 6. Send 和 Sync trait

这两个 trait 是 Rust 并发安全的基石：

| Trait | 含义 | 示例 |
|-------|------|------|
| `Send` | 可以跨线程**移动**所有权 | 大多数类型 |
| `Sync` | 可以跨线程**共享引用** | `&T` 是 Send 当且仅当 T 是 Sync |

**不实现 Send/Sync 的类型**：
- `Rc<T>`：非原子引用计数，不安全
- `RefCell<T>`：非线程安全的内部可变性
- 裸指针 `*const T`、`*mut T`

```rust
use std::rc::Rc;
use std::thread;

let data = Rc::new(5);
thread::spawn(move || {
    println!("{}", data);  // 编译错误！Rc 不是 Send
});
```

**编译器错误**：
```
error: `Rc<i32>` cannot be sent between threads safely
```

---

## 项目实现：parallel-hash

```rust
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use sha2::{Sha256, Digest};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("用法: parallel-hash <文件>...");
        std::process::exit(1);
    }

    let paths: Vec<PathBuf> = args.iter()
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .collect();

    let start = Instant::now();

    // 并行计算
    let results = hash_files_parallel(&paths);

    // 输出结果
    for (path, hash) in &results {
        println!("{}  sha256:{}", path.display(), hash);
    }

    let duration = start.elapsed();
    println!("\n完成：{} 个文件，用时 {:.2} 秒", results.len(), duration.as_secs_f64());
}

/// 并行计算多个文件的哈希值
fn hash_files_parallel(paths: &[PathBuf]) -> Vec<(PathBuf, String)> {
    // 使用 Arc 共享文件路径列表
    let paths = Arc::new(paths.to_vec());
    let mut handles = vec![];

    for i in 0..paths.len() {
        let paths = Arc::clone(&paths);

        // 每个文件一个线程
        let handle = thread::spawn(move || {
            let path = &paths[i];
            let hash = compute_hash(path);
            (path.clone(), hash)
        });

        handles.push(handle);
    }

    // 收集所有结果
    handles.into_iter()
        .filter_map(|h| h.join().ok())
        .collect()
}

/// 计算单个文件的 SHA256 哈希
fn compute_hash(path: &PathBuf) -> String {
    match fs::read(path) {
        Ok(content) => {
            let hash = Sha256::digest(&content);
            format!("{:x}", hash)
        }
        Err(e) => format!("ERROR: {}", e),
    }
}
```

---

## 最佳实践

### 选择同步原语

| 场景 | 推荐 | 原因 |
|------|------|------|
| 单线程共享 | `Rc<T>` | 无原子操作开销 |
| 多线程只读共享 | `Arc<T>` | 不需要锁 |
| 多线程读写，写多 | `Arc<Mutex<T>>` | Mutex 开销更低 |
| 多线程读写，读多 | `Arc<RwLock<T>>` | 读可并发 |
| 简单计数器 | `AtomicUsize` | 无锁，最快 |

### 避免死锁

1. **固定加锁顺序**：所有线程按相同顺序获取多个锁
2. **缩小锁范围**：尽快释放锁
3. **使用 `try_lock`**：非阻塞尝试，失败则重试或放弃
4. **避免嵌套锁**：尽量不在持有锁时获取另一个锁

```rust
// 好：缩小锁范围
{
    let mut data = mutex.lock().unwrap();
    *data += 1;
}  // 锁在这里释放
do_expensive_work();  // 不持有锁

// 差：持有锁太久
let mut data = mutex.lock().unwrap();
*data += 1;
do_expensive_work();  // 持有锁期间其他线程阻塞
```

### 线程数量

```rust
// 获取 CPU 核心数
let num_cpus = std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4);

// CPU 密集型任务：线程数 ≈ 核心数
// IO 密集型任务：可以更多（因为线程常在等待）
```

### 常见陷阱

| 陷阱 | 问题 | 解决 |
|------|------|------|
| 忘记 `move` | 闭包借用导致编译错误 | 添加 `move` 关键字 |
| 用 `Rc` 代替 `Arc` | 编译错误（Rc 不是 Send） | 多线程用 `Arc` |
| 锁范围太大 | 性能下降 | 尽快释放锁 |
| 忽略 `join()` | 主线程结束时子线程被杀死 | 总是 `join()` 等待 |
| 死锁 | 程序卡住 | 固定加锁顺序 |

---

## 与 Java 并发对比

| 特性 | Rust | Java |
|------|------|------|
| 线程创建 | `thread::spawn` | `new Thread()` 或线程池 |
| 共享数据 | `Arc<Mutex<T>>` | `synchronized` / `Lock` |
| 数据竞争 | 编译时阻止 | 运行时可能发生 |
| 内存模型 | 所有权系统 | Java 内存模型 |
| 锁释放 | 自动（RAII） | 需要 finally 或 try-with-resources |

```java
// Java - 容易忘记释放锁
Lock lock = new ReentrantLock();
lock.lock();
try {
    // 临界区
} finally {
    lock.unlock();  // 必须手动释放
}
```

```rust
// Rust - 自动释放
let data = mutex.lock().unwrap();
// 临界区
// 离开作用域自动释放，不可能忘记
```

---

## 要点回顾

1. **无畏并发**：Rust 编译器在编译时阻止数据竞争
2. **`thread::spawn`**：创建线程，需要 `move` 闭包
3. **`Arc`**：原子引用计数，多线程共享所有权
4. **`Mutex`**：互斥锁，保护可变数据
5. **`RwLock`**：读写锁，读多写少时性能更好
6. **`Send`/`Sync`**：标记类型的线程安全性

---

## 练习

1. **基础**：创建两个线程，分别打印奇数和偶数（1-20）
2. **进阶**：实现一个线程安全的计数器，支持 `increment` 和 `get` 操作
3. **挑战**：使用多线程实现并行排序（如并行归并排序）

---

## 扩展阅读

- [The Rust Book - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust by Example - Threads](https://doc.rust-lang.org/rust-by-example/std_misc/threads.html)
- [std::sync 文档](https://doc.rust-lang.org/std/sync/)
- [Rustonomicon - Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)

---

## 下一章预告

线程间通信除了共享内存，还有消息传递。下一章学习通道（Channel），体验 "不要通过共享内存来通信，而要通过通信来共享内存" 的设计哲学。
