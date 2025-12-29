# 第 6 章：结构体

## 本章目标

学完本章，你将能够：
- 定义和使用结构体
- 为结构体实现方法
- 理解关联函数与方法的区别
- 开始构建 task-cli 待办事项管理器

---

## 前置知识

- 第 1-5 章：Cargo、变量类型、函数、所有权

---

## 项目：task-cli - 命令行待办事项管理器

> **说明**：CLI 工具线（echo-rs、word-count、uniq-rs）将在第 12 章继续。
> 接下来几章我们专注于构建一个完整应用——task-cli。

### 功能概览

task-cli 是一个帮助你在终端管理日常待办事项的工具，类似于 Todoist/Things 的命令行版本。

### 为什么做这个项目？

- 程序员经常在终端工作，不想切换到 GUI
- 想要一个轻量、快速、可定制的任务管理工具
- 数据本地存储，隐私可控
- 可通过脚本自动化

### 最终效果（第 15 章完成后）

```bash
$ task add "学习 Rust 所有权" --due tomorrow
✓ 任务已添加 (ID: 1)

$ task list
ID  状态    任务
1   待办    学习 Rust 所有权
2   完成    安装 Rust 环境

$ task done 1
✓ 任务已完成
```

### 本章实现范围

本章实现基础版本：
- Task 结构体：id, title, done
- 内存中的任务列表
- 简单的 add/list/done 命令

后续章节会逐步添加：枚举状态、文件存储、JSON 格式、子命令等。

---

## 核心概念

### 1. 结构体定义

**基本语法**：

```rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}
```

**命名解释**：`struct` 是 structure 的缩写，表示"结构"。

**创建实例**：

```rust
let task = Task {
    id: 1,
    title: String::from("Learn Rust"),
    done: false,
};
```

**字段简写**：当变量名与字段名相同时可以简写：

```rust
let id = 1;
let title = String::from("Learn Rust");
let done = false;

let task = Task { id, title, done };  // 简写形式
```

**访问字段**：

```rust
println!("Task: {}", task.title);
```

**修改字段**（需要 mut）：

```rust
let mut task = Task { ... };
task.done = true;
```

### 2. 与 Java 类对比

```java
// Java
public class Task {
    private int id;
    private String title;
    private boolean done;

    // 需要 getter/setter
    public int getId() { return id; }
    public void setId(int id) { this.id = id; }
    // ...
}
```

```rust
// Rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}
// 字段直接访问（受可见性控制）
```

**关键差异**：

| Java | Rust |
|------|------|
| class 包含数据和方法 | struct 只有数据 |
| 方法在类内定义 | 方法在 impl 块定义 |
| 有继承 | 无继承，用组合和 trait |
| 默认可变 | 默认不可变 |

### 3. 方法定义

使用 `impl` 块为结构体定义方法：

```rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}

impl Task {
    // 方法：第一个参数是 self
    fn mark_done(&mut self) {
        self.done = true;
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn title(&self) -> &str {
        &self.title
    }
}
```

**命名解释**：`impl` 是 implementation 的缩写，表示"实现"。

**方法的 self 参数**：

| 形式 | 含义 | 使用场景 |
|------|------|---------|
| `&self` | 不可变借用 | 读取数据 |
| `&mut self` | 可变借用 | 修改数据 |
| `self` | 获取所有权 | 消耗实例 |

**调用方法**：

```rust
let mut task = Task { ... };
task.mark_done();
println!("Done: {}", task.is_done());
```

### 4. 关联函数

不以 `self` 开头的函数叫**关联函数**（类似 Java 的静态方法）：

```rust
impl Task {
    // 关联函数：用 :: 调用
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,
        }
    }
}

// 调用
let task = Task::new(1, String::from("Learn Rust"));
```

**命名惯例**：
- `new`：创建新实例（Rust 没有构造函数，new 只是惯例）
- `from_xxx`：从其他类型转换
- `with_xxx`：带特定配置创建

### 5. 多个 impl 块

可以有多个 `impl` 块，Rust 会合并它们：

```rust
impl Task {
    fn new(...) -> Task { ... }
}

impl Task {
    fn mark_done(&mut self) { ... }
}
```

这在某些场景很有用（比如条件编译、组织代码）。

---

## 逐步实现 task-cli

### 步骤 1：定义 Task 结构体

```rust
// src/main.rs

struct Task {
    id: u32,
    title: String,
    done: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,
        }
    }

    fn mark_done(&mut self) {
        self.done = true;
    }
}
```

### 步骤 2：创建任务列表

```rust
fn main() {
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 测试
    tasks.push(Task::new(next_id, String::from("Learn Rust")));
    next_id += 1;

    tasks.push(Task::new(next_id, String::from("Build task-cli")));
    next_id += 1;
}
```

**命名解释**：`Vec` 是 Vector 的缩写，动态数组。

### 步骤 3：显示任务

```rust
impl Task {
    fn display(&self) {
        let status = if self.done { "✓" } else { "○" };
        println!("{:>3} [{}] {}", self.id, status, self.title);
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("{:>3} 状态 任务", "ID");
    println!("{}", "-".repeat(40));
    for task in tasks {
        task.display();
    }
}
```

**语法说明**：
- `&[Task]`：Task 切片的引用，可以接受 `&Vec<Task>` 或其他切片
- `tasks.is_empty()`：检查是否为空
- `"-".repeat(40)`：重复字符串

### 步骤 4：解析命令

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

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
            let task = Task::new(next_id, title);
            println!("✓ 任务已添加 (ID: {})", task.id);
            tasks.push(task);
            next_id += 1;
        }
        "list" => {
            list_tasks(&tasks);
        }
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }
            // 解析 ID 并标记完成
        }
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务内容>  添加任务");
    println!("  task list            列出任务");
    println!("  task done <ID>       标记完成");
}
```

### 步骤 5：实现 done 命令

```rust
"done" => {
    if args.len() < 2 {
        println!("用法: task done <ID>");
        return;
    }

    match args[1].parse::<u32>() {
        Ok(id) => {
            // 查找并标记任务
            let mut found = false;
            for task in &mut tasks {
                if task.id == id {
                    task.mark_done();
                    println!("✓ 任务 #{} 已完成", id);
                    found = true;
                    break;
                }
            }
            if !found {
                println!("找不到任务 #{}", id);
            }
        }
        Err(_) => {
            println!("无效的 ID: {}", args[1]);
        }
    }
}
```

**语法说明**：
- `parse::<u32>()`：将字符串解析为 u32，返回 Result
- `&mut tasks`：可变借用任务列表，允许修改其中的任务

### 完整代码

```rust
use std::env;

struct Task {
    id: u32,
    title: String,
    done: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,
        }
    }

    fn mark_done(&mut self) {
        self.done = true;
    }

    fn display(&self) {
        let status = if self.done { "✓" } else { "○" };
        println!("{:>3} [{}] {}", self.id, status, self.title);
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("{:>3} 状态 任务", "ID");
    println!("{}", "-".repeat(40));
    for task in tasks {
        task.display();
    }
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务内容>  添加任务");
    println!("  task list            列出任务");
    println!("  task done <ID>       标记完成");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 为了演示，预添加一些任务
    tasks.push(Task::new(next_id, String::from("安装 Rust")));
    next_id += 1;
    tasks.push(Task::new(next_id, String::from("学习所有权")));
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
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }

            match args[1].parse::<u32>() {
                Ok(id) => {
                    let mut found = false;
                    for task in &mut tasks {
                        if task.id == id {
                            task.mark_done();
                            println!("✓ 任务 #{} 已完成: {}", id, task.title);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        println!("找不到任务 #{}", id);
                    }
                }
                Err(_) => {
                    println!("无效的 ID: {}", args[1]);
                }
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
# 显示帮助
$ cargo run

# 列出任务（预置的）
$ cargo run -- list
 ID 状态 任务
----------------------------------------
  1 [○] 安装 Rust
  2 [○] 学习所有权

# 添加任务
$ cargo run -- add 学习结构体
✓ 任务已添加 (ID: 3): 学习结构体

# 标记完成
$ cargo run -- done 1
✓ 任务 #1 已完成: 安装 Rust
```

**注意**：当前版本任务存储在内存中，程序退出后丢失。第 8 章会添加文件持久化。

---

## 与 Java 对比

### 类定义

```java
public class Task {
    private int id;
    private String title;
    private boolean done;

    public Task(int id, String title) {
        this.id = id;
        this.title = title;
        this.done = false;
    }

    public void markDone() {
        this.done = true;
    }
}
```

```rust
struct Task {
    id: u32,
    title: String,
    done: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task { id, title, done: false }
    }

    fn mark_done(&mut self) {
        self.done = true;
    }
}
```

**关键差异**：

| 方面 | Java | Rust |
|------|------|------|
| 数据与方法 | 在类中一起 | 分离：struct + impl |
| 构造函数 | 特殊语法 | 普通函数 (new) |
| this | 隐式 | 显式 (self) |
| 继承 | 支持 | 不支持，用组合 |

---

## 要点回顾

1. **结构体定义**
   - `struct Name { field: Type, ... }`
   - 字段简写：`{ x }` 等价于 `{ x: x }`

2. **impl 块**
   - 定义方法和关联函数
   - 方法第一个参数是 self
   - 关联函数无 self，用 `::` 调用

3. **self 变体**
   - `&self`：不可变借用
   - `&mut self`：可变借用
   - `self`：获取所有权

4. **命名惯例**
   - `new`：创建实例
   - `with_xxx`：带配置创建
   - `from_xxx`：类型转换

---

## 最佳实践

### 结构体设计

```rust
// 好：字段有意义、类型准确
struct User {
    id: u64,
    email: String,
    is_active: bool,
}

// 不好：用元组或过于通用的类型
struct User {
    data: (u64, String, bool),  // 不清晰
}
```

### 可见性设计

```rust
// 好：只公开必要的
pub struct Task {
    id: u32,           // 外部可读但不应直接改
    pub title: String, // 外部可改
    done: bool,        // 通过方法控制
}

impl Task {
    pub fn id(&self) -> u32 { self.id }  // 只读访问
    pub fn mark_done(&mut self) { self.done = true; }
}
```

### 方法 vs 关联函数

```rust
impl Task {
    // 关联函数：创建或与类型相关，不需要实例
    fn new(...) -> Task { ... }
    fn default_priority() -> u8 { 3 }

    // 方法：操作实例
    fn mark_done(&mut self) { ... }
    fn is_done(&self) -> bool { ... }
}
```

### 常见新手错误

1. **忘记 mut**：
   ```rust
   let task = Task::new(...);
   task.mark_done();  // 错误！task 不可变
   ```

2. **混淆 `.` 和 `::`**：
   ```rust
   let task = Task.new(...);  // 错误！应该用 ::
   let task = Task::new(...); // 正确
   ```

3. **self 参数错误**：
   ```rust
   fn mark_done(self) {  // 会消耗实例！
       self.done = true;
   }
   // 应该用 &mut self
   ```

---

## 练习

### 练习 1：添加 remove 命令

实现删除任务功能：

```bash
$ task remove 1
✓ 任务 #1 已删除
```

提示：使用 `Vec::retain` 或 `Vec::remove`。

### 练习 2：添加优先级

为 Task 添加优先级字段，修改显示格式：

```bash
$ task list
 ID 优先级 状态 任务
  1    高   [○] 紧急任务
  2    中   [✓] 普通任务
```

### 练习 3：统计功能

实现 `task stats` 命令：

```bash
$ task stats
总计: 5 个任务
待办: 3 | 完成: 2
```

---

## 扩展阅读

- [Rust Book: Defining Structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html)
- [Rust Book: Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)

---

## 下一章预告

我们的 task-cli 只能标记"完成"或"未完成"。但真实的任务管理器需要更多状态：待办、进行中、已完成、已归档...

下一章，我们将学习 Rust 强大的**枚举**类型，为 task-cli 添加丰富的状态管理。

Rust 的枚举远比 Java enum 强大——它可以携带数据，是实现状态机的利器。
