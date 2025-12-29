# 第 8 章：错误处理

## 本章目标

学完本章，你将能够：
- 使用 Result 类型处理可恢复错误
- 使用 ? 运算符简化错误传播
- 区分何时用 panic! 何时用 Result
- 为 task-cli 添加文件持久化

---

## 前置知识

- 第 7 章：枚举、Option、模式匹配

---

## 项目：升级 task-cli 支持文件持久化

### 本章目标

任务数据保存到文件，程序重启后不丢失：

```bash
$ task add "学习错误处理"
✓ 任务已添加 (ID: 1)

# 退出程序后重新运行
$ task list
ID  状态     任务
1   待办     学习错误处理   # 数据还在！
```

---

## 核心概念

### 1. Rust 的错误处理哲学

Rust 将错误分为两类：

| 类型 | 处理方式 | 典型场景 |
|------|---------|---------|
| 不可恢复 | `panic!` | 程序 bug，数组越界 |
| 可恢复 | `Result` | 文件不存在，网络超时 |

**与 Java 对比**：

| Java | Rust |
|------|------|
| `RuntimeException` | `panic!` |
| `checked Exception` | `Result<T, E>` |
| try-catch | match / ? |

### 2. Result 类型

```rust
enum Result<T, E> {
    Ok(T),   // 成功，包含结果
    Err(E),  // 失败，包含错误
}
```

**使用示例**：

```rust
use std::fs::File;

fn main() {
    let file = File::open("hello.txt");

    match file {
        Ok(f) => println!("文件打开成功"),
        Err(e) => println!("打开失败: {}", e),
    }
}
```

### 3. ? 运算符

`?` 是错误传播的语法糖：

```rust
fn read_file() -> Result<String, std::io::Error> {
    let mut file = File::open("hello.txt")?;  // 失败时提前返回 Err
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
```

等价于：

```rust
fn read_file() -> Result<String, std::io::Error> {
    let mut file = match File::open("hello.txt") {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    // ...
}
```

**命名解释**：`?` 可以理解为"如果出错就返回错误"。

### 4. 常用错误处理方法

```rust
let result: Result<i32, &str> = Ok(5);

// unwrap - 成功取值，失败 panic
let val = result.unwrap();

// expect - 同 unwrap，但可自定义错误信息
let val = result.expect("计算失败");

// unwrap_or - 提供默认值
let val = result.unwrap_or(0);

// map - 转换成功值
let doubled = result.map(|v| v * 2);

// map_err - 转换错误值
let converted = result.map_err(|e| format!("Error: {}", e));
```

### 5. 何时用什么？

| 场景 | 推荐方式 |
|------|---------|
| 快速原型 / 示例代码 | `unwrap()` |
| 确信不会失败 | `expect("原因")` |
| 需要向上传播 | `?` |
| 需要处理错误 | `match` |
| 提供默认值 | `unwrap_or` / `unwrap_or_default` |

---

## 逐步实现文件持久化

### 步骤 1：定义存储格式

使用简单的文本格式（每行一个任务）：

```
1|待办|中|学习 Rust|
2|完成|高|安装环境|2024-01-15
```

格式：`ID|状态|优先级|标题|截止日期(可选)`

### 步骤 2：实现保存功能

```rust
use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};

impl Task {
    fn to_line(&self) -> String {
        let status = match self.status {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        };
        let priority = match self.priority {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        let due = self.due_date.as_deref().unwrap_or("");

        format!("{}|{}|{}|{}|{}", self.id, status, priority, self.title, due)
    }
}

fn save_tasks(tasks: &[Task], path: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    for task in tasks {
        writeln!(file, "{}", task.to_line())?;
    }
    Ok(())
}
```

### 步骤 3：实现加载功能

```rust
fn parse_status(s: &str) -> Status {
    match s {
        "进行中" => Status::InProgress,
        "完成" => Status::Done,
        _ => Status::Pending,
    }
}

fn parse_priority(s: &str) -> Priority {
    match s {
        "低" => Priority::Low,
        "高" => Priority::High,
        _ => Priority::Medium,
    }
}

fn load_tasks(path: &str) -> io::Result<(Vec<Task>, u32)> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok((Vec::new(), 1));  // 文件不存在，返回空列表
        }
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    let mut max_id = 0u32;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() >= 4 {
            let id: u32 = parts[0].parse().unwrap_or(0);
            if id > max_id {
                max_id = id;
            }

            let task = Task {
                id,
                status: parse_status(parts[1]),
                priority: parse_priority(parts[2]),
                title: parts[3].to_string(),
                due_date: parts.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            };
            tasks.push(task);
        }
    }

    Ok((tasks, max_id + 1))
}
```

### 步骤 4：整合到主程序

```rust
const DATA_FILE: &str = "tasks.txt";

fn main() {
    // 加载任务
    let (mut tasks, mut next_id) = match load_tasks(DATA_FILE) {
        Ok((t, id)) => (t, id),
        Err(e) => {
            eprintln!("警告: 无法加载任务文件: {}", e);
            (Vec::new(), 1)
        }
    };

    // ... 处理命令 ...

    // 保存任务
    if let Err(e) = save_tasks(&tasks, DATA_FILE) {
        eprintln!("错误: 无法保存任务: {}", e);
    }
}
```

---

## 完整代码（关键部分）

```rust
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

// ... Status, Priority, Task 定义 ...

const DATA_FILE: &str = "tasks.txt";

fn save_tasks(tasks: &[Task], path: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    for task in tasks {
        writeln!(file, "{}", task.to_line())?;
    }
    Ok(())
}

fn load_tasks(path: &str) -> io::Result<(Vec<Task>, u32)> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok((Vec::new(), 1));
        }
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    let mut max_id = 0u32;

    for line in reader.lines() {
        let line = line?;
        if let Some(task) = Task::from_line(&line) {
            if task.id > max_id {
                max_id = task.id;
            }
            tasks.push(task);
        }
    }

    Ok((tasks, max_id + 1))
}

fn main() {
    let (mut tasks, mut next_id) = load_tasks(DATA_FILE).unwrap_or_else(|e| {
        eprintln!("警告: {}", e);
        (Vec::new(), 1)
    });

    // ... 命令处理逻辑 ...

    // 保存
    if let Err(e) = save_tasks(&tasks, DATA_FILE) {
        eprintln!("保存失败: {}", e);
    }
}
```

---

## 要点回顾

1. **Result 用于可恢复错误**
   - `Ok(T)` 成功
   - `Err(E)` 失败

2. **? 运算符简化错误传播**
   - 成功继续，失败提前返回

3. **区分 panic 和 Result**
   - bug 用 panic
   - 预期错误用 Result

4. **处理文件不存在**
   - 检查 `ErrorKind::NotFound`

---

## 最佳实践

| 场景 | 推荐 |
|------|------|
| 库代码 | 返回 Result，让调用者决定 |
| 应用代码 | 可以在顶层处理错误 |
| 测试代码 | 可用 unwrap |
| 文件操作 | 始终用 Result |

---

## 练习

### 练习 1：更友好的错误信息

当文件格式错误时，显示具体行号。

### 练习 2：备份功能

保存前创建 `.bak` 备份文件。

---

## 扩展阅读

- [The Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [anyhow crate](https://docs.rs/anyhow) - 应用级错误处理
- [thiserror crate](https://docs.rs/thiserror) - 库级错误定义

---

## 下一章预告

我们的代码可以工作，但输出格式写死了。下一章学习 **Trait**，让 Task 支持 Display 和 Debug，实现更灵活的格式化。
