# 第 7 章：枚举与模式匹配

## 本章目标

学完本章，你将能够：
- 定义携带数据的枚举
- 使用 match 进行模式匹配
- 理解 Option 类型及其使用
- 为 task-cli 添加任务状态管理

---

## 前置知识

- 第 6 章：结构体、方法定义

---

## 项目：升级 task-cli 支持任务状态

### 本章目标

为 task-cli 添加更丰富的状态管理：

```bash
$ task list
ID  状态     任务
1   待办     学习枚举
2   进行中   写代码
3   完成     安装 Rust

$ task start 1
✓ 任务 #1 已开始
```

### 新增功能

- Status 枚举（Pending/InProgress/Done）
- Priority 枚举（Low/Medium/High）
- Option<String> 可选截止日期

---

## 核心概念

### 1. 基本枚举

```rust
enum Status {
    Pending,
    InProgress,
    Done,
}

let status = Status::Pending;
```

**命名解释**：`enum` 是 enumeration 的缩写，表示"枚举"。

**与 Java 对比**：

```java
// Java
enum Status {
    PENDING,
    IN_PROGRESS,
    DONE
}
```

```rust
// Rust - 语法相似
enum Status {
    Pending,
    InProgress,
    Done,
}
```

到这里看起来很像。但 Rust 枚举强大得多...

### 2. 携带数据的枚举

Rust 枚举的变体可以携带数据：

```rust
enum Message {
    Quit,                       // 无数据
    Move { x: i32, y: i32 },    // 命名字段
    Write(String),              // 单个值
    ChangeColor(i32, i32, i32), // 多个值
}

let msg = Message::Write(String::from("hello"));
let quit = Message::Quit;
let mv = Message::Move { x: 10, y: 20 };
```

**这是 Rust 枚举与 Java 的最大区别！**

Java 枚举只能有固定的属性，而 Rust 每个变体可以有不同的结构。

**实际应用**：

```rust
enum TaskEvent {
    Created { id: u32, title: String },
    Updated { id: u32, field: String, new_value: String },
    Completed { id: u32, completed_at: String },
    Deleted { id: u32 },
}
```

### 3. match 表达式

`match` 是 Rust 的模式匹配，类似 Java 的 switch 但更强大：

```rust
enum Status {
    Pending,
    InProgress,
    Done,
}

fn status_text(status: &Status) -> &str {
    match status {
        Status::Pending => "待办",
        Status::InProgress => "进行中",
        Status::Done => "完成",
    }
}
```

**关键特性**：

1. **必须穷尽所有可能**：
   ```rust
   match status {
       Status::Pending => "待办",
       // 错误！缺少 InProgress 和 Done
   }
   ```

2. **可以匹配携带的数据**：
   ```rust
   match message {
       Message::Quit => println!("Quit"),
       Message::Move { x, y } => println!("Move to ({}, {})", x, y),
       Message::Write(text) => println!("Text: {}", text),
       Message::ChangeColor(r, g, b) => println!("Color: {},{},{}", r, g, b),
   }
   ```

3. **使用 `_` 匹配剩余情况**：
   ```rust
   match status {
       Status::Done => "完成",
       _ => "未完成",  // 匹配其他所有情况
   }
   ```

### 4. Option 类型

Rust 没有 null。用 `Option` 表示"可能没有值"：

```rust
enum Option<T> {
    Some(T),  // 有值
    None,     // 无值
}
```

**命名解释**：
- `Some`：有一些（值）
- `None`：没有

**使用示例**：

```rust
fn find_task(tasks: &[Task], id: u32) -> Option<&Task> {
    for task in tasks {
        if task.id == id {
            return Some(task);
        }
    }
    None
}

// 使用
match find_task(&tasks, 1) {
    Some(task) => println!("找到: {}", task.title),
    None => println!("找不到任务"),
}
```

**与 Java 对比**：

```java
// Java - 可能返回 null
Task findTask(List<Task> tasks, int id) {
    for (Task t : tasks) {
        if (t.id == id) return t;
    }
    return null;  // 危险！调用者可能忘记检查
}
```

```rust
// Rust - 必须处理 None
fn find_task(...) -> Option<&Task> { ... }

// 不处理会编译错误
let task = find_task(...);
println!("{}", task.title);  // 错误！task 是 Option
```

### 5. Option 的常用方法

```rust
let x: Option<i32> = Some(5);
let y: Option<i32> = None;

// unwrap - 取出值，如果是 None 则 panic
let val = x.unwrap();  // 5
// let val = y.unwrap();  // panic!

// unwrap_or - 提供默认值
let val = y.unwrap_or(0);  // 0

// map - 转换内部值
let doubled = x.map(|v| v * 2);  // Some(10)

// is_some / is_none - 检查
if x.is_some() { ... }

// and_then - 链式处理
let result = x.and_then(|v| if v > 0 { Some(v) } else { None });
```

**命名解释**：
- `unwrap`：解包，取出包装的值
- `map`：映射，对内部值应用函数
- `and_then`：然后，链式组合

### 6. if let 简化匹配

当只关心一种情况时，用 `if let` 更简洁：

```rust
// match 写法
match option {
    Some(value) => println!("Got: {}", value),
    None => {},  // 必须写，即使什么都不做
}

// if let 写法
if let Some(value) = option {
    println!("Got: {}", value);
}
```

---

## 逐步升级 task-cli

### 步骤 1：定义状态和优先级枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Status {
    fn as_str(&self) -> &str {
        match self {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        }
    }
}

impl Priority {
    fn as_str(&self) -> &str {
        match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        }
    }
}
```

**`#[derive(...)]` 说明**：
- `Debug`：允许 `{:?}` 格式化
- `Clone`：允许复制
- `Copy`：允许按位复制（小类型）
- `PartialEq`：允许 `==` 比较

### 步骤 2：更新 Task 结构体

```rust
struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
    due_date: Option<String>,  // 可选的截止日期
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            status: Status::Pending,
            priority: Priority::Medium,
            due_date: None,
        }
    }

    fn with_priority(mut self, priority: Priority) -> Task {
        self.priority = priority;
        self
    }

    fn with_due_date(mut self, due: String) -> Task {
        self.due_date = Some(due);
        self
    }

    fn start(&mut self) {
        self.status = Status::InProgress;
    }

    fn complete(&mut self) {
        self.status = Status::Done;
    }
}
```

### 步骤 3：更新显示

```rust
impl Task {
    fn display(&self) {
        let due = match &self.due_date {
            Some(date) => date.as_str(),
            None => "-",
        };

        println!(
            "{:>3}  {:>4}  {:>6}  {:>10}  {}",
            self.id,
            self.priority.as_str(),
            self.status.as_str(),
            due,
            self.title
        );
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("{:>3}  {:>4}  {:>6}  {:>10}  任务", "ID", "优先级", "状态", "截止");
    println!("{}", "-".repeat(60));
    for task in tasks {
        task.display();
    }
}
```

### 步骤 4：添加新命令

```rust
fn main() {
    // ... 前面的代码

    match command.as_str() {
        "add" => { /* 原有逻辑 */ }
        "list" => { /* 原有逻辑 */ }

        "start" => {
            if args.len() < 2 {
                println!("用法: task start <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                    task.start();
                    println!("✓ 任务 #{} 已开始: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }

        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                    task.complete();
                    println!("✓ 任务 #{} 已完成: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }

        _ => { /* 原有逻辑 */ }
    }
}
```

### 完整代码

```rust
use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Status {
    fn as_str(&self) -> &str {
        match self {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        }
    }
}

impl Priority {
    fn as_str(&self) -> &str {
        match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        }
    }
}

struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
    due_date: Option<String>,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            status: Status::Pending,
            priority: Priority::Medium,
            due_date: None,
        }
    }

    fn start(&mut self) {
        self.status = Status::InProgress;
    }

    fn complete(&mut self) {
        self.status = Status::Done;
    }

    fn display(&self) {
        let due = match &self.due_date {
            Some(date) => date.as_str(),
            None => "-",
        };

        println!(
            "{:>3}  {:>4}  {:>6}  {:>10}  {}",
            self.id,
            self.priority.as_str(),
            self.status.as_str(),
            due,
            self.title
        );
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("{:>3}  {:>4}  {:>6}  {:>10}  任务", "ID", "优先级", "状态", "截止");
    println!("{}", "-".repeat(60));
    for task in tasks {
        task.display();
    }
}

fn find_task_mut(tasks: &mut [Task], id: u32) -> Option<&mut Task> {
    tasks.iter_mut().find(|t| t.id == id)
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务内容>  添加任务");
    println!("  task list            列出任务");
    println!("  task start <ID>      开始任务");
    println!("  task done <ID>       完成任务");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 预添加演示任务
    let mut t1 = Task::new(next_id, String::from("安装 Rust"));
    t1.complete();
    tasks.push(t1);
    next_id += 1;

    tasks.push(Task::new(next_id, String::from("学习枚举")));
    next_id += 1;

    let mut t3 = Task::new(next_id, String::from("写代码"));
    t3.start();
    tasks.push(t3);
    next_id += 1;

    if args.is_empty() {
        print_help();
        return;
    }

    let command = &args[0];
    match command.as_str() {
        "add" => {
            if args.len() < 2 {
                println!("用法: task add <任务内容>");
                return;
            }
            let title = args[1..].join(" ");
            let task = Task::new(next_id, title.clone());
            println!("✓ 任务已添加 (ID: {}): {}", task.id, title);
            tasks.push(task);
        }
        "list" => {
            list_tasks(&tasks);
        }
        "start" => {
            if args.len() < 2 {
                println!("用法: task start <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.start();
                    println!("✓ 任务 #{} 已开始: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            } else {
                println!("无效的 ID: {}", args[1]);
            }
        }
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.complete();
                    println!("✓ 任务 #{} 已完成: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            } else {
                println!("无效的 ID: {}", args[1]);
            }
        }
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }
}
```

---

## 运行与测试

```bash
$ cargo run -- list
 ID  优先级  状态    截止  任务
------------------------------------------------------------
  1    中    完成      -  安装 Rust
  2    中    待办      -  学习枚举
  3    中  进行中      -  写代码

$ cargo run -- start 2
✓ 任务 #2 已开始: 学习枚举

$ cargo run -- done 2
✓ 任务 #2 已完成: 学习枚举
```

---

## 要点回顾

1. **Rust 枚举可以携带数据**
   - 每个变体可以有不同结构
   - 比 Java enum 强大得多

2. **match 必须穷尽所有情况**
   - 编译器保证不遗漏
   - 用 `_` 匹配剩余

3. **Option 替代 null**
   - `Some(T)` 表示有值
   - `None` 表示无值
   - 必须显式处理

4. **if let 简化单分支匹配**
   - 只关心一种情况时使用

---

## 最佳实践

### if let vs match

```rust
// 只关心一种情况：用 if let
if let Some(value) = option {
    // 使用 value
}

// 多种情况或需要穷尽：用 match
match option {
    Some(v) if v > 0 => ...,
    Some(v) => ...,
    None => ...,
}
```

### Option 组合子

```rust
// 不好：嵌套 if let
if let Some(x) = get_x() {
    if let Some(y) = get_y(x) {
        do_something(y);
    }
}

// 好：链式调用
get_x()
    .and_then(|x| get_y(x))
    .map(|y| do_something(y));
```

### 何时用枚举 vs 结构体

| 场景 | 推荐 |
|------|------|
| 固定的互斥状态 | 枚举 |
| 多种不同形态的数据 | 携带数据的枚举 |
| 有多个并存字段 | 结构体 |
| 字段可选 | 结构体 + Option |

---

## 练习

### 练习 1：按状态过滤

实现 `task list --status pending` 只显示待办任务。

### 练习 2：添加优先级支持

实现 `task add "任务" --priority high` 指定优先级。

---

## 扩展阅读

- [The Rust Book - Enums](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [The Rust Book - Pattern Matching](https://doc.rust-lang.org/book/ch18-00-patterns.html)
- [Rust By Example - Enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html)

---

## 下一章预告

我们的任务数据在程序退出后就丢失了。下一章，我们将学习**错误处理**，为 task-cli 添加文件持久化功能。

你会学到 Rust 独特的错误处理方式——没有 try-catch，但更安全、更显式。
