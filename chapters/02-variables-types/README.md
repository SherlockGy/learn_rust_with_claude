# 第 2 章：变量与类型

## 本章目标

学完本章，你将能够：
- 理解 Rust 默认不可变的设计哲学
- 掌握基本数据类型和类型推断
- 初步了解 `&str` 和 `String` 的区别
- 构建一个能统计文本的 word-count 工具

---

## 前置知识

- 第 1 章：Cargo 基础、命令行参数处理

---

## 项目：word-count - 文本统计工具

### 功能概览

`wc`（word count）是 Unix 系统中最常用的文本统计工具之一：

```bash
$ wc README.md
     42     256    1832 README.md
#   行数   单词数   字符数
```

我们要用 Rust 实现一个 `word-count`，统计文本的行数、单词数、字符数。

### 为什么做这个项目？

1. **需要处理字符串**：自然引出 Rust 的字符串类型
2. **需要声明变量**：计数器需要可变变量
3. **需要类型转换**：统计结果需要格式化输出
4. **实用性强**：文本分析的基础工具

### 最终效果

```bash
# 从标准输入读取
$ echo "Hello World" | word-count
      1       2      12

# 从文件读取（下一章实现）
$ word-count README.md
     42     256    1832 README.md
```

### 本章实现范围

本章实现基础版本：
- 从标准输入读取文本
- 统计行数、单词数、字符数
- 格式化对齐输出

下一章会重构为多文件结构并添加文件读取功能。

---

## 核心概念

### 1. 变量与不可变性

**Rust 变量默认不可变**

```rust
let x = 5;
x = 6;  // 错误！cannot assign twice to immutable variable
```

**为什么默认不可变？**

这是 Rust 的核心设计哲学之一：
- **减少 bug**：很多 bug 来自意外修改变量
- **便于推理**：不可变值更容易理解代码行为
- **并发安全**：不可变数据天然线程安全

**与 Java 对比**：

| 语言 | 默认行为 | 声明不可变 |
|------|---------|-----------|
| Java | 可变 | `final int x = 5;` |
| Rust | 不可变 | `let x = 5;` |
| Rust | 可变 | `let mut x = 5;` |

**命名解释**：`mut` 是 mutable（可变）的缩写。

```rust
let mut count = 0;  // 声明可变变量
count = count + 1;  // 现在可以修改了
count += 1;         // 简写形式
```

**不可变绑定 vs 常量**

Rust 还有 `const` 声明真正的常量：

```rust
const MAX_POINTS: u32 = 100_000;  // 编译期常量，必须标注类型
let x = 5;                         // 运行时绑定，可类型推断
```

区别：
- `const` 必须在编译期确定值，必须标注类型
- `let` 可以是运行时计算的值，类型可推断
- `const` 可以在任何作用域声明（包括全局）

### 2. 基本数据类型

**整数类型**

| 类型 | 长度 | 范围 |
|------|------|------|
| `i8` | 8-bit | -128 ~ 127 |
| `i16` | 16-bit | -32,768 ~ 32,767 |
| `i32` | 32-bit | -2³¹ ~ 2³¹-1 |
| `i64` | 64-bit | -2⁶³ ~ 2⁶³-1 |
| `isize` | 架构相关 | 32/64-bit |
| `u8` | 8-bit | 0 ~ 255 |
| `u32` | 32-bit | 0 ~ 2³²-1 |
| `usize` | 架构相关 | 用于索引 |

**命名解释**：
- `i` = integer（有符号整数）
- `u` = unsigned（无符号整数）
- 数字 = 位数
- `size` = 与平台指针大小相同

```rust
let count: i32 = 42;       // 明确标注类型
let count = 42;            // 默认推断为 i32
let count = 42_i64;        // 后缀指定类型
let big = 1_000_000;       // 下划线增加可读性
```

**与 Java 对比**：

| Java | Rust | 说明 |
|------|------|------|
| `int` | `i32` | 32 位有符号 |
| `long` | `i64` | 64 位有符号 |
| 无对应 | `usize` | 用于数组索引 |

Java 没有无符号类型（直到 Java 8 引入一些支持），Rust 有完整的无符号类型族。

**浮点类型**

```rust
let x = 2.0;      // f64（默认）
let y: f32 = 3.0; // f32
```

**布尔类型**

```rust
let t = true;
let f: bool = false;
```

**字符类型**

```rust
let c = 'z';
let emoji = '😀';  // Rust 的 char 是 Unicode 标量值，4 字节
```

**注意**：Rust 的 `char` 是 4 字节 Unicode 标量值，与 Java 的 2 字节 UTF-16 不同。

### 3. 类型推断

Rust 有强大的类型推断（Type Inference）：

```rust
let x = 5;              // 推断为 i32
let y = 5.0;            // 推断为 f64
let v = vec![1, 2, 3];  // 推断为 Vec<i32>
```

**与 Java 对比**：

Java（较新版本）有 `var` 关键字，但 Rust 的类型推断更强大：

```java
// Java
var list = new ArrayList<String>();  // 只能推断局部变量
```

```rust
// Rust - 可以从后续使用推断
let mut v = Vec::new();  // 此时还不知道类型
v.push(1);               // 现在推断出 Vec<i32>
```

Rust 可以从变量的使用方式"回溯"推断类型，Java 只能从初始化表达式推断。

### 4. 字符串初步：&str 与 String

**这是 Java 开发者学 Rust 的第一个困惑点**。

在 Java 中，字符串就是 `String`，简单明了。但 Rust 有两种主要的字符串类型：

| 类型 | 存储位置 | 是否可变 | 所有权 |
|------|---------|---------|-------|
| `&str` | 通常在程序二进制中 | 不可变 | 借用 |
| `String` | 堆上 | 可变（如果 mut） | 拥有 |

**命名解释**：
- `str`：string slice（字符串切片）的缩写
- `&str`：对字符串切片的引用（借用）
- `String`：拥有所有权的、可增长的字符串

**简化理解**（后续章节会深入）：

```rust
let s1 = "Hello";           // &str - 字符串字面量，硬编码在程序中
let s2 = String::from("Hello");  // String - 堆上分配，可修改

let s3 = "Hello".to_string();    // &str -> String
let s4: &str = &s2;              // String -> &str（借用）
```

**为什么 Rust 要这样设计？**

这涉及所有权系统（第 4-5 章详解）。简单说：
- `&str` 是"视图"，不拥有数据，不能独立存在
- `String` 拥有数据，负责内存管理
- 这种区分让内存管理更精确，避免不必要的复制

**现阶段的使用建议**：
- 字符串字面量：用 `&str`
- 需要拥有/修改的字符串：用 `String`
- 需要从 `&str` 转换：`.to_string()` 或 `String::from()`

---

## 逐步实现 word-count

### 步骤 1：读取标准输入

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("Read: {}", line);
    }
}
```

**命名解释**：
- `io`：input/output，输入输出模块
- `stdin`：standard input，标准输入
- `BufRead`：Buffered Read trait，带缓冲读取的特性
- `lock()`：获取标准输入的锁，以便逐行读取
- `lines()`：返回按行分割的迭代器

**为什么需要 `lock()`？**

标准输入在多线程环境中是共享资源，`lock()` 获取独占访问。单线程程序中这是惯用写法。

### 步骤 2：统计行数

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count = 0;  // 可变变量

    for line in stdin.lock().lines() {
        let _line = line.unwrap();  // 前缀 _ 表示故意不用这个变量
        line_count += 1;
    }

    println!("Lines: {}", line_count);
}
```

测试：

```bash
$ echo -e "Hello\nWorld" | cargo run
Lines: 2
```

### 步骤 3：统计单词数

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count = 0;
    let mut word_count = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        line_count += 1;

        // split_whitespace() 按空白字符分割，返回迭代器
        // count() 计算迭代器元素数量
        word_count += line.split_whitespace().count();
    }

    println!("Lines: {}, Words: {}", line_count, word_count);
}
```

**命名解释**：
- `split_whitespace()`：按空白字符（空格、制表符等）分割
- `count()`：计算迭代器中元素的数量

### 步骤 4：统计字符数

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count = 0;
    let mut word_count = 0;
    let mut char_count = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        line_count += 1;
        word_count += line.split_whitespace().count();
        char_count += line.len() + 1;  // +1 是换行符
    }

    println!("{} {} {}", line_count, word_count, char_count);
}
```

**注意**：`line.len()` 返回的是字节数，不是字符数！对于纯 ASCII 文本这没问题，但对于中文等 UTF-8 字符会不准确。

**正确统计 Unicode 字符**：

```rust
char_count += line.chars().count() + 1;  // chars() 按 Unicode 字符迭代
```

### 步骤 5：格式化对齐输出

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count: usize = 0;
    let mut word_count: usize = 0;
    let mut char_count: usize = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        line_count += 1;
        word_count += line.split_whitespace().count();
        char_count += line.chars().count() + 1;  // +1 是换行符
    }

    // 格式化输出：右对齐，宽度 8
    println!("{:>8}{:>8}{:>8}", line_count, word_count, char_count);
}
```

**格式化语法**：
- `{:>8}`：右对齐（`>`），宽度 8
- `{:<8}`：左对齐（`<`），宽度 8
- `{:^8}`：居中（`^`），宽度 8
- `{:08}`：用 0 填充到 8 位

### 完整代码

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut line_count: usize = 0;
    let mut word_count: usize = 0;
    let mut char_count: usize = 0;

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        line_count += 1;
        word_count += line.split_whitespace().count();
        char_count += line.chars().count() + 1;
    }

    // 如果输入为空，不输出换行符计数
    if line_count > 0 {
        // 最后一行可能没有换行符，减去多加的 1
        char_count -= 1;
    }

    println!("{:>8}{:>8}{:>8}", line_count, word_count, char_count);
}
```

---

## 运行与测试

```bash
# 构建
cargo build

# 测试英文
$ echo "Hello World" | cargo run
       1       2      11

# 测试多行
$ echo -e "Hello\nWorld" | cargo run
       2       2      11

# 测试中文（注意字符计数）
$ echo "你好世界" | cargo run
       1       1       4

# 与系统 wc 对比
$ echo "Hello World" | wc
       1       2      12    # wc 把换行符也算进去了
```

**注意**：我们的实现和系统 `wc` 在字符计数上有细微差异，因为对换行符的处理不同。这是正常的。

---

## 与 Java 对比

Java 实现相同功能：

```java
import java.util.Scanner;

public class WordCount {
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        int lineCount = 0;
        int wordCount = 0;
        int charCount = 0;

        while (scanner.hasNextLine()) {
            String line = scanner.nextLine();
            lineCount++;
            wordCount += line.trim().isEmpty() ? 0 : line.trim().split("\\s+").length;
            charCount += line.length() + 1;
        }

        System.out.printf("%8d%8d%8d%n", lineCount, wordCount, charCount);
    }
}
```

**关键差异**：

| 方面 | Java | Rust |
|------|------|------|
| 默认可变性 | 可变 | 不可变 |
| 字符串分割 | `split("\\s+")` 返回数组 | `split_whitespace()` 返回迭代器 |
| 类型推断 | 需要声明类型或用 var | 自动推断 |
| 字符串类型 | 只有 String | &str 和 String |

---

## 要点回顾

1. **Rust 变量默认不可变**
   - 用 `let` 声明不可变绑定
   - 用 `let mut` 声明可变绑定
   - 这是刻意的设计，减少 bug

2. **类型推断很强大**
   - 大多数情况不需要标注类型
   - 编译器能从使用方式推断

3. **两种字符串类型**
   - `&str`：字符串切片，借用
   - `String`：拥有所有权的字符串
   - 细节在所有权章节深入

4. **格式化输出**
   - `{:>8}` 右对齐 8 位
   - `{:<8}` 左对齐 8 位

---

## 最佳实践

### 变量命名

| 场景 | 推荐 | 不推荐 |
|------|------|--------|
| 一般变量 | `snake_case` | `camelCase` |
| 常量 | `SCREAMING_SNAKE_CASE` | `snake_case` |
| 未使用变量 | `_name` | 让编译器警告 |

### 何时使用 mut？

- **需要时才用**：不要习惯性地加 `mut`
- **编译器会提示**：如果需要但没加，编译器会告诉你
- **思考是否必要**：有时可以用不可变 + 遮蔽（shadowing）替代

```rust
// 遮蔽（shadowing）- 创建新绑定，而不是修改
let x = 5;
let x = x + 1;  // 创建新的 x，旧的被遮蔽
```

### 类型标注

- **让编译器推断**：大多数情况不需要手动标注
- **模糊时标注**：当编译器无法推断时再标注
- **公共 API 标注**：函数签名要明确标注类型

### 常见新手错误

1. **混淆 `&str` 和 `String`**：
   ```rust
   let s: String = "hello";  // 错误！字面量是 &str
   let s: String = "hello".to_string();  // 正确
   let s: String = String::from("hello");  // 也正确
   ```

2. **忘记 mut**：
   ```rust
   let count = 0;
   count += 1;  // 错误！count 是不可变的
   ```

3. **类型后缀混淆**：
   ```rust
   let x = 42_i32;   // 正确：i32 类型
   let x = 42_32;    // 错误：不是有效的后缀
   ```

---

## 练习

### 练习 1：添加 -l/-w/-c 选项

让程序支持只输出特定统计项：

```bash
$ echo "Hello World" | word-count -l
       1
$ echo "Hello World" | word-count -w
       2
$ echo "Hello World" | word-count -c
      11
```

### 练习 2：支持多个选项组合

```bash
$ echo "Hello World" | word-count -lw
       1       2
```

提示：使用我们在第 1 章学到的命令行参数处理。

---

## 扩展阅读

- [Rust Book: Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- [Rust Book: Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)

---

## 下一章预告

我们的 word-count 能工作了，但所有代码都在 `main.rs` 里。下一章，我们将学习 Rust 的模块系统，把代码组织成清晰的结构，并添加从文件读取的功能。

真实项目不可能把所有代码都塞进一个文件——是时候学习如何组织代码了。
