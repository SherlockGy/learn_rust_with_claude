# 第 1 章：你好，Rust

## 本章目标

学完本章，你将能够：
- 使用 Cargo 创建、构建、运行 Rust 项目
- 理解 Rust 项目的基本结构
- 编写一个可以处理命令行参数的小程序

---

## 前置知识

- 已安装 Rust（通过 `rustup`）
- 熟悉任意一种命令行终端
- 有任意编程语言经验（本教程假设你有 Java 背景）

如果还没安装 Rust，请访问 https://rustup.rs/ 按提示安装。

---

## 项目：echo-rs - 你的第一个 Rust 程序

### 功能概览

`echo` 是最简单的 Unix 命令之一——它只做一件事：把你给它的文字原样输出。

```bash
$ echo Hello World
Hello World
```

我们要用 Rust 实现一个 `echo-rs`，虽然简单，但它能帮你快速理解：
- Rust 程序是怎么跑起来的
- 怎么接收用户输入（命令行参数）
- 怎么输出内容

### 为什么从这个项目开始？

1. **5 分钟内完成**：立即获得成就感，而不是在复杂概念中迷失
2. **真实可用**：完成后就是一个真正的命令行工具
3. **自然引出核心概念**：命令行参数 → 字符串处理 → 迭代器基础

### 最终效果

```bash
# 基本用法：输出所有参数
$ echo-rs Hello World
Hello World

# -n 选项：不输出末尾换行符
$ echo-rs -n "No newline"
No newline$   # 注意：没有换行，命令提示符紧跟输出
```

---

## 核心概念

### 1. Cargo：Rust 的项目管理器

**Cargo 是什么？**

Cargo 是 Rust 的官方构建系统和包管理器，相当于 Java 世界的 Maven + Gradle 的合体。

**命名解释**：Cargo 意为"货物"，管理你项目中的各种"货物"（依赖、构建产物等）。

**为什么需要 Cargo？**

在 C/C++ 中，你需要手写 Makefile、管理头文件路径、处理依赖下载……这些琐事让人头疼。Cargo 把这些全部自动化：

- 创建标准化的项目结构
- 管理依赖（自动下载、编译）
- 统一的构建命令
- 运行测试、生成文档

**与 Java 构建工具对比**：

| 功能 | Maven/Gradle | Cargo |
|------|-------------|-------|
| 配置文件 | `pom.xml` / `build.gradle` | `Cargo.toml` |
| 依赖仓库 | Maven Central | crates.io |
| 构建命令 | `mvn compile` / `gradle build` | `cargo build` |
| 运行命令 | `mvn exec:java` | `cargo run` |
| 配置语法 | XML / Groovy / Kotlin | TOML（简洁！） |

Cargo 的配置使用 TOML 格式，比 XML 简洁得多。

### 2. 创建项目

打开终端，运行：

```bash
cargo new echo-rs
cd echo-rs
```

**命名解释**：`new` 表示"新建"，`echo-rs` 是项目名（`-rs` 后缀是 Rust 社区命名 Rust 实现的惯例）。

Cargo 会创建以下结构：

```
echo-rs/
├── Cargo.toml    # 项目配置文件（元数据、依赖）
└── src/
    └── main.rs   # 程序入口
```

看看 `Cargo.toml`：

```toml
[package]
name = "echo-rs"       # 包名
version = "0.1.0"      # 版本号（语义化版本）
edition = "2021"       # Rust 版本（edition，不是编译器版本）

[dependencies]
# 这里将来添加依赖
```

**TOML 是什么？** Tom's Obvious, Minimal Language——一种简洁的配置文件格式。

看看默认生成的 `src/main.rs`：

```rust
fn main() {
    println!("Hello, world!");
}
```

### 3. 构建与运行

```bash
# 编译项目
cargo build

# 编译并运行
cargo run
```

**关键区别：Rust 编译为原生二进制**

Java 编译后得到 `.class` 字节码，需要 JVM 解释执行。Rust 直接编译为机器码，生成的可执行文件无需任何运行时：

```bash
# 编译后的二进制在这里
./target/debug/echo-rs    # Unix
.\target\debug\echo-rs.exe  # Windows
```

这个二进制可以直接复制到任何同架构的机器运行，不需要安装 Rust 或任何运行时。

**Release vs Debug 构建**：

```bash
cargo build           # Debug 模式：编译快，运行慢，有调试信息
cargo build --release # Release 模式：编译慢，运行快，优化后的二进制
```

---

## 逐步实现 echo-rs

### 步骤 1：理解 main 函数

```rust
fn main() {
    println!("Hello, world!");
}
```

**语法解释**：
- `fn`：function 的缩写，声明函数
- `main`：程序入口点（和 Java/C 一样）
- `{}`：代码块
- `println!`：打印宏（注意有 `!`，说明它是宏，不是普通函数）

**为什么 `println!` 是宏而不是函数？**

因为它能接受可变数量的参数和不同类型的格式化。在 Rust 中，普通函数的参数数量和类型是固定的，但宏可以在编译期展开，实现更灵活的功能。

现在你不需要深入理解宏，只需知道：带 `!` 的是宏，用法和函数类似。

### 步骤 2：获取命令行参数

我们需要获取用户在命令行输入的参数：

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
}
```

运行看看：

```bash
$ cargo run -- Hello World
["target/debug/echo-rs", "Hello", "World"]
```

**`--` 是什么？** 这是告诉 cargo："后面的参数传给程序，不是给 cargo 的"。

---

这行代码的模式你会经常看到，让我们用熟悉的 Java 概念来理解：

```rust
let args: Vec<String> = env::args().collect();
```

**`Vec<String>` 就是 Rust 版的 `ArrayList<String>`**：

| Java | Rust | 说明 |
|------|------|------|
| `ArrayList<String>` | `Vec<String>` | 可变长度的字符串数组 |
| `list.get(0)` | `args[0]` | 获取第一个元素 |
| `list.size()` | `args.len()` | 获取长度 |
| `list.isEmpty()` | `args.is_empty()` | 判断是否为空 |

**`env::args().collect()` 的流水线模式**：

这类似于 Java 8 的 Stream API：

```java
// Java 等价写法（概念上）
List<String> args = Arrays.stream(rawArgs)
    .collect(Collectors.toList());
```

```rust
// Rust 写法
let args: Vec<String> = env::args()  // 1. 获取参数迭代器
    .collect();                       // 2. 收集到 Vec 中
```

> **💡 暂时不用深入理解**
>
> 迭代器和 `.collect()` 的详细原理会在第 12 章深入讲解。
> 现在只需记住：这行代码的作用是"获取命令行参数，放到数组里"。

---

**代码各部分解释**：
- `use std::env`：引入标准库的 env 模块（类似 Java 的 `import`）
- `let`：声明变量（类似 Java 11+ 的 `var`，第 2 章详解）
- `env::args()`：获取命令行参数
- `{:?}`：调试格式打印（可以打印数组等复杂类型）

**注意**：Rust 的 args 第一个参数是程序路径，Java 的 args 不包含程序名！

### 步骤 3：输出参数

跳过第一个参数（程序名），输出剩余参数：

```rust
use std::env;

fn main() {
    // 跳过第一个参数（程序自身路径）
    let args: Vec<String> = env::args().skip(1).collect();

    // 用空格连接所有参数
    let output = args.join(" ");

    println!("{}", output);
}
```

**命名解释**：
- `skip(n)`：跳过前 n 个元素
- `join(sep)`：用分隔符 sep 连接所有元素

运行测试：

```bash
$ cargo run -- Hello World
Hello World
```

### 步骤 4：支持 -n 选项

真正的 `echo` 命令支持 `-n` 选项，表示不输出末尾换行符。让我们实现它：

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // 没有参数时，输出空行
    if args.is_empty() {
        println!();
        return;
    }

    // 检查第一个参数是否是 -n
    let no_newline = args[0] == "-n";

    // 确定要输出的文本参数
    let text_args = if no_newline {
        &args[1..]  // 跳过 -n，从第二个参数开始
    } else {
        &args[..]   // 使用全部参数
    };

    let output = text_args.join(" ");

    if no_newline {
        print!("{}", output);  // print! 不带换行
    } else {
        println!("{}", output);  // println! 带换行
    }
}
```

**新语法解释**：

**1. 切片（Slice）—— 类似 Java 的 `subList()`**

切片是对数组一部分的"视图"，不复制数据：

| Rust 切片 | 含义 | Java 类比 |
|----------|------|----------|
| `&args[1..]` | 从索引1到末尾 | `args.subList(1, args.size())` |
| `&args[..]` | 全部元素 | `args` 本身 |
| `&args[0..3]` | 索引0到2 | `args.subList(0, 3)` |

> **💡 关于 `&` 符号**
>
> 切片前面的 `&` 表示"借用"，这是 Rust 所有权系统的核心概念。
> 第 4 章会详细讲解，现在只需知道：`&args[1..]` 表示"引用 args 从索引1开始的部分"。

**2. `print!` vs `println!`**：
- `print!`：不带换行符
- `println!`：带换行符（ln = line，换行的意思）

**3. `if` 表达式返回值**：

注意这个写法——Rust 的 `if` 可以返回值：

```rust
let text_args = if no_newline {
    &args[1..]  // 如果有 -n，返回这个
} else {
    &args[..]   // 否则返回这个
};
```

这类似于 Java 的三元运算符 `? :`，但更灵活（可以写多行）。

### 完整代码

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!();
        return;
    }

    let no_newline = args[0] == "-n";
    let text_args = if no_newline {
        &args[1..]
    } else {
        &args[..]
    };

    let output = text_args.join(" ");

    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}
```

短短 20 行代码，我们就实现了一个功能完整的命令行工具！

---

## 运行与测试

```bash
# 构建 release 版本
cargo build --release

# 测试基本功能
$ cargo run -- Hello Rust
Hello Rust

# 测试 -n 选项
$ cargo run -- -n "No newline"
No newline$   # 命令提示符紧跟输出

# 测试空参数
$ cargo run
<空行>
```

构建完成后，你的可执行文件在 `target/release/echo-rs`（Windows 上是 `.exe`），可以复制到任何地方使用。

---

## 与 Java 对比

让我们对比实现相同功能的 Java 代码：

```java
// Echo.java
public class Echo {
    public static void main(String[] args) {
        if (args.length == 0) {
            System.out.println();
            return;
        }

        boolean noNewline = args[0].equals("-n");
        int start = noNewline ? 1 : 0;

        StringBuilder output = new StringBuilder();
        for (int i = start; i < args.length; i++) {
            if (i > start) output.append(" ");
            output.append(args[i]);
        }

        if (noNewline) {
            System.out.print(output);
        } else {
            System.out.println(output);
        }
    }
}
```

**关键差异**：

| 方面 | Java | Rust |
|------|------|------|
| 程序名 | args 不包含程序名 | args 第一个是程序路径 |
| 数组操作 | 手动循环拼接 | `join()` 方法更简洁 |
| 字符串 | 一种类型 String | 多种类型（后续章节详解） |
| 输出物 | 需要 JVM | 独立二进制 |
| 启动时间 | 需要 JVM 启动（~50ms+） | 几乎瞬间（~1ms） |

---

## 要点回顾

1. **Cargo 是 Rust 的标准构建工具**
   - `cargo new` 创建项目
   - `cargo build` 编译
   - `cargo run` 运行
   - `cargo build --release` 发布构建

2. **Rust 程序编译为原生二进制**
   - 无需运行时
   - 启动极快
   - 可独立分发

3. **基本语法**
   - `fn main()` 是程序入口
   - `println!` / `print!` 输出
   - `use` 引入模块
   - `let` 声明变量

4. **命名规律**
   - 标准库路径：`std::模块名::功能`
   - 迭代器方法：动词形式（`skip`、`collect`、`join`）

---

## 最佳实践

### 何时使用 `cargo run` vs 直接运行二进制？

| 场景 | 推荐方式 |
|------|---------|
| 开发调试 | `cargo run` |
| 性能测试 | `cargo run --release` 或直接运行 release 二进制 |
| 部署分发 | 复制 `target/release/` 下的二进制 |

### 项目命名惯例

- Rust 项目用 `kebab-case`（短横线分隔）：`echo-rs`、`word-count`
- Rust 实现的工具常加 `-rs` 后缀
- 包名和目录名保持一致

### 常见新手错误

1. **忘记 `--` 分隔符**：
   ```bash
   cargo run Hello    # 错误：cargo 会把 Hello 当成 cargo 的参数
   cargo run -- Hello # 正确：-- 后面的是程序参数
   ```

2. **修改代码后忘记重新编译**：`cargo run` 会自动重新编译，但直接运行二进制不会

3. **在错误的目录运行**：确保在项目根目录（有 `Cargo.toml` 的目录）运行 cargo 命令

---

## 练习

### 练习 1：添加 --help 选项 ⭐

让程序支持 `--help` 选项，输出使用说明：

```bash
$ echo-rs --help
Usage: echo-rs [-n] [STRING]...
Echo the STRING(s) to standard output.

  -n    do not output the trailing newline
```

**提示**：检查 `args[0]` 是否等于 `"--help"`，如果是就打印帮助信息并返回。

### 练习 2：同时支持 -n 和 --help ⭐⭐

让程序能正确处理多种情况：

```bash
$ echo-rs --help      # 输出帮助
$ echo-rs -n --help   # 应该输出 "--help"（-n 后面都是文本）
$ echo-rs -n Hello    # 输出 "Hello" 不换行
```

**提示**：注意处理顺序和边界情况。

### 练习 3：支持 -e 转义选项 ⭐⭐⭐（挑战）

> **💡 建议学完第 7 章后再尝试**
>
> 此练习需要用到模式匹配和字符串遍历，这些概念会在后续章节讲解。
> 如果你有 Rust 基础或想挑战自己，可以现在尝试。

真正的 `echo` 还支持 `-e` 选项，解释转义字符：

```bash
$ echo-rs -e "Hello\nWorld"
Hello
World

$ echo-rs -e "Tab:\there"
Tab:	here
```

需要处理的转义序列：`\n`（换行）、`\t`（制表符）、`\\`（反斜杠）。

---

## 扩展阅读

- [The Cargo Book](https://doc.rust-lang.org/cargo/)：Cargo 官方文档
- [Rust By Example: Hello World](https://doc.rust-lang.org/rust-by-example/hello.html)：更多输出格式化示例

---

## 下一章预告

我们的 echo-rs 工作正常！但代码中有些地方你可能还有疑问：

- 为什么用 `let` 声明变量？变量能修改吗？
- `Vec<String>` 的 `String` 和字符串字面量 `"hello"` 是一回事吗？
- `.skip(1).collect()` 这个链式调用到底怎么工作的？

下一章，我们将构建 **word-count** 工具（统计文本行数/单词数/字符数），深入学习：

- **Rust 默认不可变**：为什么这是刻意的设计？
- **`&str` vs `String`**：Rust 为什么有两种字符串类型？
- **类型推断**：为什么有时需要标注类型，有时不需要？

你会发现 Rust 的变量和类型系统与 Java 有根本性的不同——这正是 Rust 强大的基础。
