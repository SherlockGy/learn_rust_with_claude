# 第 6 章：结构体

## 本章目标

学完本章，你将能够：
- 定义和使用结构体
- 为结构体实现方法
- 理解关联函数与方法的区别
- 理解 Rust 为什么这样设计结构体和方法
- 知道何时用 `new()` 何时用结构体字面量
- 开始构建 task-cli 待办事项管理器

---

## 前置知识

- 第 1-5 章：Cargo、变量类型、函数、所有权、借用

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

**命名解释**：`struct` 是 structure 的缩写，表示"结构"——把相关的数据组织在一起。

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
// 等价于 Task { id: id, title: title, done: done }
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

### 3. 为什么数据与行为分离？

这是 Java 开发者学 Rust 时常问的问题：为什么不把方法写在结构体里面？

#### 设计哲学对比

**Java/C++ 的 OOP 方式：**
```java
public class Task {
    private int id;
    private String title;

    // 数据和方法绑定在一起
    public void markDone() { ... }
}
```

设计意图：对象是"活的"实体，数据和行为封装在一起。

**Rust 的方式：**
```rust
struct Task {
    id: u32,
    title: String,
}

impl Task {
    fn mark_done(&mut self) { ... }
}
```

设计意图：**组合优于继承，关注点分离**。

#### 这样设计的好处

**1. 可以为别人的类型添加方法**

```rust
// 你可以为标准库类型添加方法！（通过 trait）
trait Greet {
    fn greet(&self);
}

impl Greet for String {
    fn greet(&self) {
        println!("Hello, {}!", self);
    }
}

let name = String::from("Rust");
name.greet();  // 输出：Hello, Rust!
```

在 Java 中，你无法给 `java.lang.String` 添加方法。

**2. 清晰的数据布局**

struct 定义就是内存布局，一目了然。没有虚表、没有隐藏字段。

**3. 灵活的代码组织**

impl 块可以分散在多个地方，甚至多个文件：

```rust
// task.rs
struct Task { ... }

impl Task {
    fn new(...) -> Task { ... }
}

// task_display.rs（同一模块或使用 trait）
impl Task {
    fn display(&self) { ... }
}
```

这种设计让 Rust 的 trait 系统比 Java 接口更强大，我们将在第 9 章详细学习。

### 4. 方法定义

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

**命名解释**：`impl` 是 implementation 的缩写，表示"实现"——为这个类型实现方法。

**方法的 self 参数**：

| 形式 | 含义 | 使用场景 | 调用后实例状态 |
|------|------|---------|--------------|
| `&self` | 不可变借用 | 读取数据 | 仍可用 |
| `&mut self` | 可变借用 | 修改数据 | 仍可用 |
| `self` | 获取所有权 | 消耗/转换实例 | **被消耗，不能再用** |

这与你在第 4-5 章学的所有权规则完全一致！方法签名就是所有权规则的应用。

**调用方法**：

```rust
let mut task = Task { ... };
task.mark_done();                    // 需要 &mut self
println!("Done: {}", task.is_done()); // 只需要 &self
```

### 5. 关联函数

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

**为什么用 `::` 而不是 `.`？**

- `::` 表示"与类型关联"，不需要实例
- `.` 表示"在实例上调用"，需要实例

这是 Rust 的一致设计：
```rust
String::from("hello")  // 关联函数，创建 String
Vec::new()             // 关联函数，创建 Vec
std::fs::File::open()  // 关联函数，打开文件
```

**命名惯例**：
- `new`：创建新实例（Rust 没有构造函数，new 只是惯例）
- `from_xxx`：从其他类型转换（如 `from_str`）
- `with_xxx`：带特定配置创建（如 `with_capacity`）

### 6. 为什么没有构造函数？为什么显式 self？

#### 为什么没有构造函数？

**Java 的构造函数：**
```java
public class Task {
    public Task(int id, String title) {  // 特殊语法，名字必须和类名相同
        this.id = id;
        this.title = title;
    }
}
```

**Rust 选择普通函数：**
```rust
impl Task {
    fn new(id: u32, title: String) -> Task {  // 就是普通函数
        Task { id, title, done: false }
    }
}
```

**设计意图**：

1. **没有魔法**：`new` 不是关键字，只是社区约定。你可以叫 `create()`、`build()` 都行。

2. **多个"构造函数"平等对待**：
   ```rust
   impl Task {
       fn new(id: u32, title: String) -> Task { ... }
       fn with_done(id: u32, title: String, done: bool) -> Task { ... }
       fn from_json(json: &str) -> Result<Task, Error> { ... }
   }
   // 三个函数地位相同，没有主次之分
   ```

3. **返回值显式**：`-> Task` 明确告诉你返回什么类型，一目了然。

#### 为什么 self 必须显式？

**Java 的 this：**
```java
public void markDone() {
    this.done = true;  // this 可以省略
    done = true;       // 隐式访问 this.done
}
```

**Rust 要求显式 self：**
```rust
fn mark_done(&mut self) {  // 必须声明 self
    self.done = true;      // 必须写 self.
}
```

**设计意图**：

1. **明确所有权语义**：`self`、`&self`、`&mut self` 一眼就能看出方法如何使用实例。

2. **无隐式规则**：Rust 厌恶隐式行为。`self` 显式出现，没有"魔法变量"。

3. **与函数统一**：方法就是第一个参数是 self 的函数，没有特殊规则：
   ```rust
   // 这两种调用完全等价
   task.mark_done();
   Task::mark_done(&mut task);
   ```

### 7. 多个 impl 块

可以有多个 `impl` 块，Rust 会合并它们：

```rust
impl Task {
    fn new(...) -> Task { ... }
}

impl Task {
    fn mark_done(&mut self) { ... }
}
```

这在某些场景很有用（比如条件编译、组织代码、为不同 trait 分开实现）。

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

**命名解释**：`Vec` 是 Vector 的缩写，动态数组——可以动态增长的数组。

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

- **`&[Task]`**：Task 切片的引用。为什么不用 `&Vec<Task>`？

  ```rust
  fn list_tasks(tasks: &[Task]) {  // 接受切片
      // ...
  }

  let tasks: Vec<Task> = vec![...];
  list_tasks(&tasks);  // &Vec<Task> 自动转换为 &[Task]
  ```

  `&[Task]` 更通用，可以接受 `&Vec<Task>`、数组切片等。这是 Rust 惯用法：**函数参数用最宽松的类型**。

- **`{:>3}`**：格式化语法。`>` 表示右对齐，`3` 表示宽度。类似 Java 的 `String.format("%3d", id)`。

- **`"-".repeat(40)`**：重复字符串 40 次。

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
            // 注意：先打印再移动，因为 title 会被移动到 Task::new()
            println!("✓ 任务已添加 (ID: {}): {}", next_id, title);
            let task = Task::new(next_id, title);
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

### 语法与命名详解

上面代码中有几个新语法，让我们逐一解释：

#### `env::args().skip(1).collect()`

```rust
let args: Vec<String> = env::args().skip(1).collect();
```

- **`env::args()`**：返回命令行参数的迭代器。第一个参数是程序名本身。
- **`skip(1)`**：跳过前 1 个元素（程序名）。为什么叫 skip？想象排队，`skip(1)` 就是"跳过前面 1 个人"。
- **`collect()`**：将迭代器"收集"成一个集合。为什么叫 collect？想象散落在流水线上的零件，`collect` 把它们收集到容器里。

#### `command.as_str()`

```rust
match command.as_str() {
    "add" => ...
}
```

- **`as_str()`**：将 `&String` 转换为 `&str`（字符串切片）。
- **前缀 `as_`**：表示**低成本转换**，不分配新内存，只是"换个视角看"。
- **为什么需要？** `command` 是 `&String`，但 match 的模式 `"add"` 是 `&str`，需要统一类型。

#### `args[1..].join(" ")`

```rust
let title = args[1..].join(" ");
```

- **`args[1..]`**：范围语法（Range），表示从索引 1 到末尾的切片。
- **`join(" ")`**：用空格连接切片中的所有元素。

常见范围语法：
| 语法 | 含义 |
|------|------|
| `1..5` | 1, 2, 3, 4（不含 5） |
| `1..=5` | 1, 2, 3, 4, 5（含 5） |
| `..5` | 0, 1, 2, 3, 4 |
| `1..` | 从 1 到末尾 |
| `..` | 全部 |

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
```

#### `parse::<u32>()` - Turbofish 语法

```rust
match args[1].parse::<u32>() {
```

- **`parse()`**：将字符串"解析"成其他类型。
- **`::<u32>`**：这个语法叫 **turbofish**（涡轮鱼），用于显式指定泛型类型。

**为什么叫 turbofish？**
`::<>` 的形状像一条鱼：`::`是眼睛，`<>`是身体。它"加速"了类型推断。

**为什么需要它？**
```rust
// 编译器不知道要解析成什么类型
let id = args[1].parse();  // 错误！parse 成 i32? u32? f64?

// 方式 1：turbofish 语法
let id = args[1].parse::<u32>();  // 明确：解析为 u32

// 方式 2：类型标注
let id: Result<u32, _> = args[1].parse();  // 也行
```

---

## 完整代码

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
            // 注意所有权：先打印 title，再移动到 Task::new()
            // 因为 String 不是 Copy 类型，移动后就不能再使用了
            println!("✓ 任务已添加 (ID: {}): {}", next_id, title);
            let task = Task::new(next_id, title);  // title 被移动
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

4. **设计意图**
   - 数据与行为分离：组合优于继承
   - 无构造函数：普通函数更灵活
   - 显式 self：明确所有权语义

5. **命名惯例**
   - `new`：创建实例
   - `with_xxx`：带配置创建
   - `from_xxx`：类型转换
   - `as_xxx`：低成本转换

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

### 结构体初始化：new() vs 字面量

什么时候用 `Task::new(...)`，什么时候用 `Task { ... }` 字面量？

#### 两种方式对比

```rust
// 方式 1：结构体字面量
let task = Task {
    id: 1,
    title: String::from("Learn Rust"),
    done: false,
};

// 方式 2：关联函数 new()
let task = Task::new(1, String::from("Learn Rust"));
```

#### 选择原则

| 场景 | 推荐方式 | 理由 |
|------|---------|------|
| 需要设置默认值 | `new()` | 封装默认值逻辑，调用者不需知道 |
| 需要校验参数 | `new()` 或 `try_new()` | 可以在函数内校验并返回 Result |
| 所有字段都由调用者指定 | 字面量 | 更直观，IDE 补全友好 |
| 字段很多，只想设置几个 | Builder 模式 | 见下一节 |
| 测试代码/临时代码 | 字面量 | 快速创建实例 |
| 公开 API | `new()` | 隐藏内部实现细节 |

#### 为什么本章示例用 new()？

```rust
impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,  // 默认值！
        }
    }
}
```

因为：
1. **封装默认值**：新任务默认 `done: false`，调用者不需要关心
2. **减少重复**：每次创建都写 `done: false` 很烦
3. **易于修改**：如果以后添加新字段（如 `created_at`），只改 `new()` 即可

#### 什么时候直接用字面量？

```rust
// 在测试中，想要特定状态
let completed_task = Task {
    id: 1,
    title: String::from("Test"),
    done: true,  // 想要已完成状态
};

// 所有字段都需要调用者指定时
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 3.0, y: 4.0 };  // 直接字面量更清晰
```

#### 惯用写法总结

```rust
// 公开 API：提供 new()，封装合理默认值
impl Config {
    pub fn new(path: &str) -> Config {
        Config {
            path: path.to_string(),
            timeout: Duration::from_secs(30),  // 合理默认值
            retries: 3,
        }
    }
}

// 内部使用或测试：字面量也行
let config = Config {
    path: "/etc/app.conf".to_string(),
    timeout: Duration::from_secs(60),  // 覆盖默认值
    retries: 5,
};
```

### Builder 模式简介

当结构体字段很多，且大部分有默认值时，Builder 模式很有用。

#### 问题场景

```rust
struct Server {
    host: String,
    port: u16,
    max_connections: u32,
    timeout: u64,
    tls_enabled: bool,
    log_level: String,
}

// 问题：每次创建都要写所有字段
let server = Server {
    host: "localhost".to_string(),
    port: 8080,
    max_connections: 100,   // 想用默认值...
    timeout: 30,            // 想用默认值...
    tls_enabled: false,     // 想用默认值...
    log_level: "info".to_string(), // 想用默认值...
};
```

#### Builder 模式解决方案

```rust
struct ServerBuilder {
    host: String,
    port: u16,
    max_connections: u32,
    timeout: u64,
    tls_enabled: bool,
    log_level: String,
}

impl ServerBuilder {
    fn new(host: &str, port: u16) -> Self {
        ServerBuilder {
            host: host.to_string(),
            port,
            max_connections: 100,    // 默认值
            timeout: 30,             // 默认值
            tls_enabled: false,      // 默认值
            log_level: "info".to_string(),
        }
    }

    fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    fn timeout(mut self, secs: u64) -> Self {
        self.timeout = secs;
        self
    }

    fn tls(mut self, enabled: bool) -> Self {
        self.tls_enabled = enabled;
        self
    }

    fn build(self) -> Server {
        Server {
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
            timeout: self.timeout,
            tls_enabled: self.tls_enabled,
            log_level: self.log_level,
        }
    }
}

// 使用：链式调用，只设置需要的
let server = ServerBuilder::new("localhost", 8080)
    .timeout(60)
    .tls(true)
    .build();
```

#### 为什么 Builder 方法用 `mut self` 而不是 `&mut self`？

```rust
fn timeout(mut self, secs: u64) -> Self {  // 获取所有权
    self.timeout = secs;
    self  // 返回所有权
}
```

这样设计支持**链式调用**：
```rust
ServerBuilder::new("localhost", 8080)
    .timeout(60)   // 消耗 builder，返回新 builder
    .tls(true)     // 消耗上一个，返回新的
    .build()       // 最终消耗并构建
```

如果用 `&mut self`，就无法链式调用了：
```rust
// 假设用 &mut self
let mut builder = ServerBuilder::new("localhost", 8080);
builder.timeout(60);  // 返回 &mut Self
builder.tls(true);    // 需要分开写
let server = builder.build();  // 麻烦！
```

#### 本章不深入 Builder

对于 Task 这样简单的结构体，`new()` 足够了。Builder 模式适用于：
- 字段很多（>5 个）
- 大部分有默认值
- 想要链式 API

实际项目中，可以用 `derive_builder` crate 自动生成 Builder 代码。

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

   // 修复
   let mut task = Task::new(...);
   task.mark_done();  // OK
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
   // 调用后 task 就不能用了

   // 应该用 &mut self
   fn mark_done(&mut self) {
       self.done = true;
   }
   ```

4. **忘记所有权移动**：
   ```rust
   let title = String::from("Learn Rust");
   let task = Task::new(1, title);  // title 被移动
   println!("{}", title);  // 错误！title 已经被移动

   // 修复方案 1：先用再移动
   println!("{}", title);
   let task = Task::new(1, title);

   // 修复方案 2：借用
   println!("{}", &title);
   let task = Task::new(1, title);

   // 修复方案 3：clone（有性能成本）
   let task = Task::new(1, title.clone());
   println!("{}", title);
   ```

---

## 练习

### 练习 1：添加 remove 命令

实现删除任务功能：

```bash
$ task remove 1
✓ 任务 #1 已删除
```

提示：使用 `Vec::retain` 方法——它保留满足条件的元素，删除不满足的。

```rust
// retain 用法示例
let mut v = vec![1, 2, 3, 4, 5];
v.retain(|x| *x > 2);  // 保留大于 2 的
// v 现在是 [3, 4, 5]
```

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
