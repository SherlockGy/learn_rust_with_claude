# 第 6 章：结构体

## 本章目标

学完本章，你将能够：
- 定义和使用结构体
- 为结构体实现方法
- 理解关联函数与方法的区别
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

## 逐步实现 task-cli

### 步骤 1：定义 Task 结构体

我们要表示一个"任务"。任务有哪些属性？
- ID：用于标识
- 标题：任务内容
- 是否完成：布尔值

在 Rust 中，用 **struct**（结构体）把相关数据组织在一起：

```rust
// src/main.rs

struct Task {
    id: u32,
    title: String,
    done: bool,
}
```

**命名解释**：`struct` 是 structure 的缩写，表示"结构"——把相关的数据字段组织成一个整体。

**创建实例**：

```rust
fn main() {
    let task = Task {
        id: 1,
        title: String::from("Learn Rust"),
        done: false,
    };

    println!("任务: {}", task.title);
}
```

用 `.` 访问字段，和大多数语言一样。

**字段简写**：当变量名与字段名相同时可以简写：

```rust
let id = 1;
let title = String::from("Learn Rust");
let done = false;

let task = Task { id, title, done };  // 简写形式
// 等价于 Task { id: id, title: title, done: done }
```

试着运行这段代码，确保能正常编译。

---

### 步骤 2：创建构造函数 - Task::new()

每次创建任务都要写 `done: false` 很烦。我们想要一个函数，只传 `id` 和 `title`，自动设置 `done: false`。

在 Rust 中，用 **impl 块**为结构体添加相关函数：

```rust
impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,  // 默认值
        }
    }
}
```

**命名解释**：
- `impl` 是 implementation 的缩写，表示"实现"——为这个类型实现功能
- `new` 不是关键字，只是 Rust 社区的命名惯例

**调用方式**——用 `::` 而不是 `.`：

```rust
let task = Task::new(1, String::from("Learn Rust"));
```

**为什么用 `::`？**

- `::` 表示"与类型关联"，不需要实例就能调用
- `.` 表示"在实例上调用"，需要先有实例

这类不需要实例的函数叫**关联函数**（类似 Java 的静态方法）。你已经见过很多了：

```rust
String::from("hello")  // 关联函数，创建 String
Vec::new()             // 关联函数，创建 Vec
```

现在我们可以更简洁地创建任务了：

```rust
fn main() {
    let task = Task::new(1, String::from("Learn Rust"));
    println!("任务 #{}: {}", task.id, task.title);
}
```

---

### 步骤 3：创建任务列表

一个任务不够，我们需要一个列表来存储多个任务。用 `Vec<Task>`：

```rust
fn main() {
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 添加测试任务
    tasks.push(Task::new(next_id, String::from("安装 Rust")));
    next_id += 1;

    tasks.push(Task::new(next_id, String::from("学习所有权")));
    next_id += 1;

    println!("共有 {} 个任务", tasks.len());
}
```

**命名解释**：`Vec` 是 Vector 的缩写，动态数组——长度可以动态增长的数组。

---

### 步骤 4：显示任务列表

我们想要漂亮地显示任务列表。这是一个"与 Task 相关的行为"，适合作为方法。

**方法**是定义在 impl 块中、第一个参数是 `self` 的函数：

```rust
impl Task {
    fn new(id: u32, title: String) -> Task {
        Task { id, title, done: false }
    }

    // 方法：第一个参数是 &self
    fn display(&self) {
        let status = if self.done { "✓" } else { "○" };
        println!("{:>3} [{}] {}", self.id, status, self.title);
    }
}
```

**`&self` 是什么意思？**

- `self` 代表"调用这个方法的实例"
- `&self` 表示"借用这个实例"（不可变借用）

这与你在第 4-5 章学的所有权规则完全一致！方法签名就是所有权规则的应用：

| 形式 | 含义 | 使用场景 |
|------|------|---------|
| `&self` | 不可变借用 | 只读取数据 |
| `&mut self` | 可变借用 | 需要修改数据 |
| `self` | 获取所有权 | 消耗/转换实例 |

`display()` 只是读取数据打印出来，所以用 `&self`。

**调用方法**——用 `.`：

```rust
let task = Task::new(1, String::from("Learn Rust"));
task.display();  // 用 . 调用方法
```

**关联函数 vs 方法的调用方式**：

```rust
Task::new(...)     // :: 关联函数，不需要实例
task.display()     // .  方法，需要实例
```

现在写一个函数来显示整个列表：

```rust
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

**`&[Task]` 是什么？**

这是"Task 切片的引用"。为什么不用 `&Vec<Task>`？

```rust
fn list_tasks(tasks: &[Task]) { ... }  // 接受切片

let tasks: Vec<Task> = vec![...];
list_tasks(&tasks);  // &Vec<Task> 自动转换为 &[Task]
```

`&[Task]` 更通用，可以接受 `&Vec<Task>`、数组切片等。这是 Rust 惯用法：**函数参数用最宽松的类型**。

**格式化语法**：

- `{:>3}`：右对齐，宽度 3。类似 Java 的 `String.format("%3d", id)`
- `"-".repeat(40)`：重复字符串 40 次

---

### 步骤 5：解析命令行参数

现在让程序接受命令行参数：

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 预添加测试任务
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
            println!("✓ 任务已添加 (ID: {}): {}", next_id, title);
            let task = Task::new(next_id, title);
            tasks.push(task);
        }
        "list" => {
            list_tasks(&tasks);
        }
        "done" => {
            // 下一步实现
            println!("done 命令待实现");
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

**几个语法点解释**：

#### `&args[0]` - 为什么要取引用？

```rust
let command = &args[0];
```

String 不是 Copy 类型，直接 `args[0]` 会尝试移动所有权，在 Vec 中留下"空洞"——Rust 不允许。加 `&` 表示借用。

**记住**：能 Copy 的类型（如 i32）随便取，不能 Copy 的要借用或克隆。

#### `command.as_str()`

```rust
match command.as_str() {
    "add" => ...
}
```

- `command` 是 `&String`，match 的模式 `"add"` 是 `&str`
- `as_str()` 将 `&String` 转换为 `&str`
- **`as_` 前缀**：表示低成本转换，不分配新内存

#### `args[1..].join(" ")`

```rust
let title = args[1..].join(" ");
```

- `args[1..]`：范围语法，从索引 1 到末尾的切片
- `join(" ")`：用空格连接所有元素

常见范围语法：

| 语法 | 含义 |
|------|------|
| `1..5` | 1, 2, 3, 4（不含 5） |
| `1..=5` | 1, 2, 3, 4, 5（含 5） |
| `1..` | 从 1 到末尾 |
| `..5` | 从 0 到 4 |

---

### 步骤 6：实现 done 命令

要标记任务完成，需要**修改** Task。这时候方法参数要用 `&mut self`：

```rust
impl Task {
    // ... new() 和 display() ...

    fn mark_done(&mut self) {
        self.done = true;
    }
}
```

`&mut self` 表示可变借用——可以修改实例的数据。

现在实现 done 命令：

```rust
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
```

#### `parse::<u32>()` - Turbofish 语法

```rust
args[1].parse::<u32>()
```

- `parse()`：将字符串解析成其他类型
- `::<u32>`：叫 **turbofish**（涡轮鱼），显式指定泛型类型

**为什么叫 turbofish？** `::<>` 的形状像一条鱼——`::`是眼睛，`<>`是身体。

**为什么需要它？**

```rust
let id = args[1].parse();  // 错误！parse 成什么类型？
let id = args[1].parse::<u32>();  // 明确：解析为 u32
```

#### `for task in &mut tasks`

遍历时需要可变引用才能调用 `mark_done(&mut self)`。

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

## 设计哲学：为什么 Rust 这样设计结构体

现在你已经实现了 task-cli，对 struct 和 impl 有了直观感受。来看看 Rust 背后的设计思想。

### 为什么数据与行为分离？

你可能注意到，Rust 的结构体只有数据，方法在 impl 块里单独定义。这和 Java 的 class 很不一样：

```java
// Java：数据和方法在一起
public class Task {
    private int id;
    private String title;

    public void markDone() { ... }  // 方法在类里面
}
```

```rust
// Rust：分离
struct Task {
    id: u32,
    title: String,
}

impl Task {
    fn mark_done(&mut self) { ... }  // 方法在 impl 块
}
```

**为什么这样设计？**

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

impl 块可以分散在多个地方，甚至多个文件。

这种设计让 Rust 的 trait 系统比 Java 接口更强大，我们将在第 9 章详细学习。

### 为什么没有构造函数？

Java 有特殊的构造函数语法：

```java
public class Task {
    public Task(int id, String title) {  // 特殊语法
        this.id = id;
        this.title = title;
    }
}
```

Rust 选择普通函数：

```rust
impl Task {
    fn new(id: u32, title: String) -> Task {  // 就是普通函数
        Task { id, title, done: false }
    }
}
```

**好处**：

1. **没有魔法**：`new` 不是关键字，只是约定。你可以叫 `create()`、`build()` 都行

2. **多个"构造函数"平等对待**：
   ```rust
   impl Task {
       fn new(id: u32, title: String) -> Task { ... }
       fn with_done(id: u32, title: String, done: bool) -> Task { ... }
       fn from_json(json: &str) -> Result<Task, Error> { ... }
   }
   // 三个函数地位相同，没有主次之分
   ```

3. **返回值显式**：`-> Task` 明确告诉你返回什么类型

### 为什么 self 必须显式？

Java 的 `this` 可以省略：

```java
public void markDone() {
    done = true;  // 隐式 this.done
}
```

Rust 要求显式 `self`：

```rust
fn mark_done(&mut self) {
    self.done = true;  // 必须写 self.
}
```

**好处**：

1. **明确所有权语义**：`self`、`&self`、`&mut self` 一眼就能看出方法如何使用实例

2. **无隐式规则**：Rust 厌恶隐式行为

3. **与函数统一**：方法就是第一个参数是 self 的函数
   ```rust
   // 这两种调用完全等价
   task.mark_done();
   Task::mark_done(&mut task);
   ```

### 与 Java 类的对比

| 方面 | Java | Rust |
|------|------|------|
| 数据与方法 | 在类中一起定义 | 分离：struct + impl |
| 构造函数 | 特殊语法 | 普通函数 (new 是惯例) |
| this/self | 隐式 | 显式 |
| 继承 | 支持 | 不支持，用组合和 trait |
| 默认可变性 | 可变 | 不可变 |

---

## 要点回顾

1. **结构体定义**：`struct Name { field: Type, ... }`

2. **impl 块**：为结构体添加关联函数和方法

3. **关联函数 vs 方法**：
   - 关联函数：无 self，用 `::` 调用（如 `Task::new()`）
   - 方法：有 self，用 `.` 调用（如 `task.display()`）

4. **self 的三种形式**（所有权规则的应用）：
   - `&self`：不可变借用，只读
   - `&mut self`：可变借用，可修改
   - `self`：获取所有权，消耗实例

5. **命名惯例**：
   - `new`：创建实例
   - `as_xxx`：低成本转换
   - `from_xxx`：类型转换

---

## 最佳实践

### new() vs 结构体字面量

什么时候用 `Task::new(...)`，什么时候用 `Task { ... }` 字面量？

| 场景 | 推荐方式 | 理由 |
|------|---------|------|
| 需要设置默认值 | `new()` | 封装默认值逻辑 |
| 需要校验参数 | `new()` | 可以返回 Result |
| 所有字段都由调用者指定 | 字面量 | 更直观 |
| 公开 API | `new()` | 隐藏内部细节 |
| 测试代码 | 字面量 | 快速创建特定状态 |

### 常见新手错误

**1. 忘记 mut**：
```rust
let task = Task::new(...);
task.mark_done();  // 错误！task 不可变

// 修复
let mut task = Task::new(...);
```

**2. 混淆 `.` 和 `::`**：
```rust
let task = Task.new(...);  // 错误！应该用 ::
let task = Task::new(...); // 正确
```

**3. self 参数错误**：
```rust
fn mark_done(self) { ... }  // 会消耗实例！调用后 task 不能再用

// 应该用 &mut self
fn mark_done(&mut self) { ... }
```

**4. 忘记所有权移动**：
```rust
let title = String::from("Learn Rust");
let task = Task::new(1, title);  // title 被移动
println!("{}", title);  // 错误！title 已移动

// 修复：先用再移动，或 clone
```

---

## 练习

### 练习 1：添加 remove 命令

实现删除任务功能：

```bash
$ task remove 1
✓ 任务 #1 已删除
```

提示：使用 `Vec::retain` 方法。

```rust
let mut v = vec![1, 2, 3, 4, 5];
v.retain(|x| *x > 2);  // 保留大于 2 的
// v 现在是 [3, 4, 5]
```

### 练习 2：添加优先级

为 Task 添加优先级字段，修改显示格式。

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
