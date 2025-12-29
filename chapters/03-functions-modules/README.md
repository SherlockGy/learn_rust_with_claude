# 第 3 章：函数与模块

## 本章目标

学完本章，你将能够：
- 定义和调用函数，理解返回值语法
- 使用模块组织代码，理解 `mod`、`use`、`pub` 关键字
- 将单文件程序重构为多文件结构
- 理解 Rust 的可见性规则

---

## 前置知识

- 第 1-2 章：Cargo 基础、变量与类型、word-count 项目

---

## 项目：重构 word-count

### 为什么需要模块？

上一章的 `word-count` 所有代码都在 `main.rs` 里。这对于小程序没问题，但随着项目增长：

- 代码变得难以导航
- 函数职责不清晰
- 无法复用代码

**真实项目不可能都写在一个文件里**——是时候学习如何组织代码了。

### 本章目标结构

```
word-count/
├── Cargo.toml
└── src/
    ├── main.rs      # 入口点：解析参数、调用逻辑
    ├── counter.rs   # 核心逻辑：统计功能
    └── output.rs    # 输出逻辑：格式化显示
```

### 新增功能

重构的同时，我们添加文件读取功能：

```bash
# 从标准输入
$ echo "Hello" | word-count
       1       1       5

# 从文件读取
$ word-count README.md
      42     256    1832 README.md
```

---

## 核心概念

### 1. 函数定义

**基本语法**：

```rust
fn add(x: i32, y: i32) -> i32 {
    x + y  // 注意：没有分号，这是返回值
}

fn main() {
    let sum = add(5, 3);
    println!("Sum: {}", sum);
}
```

**语法要点**：
- `fn` 关键字声明函数
- 参数必须标注类型
- `-> Type` 指定返回类型（无返回值时省略）
- 函数体最后一个表达式就是返回值（无分号）

**命名解释**：`fn` 是 function 的缩写。

**表达式 vs 语句**：

```rust
fn example() -> i32 {
    let x = 5;      // 语句（以分号结尾，不返回值）
    x + 1           // 表达式（无分号，有值）
}

fn explicit_return() -> i32 {
    return 42;      // 也可以用 return 显式返回
}
```

**与 Java 对比**：

```java
// Java - 必须用 return
int add(int x, int y) {
    return x + y;
}
```

```rust
// Rust - 最后表达式自动返回
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

Rust 的设计更简洁，体现了"表达式优先"的函数式风格。

**无返回值的函数**：

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// 等价于显式返回 unit 类型
fn greet(name: &str) -> () {
    println!("Hello, {}!", name);
}
```

`()` 是 Rust 的 unit 类型，类似 Java 的 `void`，但它是一个真正的类型。

### 2. 模块系统

**为什么需要模块？**

模块提供：
- **命名空间**：避免名称冲突
- **封装**：控制可见性
- **组织**：逻辑分组

**声明模块**：

```rust
// main.rs
mod counter;  // 声明模块，Rust 会寻找 counter.rs 或 counter/mod.rs

fn main() {
    counter::count_words("hello world");
}
```

**命名解释**：`mod` 是 module 的缩写。

**模块文件解析规则**：

当你写 `mod foo;`，Rust 会按以下顺序查找：

1. `foo.rs`（同级目录）
2. `foo/mod.rs`（子目录）

```
src/
├── main.rs        // mod counter;
├── counter.rs     // 选项 1：单文件模块
└── counter/       // 选项 2：目录模块
    └── mod.rs
```

**可见性：pub 关键字**

默认情况下，模块中的所有内容都是私有的：

```rust
// counter.rs
fn private_fn() {
    // 只能在本模块内调用
}

pub fn public_fn() {
    // 可以被其他模块调用
}
```

**命名解释**：`pub` 是 public 的缩写。

**可见性级别**：

| 修饰符 | 可见范围 |
|--------|---------|
| （无） | 当前模块及子模块 |
| `pub` | 任何地方 |
| `pub(crate)` | 当前 crate 内 |
| `pub(super)` | 父模块 |
| `pub(in path)` | 指定路径 |

**与 Java 对比**：

| Java | Rust | 说明 |
|------|------|------|
| `private` | （默认） | 私有 |
| `protected` | 无直接对应 | Rust 用模块层级控制 |
| `public` | `pub` | 公开 |
| `package` | `pub(crate)` | 包/crate 内可见 |

### 3. 使用其他模块：use 关键字

```rust
// 方式 1：完整路径
std::io::stdin();

// 方式 2：use 引入
use std::io;
io::stdin();

// 方式 3：use 引入具体项
use std::io::stdin;
stdin();

// 方式 4：引入多个
use std::io::{stdin, stdout, BufRead};

// 方式 5：引入所有公开项（谨慎使用）
use std::io::*;
```

**命名解释**：`use` 表示"使用"某个路径下的项。

**路径类型**：

```rust
use std::io;          // 绝对路径：从 crate 根开始
use crate::counter;   // 当前 crate 的路径
use self::helper;     // 当前模块的子模块
use super::parent;    // 父模块的项
```

**命名解释**：
- `crate`：当前 crate 的根
- `self`：当前模块
- `super`：父模块

---

## 逐步重构 word-count

### 步骤 1：提取统计函数到模块

首先，创建 `counter.rs`：

```rust
// src/counter.rs

/// 统计结果
pub struct CountResult {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
}

/// 统计文本的行数、单词数、字符数
pub fn count_text(text: &str) -> CountResult {
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    let chars = text.chars().count();

    CountResult { lines, words, chars }
}
```

**语法说明**：
- `///` 是文档注释，会生成 API 文档
- `pub struct` 定义公开的结构体
- 结构体字段也需要 `pub` 才能在外部访问

### 步骤 2：创建输出模块

```rust
// src/output.rs
use crate::counter::CountResult;

/// 格式化输出统计结果
pub fn print_result(result: &CountResult, filename: Option<&str>) {
    match filename {
        Some(name) => {
            println!("{:>8}{:>8}{:>8} {}",
                result.lines, result.words, result.chars, name);
        }
        None => {
            println!("{:>8}{:>8}{:>8}",
                result.lines, result.words, result.chars);
        }
    }
}
```

**语法说明**：
- `use crate::counter::CountResult`：从当前 crate 的 counter 模块引入 CountResult
- `Option<&str>`：可选的字符串引用，表示可能有文件名也可能没有
- `match` 是模式匹配（后续章节详解）

### 步骤 3：重写 main.rs

```rust
// src/main.rs
mod counter;  // 声明 counter 模块
mod output;   // 声明 output 模块

use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        // 从标准输入读取
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).unwrap();
        let result = counter::count_text(&text);
        output::print_result(&result, None);
    } else {
        // 从文件读取
        for filename in &args {
            match fs::read_to_string(filename) {
                Ok(text) => {
                    let result = counter::count_text(&text);
                    output::print_result(&result, Some(filename));
                }
                Err(e) => {
                    eprintln!("word-count: {}: {}", filename, e);
                }
            }
        }
    }
}
```

**新语法说明**：

1. **`mod counter;`**：声明模块，告诉 Rust 去找 `counter.rs`

2. **`fs::read_to_string`**：读取整个文件为 String
   - 返回 `Result<String, Error>`
   - 成功返回 `Ok(内容)`，失败返回 `Err(错误)`

3. **`match`**：模式匹配
   - `Ok(text) => ...`：成功时执行
   - `Err(e) => ...`：失败时执行

4. **`eprintln!`**：打印到标准错误（stderr）
   - `e` = error，打印错误信息

5. **`&args`**：借用 args 进行迭代（不获取所有权）

### 步骤 4：项目结构

最终结构：

```
word-count/
├── Cargo.toml
└── src/
    ├── main.rs      # 入口：参数解析、文件读取
    ├── counter.rs   # 核心：统计逻辑
    └── output.rs    # 输出：格式化显示
```

---

## 完整代码

### Cargo.toml

```toml
[package]
name = "word-count"
version = "0.2.0"
edition = "2021"

[dependencies]
```

### src/counter.rs

```rust
/// 统计结果
pub struct CountResult {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
}

/// 统计文本的行数、单词数、字符数
pub fn count_text(text: &str) -> CountResult {
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    let chars = text.chars().count();

    CountResult { lines, words, chars }
}
```

### src/output.rs

```rust
use crate::counter::CountResult;

/// 格式化输出统计结果
pub fn print_result(result: &CountResult, filename: Option<&str>) {
    match filename {
        Some(name) => {
            println!("{:>8}{:>8}{:>8} {}",
                result.lines, result.words, result.chars, name);
        }
        None => {
            println!("{:>8}{:>8}{:>8}",
                result.lines, result.words, result.chars);
        }
    }
}
```

### src/main.rs

```rust
mod counter;
mod output;

use std::env;
use std::fs;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        // 从标准输入读取
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).unwrap();
        let result = counter::count_text(&text);
        output::print_result(&result, None);
    } else {
        // 从文件读取
        for filename in &args {
            match fs::read_to_string(filename) {
                Ok(text) => {
                    let result = counter::count_text(&text);
                    output::print_result(&result, Some(filename));
                }
                Err(e) => {
                    eprintln!("word-count: {}: {}", filename, e);
                }
            }
        }
    }
}
```

---

## 运行与测试

```bash
# 构建
cargo build

# 测试标准输入
$ echo "Hello World" | cargo run
       1       2      11

# 测试文件
$ cargo run -- Cargo.toml
       9      21     167 Cargo.toml

# 测试多个文件
$ cargo run -- src/main.rs src/counter.rs
      27      65     726 src/main.rs
      14      38     358 src/counter.rs

# 测试不存在的文件
$ cargo run -- nonexistent.txt
word-count: nonexistent.txt: No such file or directory (os error 2)
```

---

## 与 Java 对比

### 模块 vs 包

| Java | Rust | 说明 |
|------|------|------|
| package | mod（模块） | 代码组织单元 |
| import | use | 引入其他模块的项 |
| 类文件名必须匹配类名 | 模块名匹配文件名 | 命名约定 |
| 访问修饰符写在每个成员上 | 默认私有，显式 pub | 可见性控制 |

### 项目结构对比

```
// Java 项目
src/
├── main/
│   └── java/
│       └── com/
│           └── example/
│               ├── Main.java
│               ├── Counter.java
│               └── Output.java
└── test/
```

```
// Rust 项目
src/
├── main.rs
├── counter.rs
└── output.rs
```

Rust 的结构更扁平，不需要深层嵌套的目录。

---

## 要点回顾

1. **函数定义**
   - `fn name(params) -> ReturnType { body }`
   - 最后的表达式自动返回（无分号）
   - 参数必须标注类型

2. **模块系统**
   - `mod name;` 声明模块
   - 文件名即模块名
   - 默认私有，`pub` 公开

3. **use 关键字**
   - 引入其他模块的项
   - `crate::` 从当前 crate 根开始
   - `self::` 当前模块
   - `super::` 父模块

4. **项目组织**
   - 按职责拆分文件
   - main.rs 作为入口点
   - 其他逻辑放入独立模块

---

## 最佳实践

### 模块设计原则

| 原则 | 说明 |
|------|------|
| 单一职责 | 每个模块只做一件事 |
| 最小化 pub | 只公开必要的接口 |
| 清晰边界 | 模块间依赖关系要清晰 |

### pub 使用建议

```rust
// 好：只公开需要的
pub struct Config {
    pub name: String,     // 外部需要访问
    internal_id: u32,     // 内部实现细节
}

// 不好：全部公开
pub struct Config {
    pub name: String,
    pub internal_id: u32,  // 这个应该是私有的
}
```

### 何时拆分模块？

| 信号 | 行动 |
|------|------|
| 文件超过 300 行 | 考虑拆分 |
| 功能逻辑独立 | 拆分为独立模块 |
| 需要复用某些代码 | 提取为公共模块 |
| 测试某个功能困难 | 拆分后更容易测试 |

### 常见新手错误

1. **忘记 pub**：
   ```rust
   // counter.rs
   struct CountResult { ... }  // 忘记 pub，外部无法使用

   // main.rs
   let result: counter::CountResult = ...;  // 错误！
   ```

2. **模块声明位置错误**：
   ```rust
   // 错误：mod 声明必须在文件开头（use 之前）
   use std::io;
   mod counter;  // 应该在 use 之前
   ```

3. **路径混淆**：
   ```rust
   // counter.rs 中要引用同级模块 output
   use crate::output;  // 正确：从 crate 根开始
   use output;         // 错误：没有这个路径
   ```

---

## 练习

### 练习 1：添加汇总统计

多文件时输出汇总：

```bash
$ word-count file1.txt file2.txt
      10      50     500 file1.txt
      20     100    1000 file2.txt
      30     150    1500 total
```

提示：在 `CountResult` 上实现 `add` 方法或累加统计。

### 练习 2：提取文件读取到单独模块

创建 `reader.rs` 模块，处理文件和标准输入的读取逻辑。

### 练习 3：添加帮助信息模块

创建 `help.rs` 模块，支持 `--help` 选项。

---

## 扩展阅读

- [Rust Book: Defining Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [Rust Book: Paths](https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)

---

## 下一章预告

我们的代码组织好了，但还有一个重要问题：当我们传递字符串时，到底发生了什么？

```rust
fn count_text(text: &str) -> CountResult {
    // 为什么是 &str 而不是 String？
}
```

下一章，我们将深入 Rust 最核心的概念——**所有权系统**。这是 Rust 与其他语言最不同的地方，也是它能保证内存安全的关键。

准备好了吗？这将是 Rust 学习中最重要的一课。
