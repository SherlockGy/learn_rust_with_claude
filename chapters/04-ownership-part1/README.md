# 第 4 章：所有权（上）

## 本章目标

学完本章，你将能够：
- 理解 Rust 所有权系统的核心规则
- 处理"值被移动"的编译错误
- 使用借用（引用）来共享数据
- 判断何时使用移动、借用或克隆

---

## 前置知识

- 第 1-3 章：Cargo、变量类型、函数与模块

---

## 为什么需要所有权？

在深入语法之前，让我们理解 **为什么** Rust 需要所有权系统。

### 内存管理的困境

**Java 的方式：垃圾回收（GC）**

```java
String s = new String("hello");
// ... 使用 s
// 不用管，GC 会自动回收
```

优点：简单，不用手动管理
缺点：GC 暂停、内存开销、不确定的回收时机

**C/C++ 的方式：手动管理**

```c
char* s = malloc(6);
strcpy(s, "hello");
// ... 使用 s
free(s);  // 必须手动释放
// 但如果忘了 free？→ 内存泄漏
// 如果 free 两次？→ 崩溃
// 如果 free 后还使用？→ 未定义行为
```

优点：高性能、可控
缺点：容易出错，bug 难以追踪

**Rust 的方式：所有权系统**

编译器在编译时追踪内存使用，既不需要 GC，也不需要手动释放。

```rust
let s = String::from("hello");
// ... 使用 s
// 离开作用域时自动释放，编译期保证安全
```

---

## 项目：uniq-rs - 去重工具

### 功能概览

`uniq` 是 Unix 经典工具，去除连续重复的行：

```bash
$ cat data.txt
apple
apple
banana
banana
banana
apple

$ cat data.txt | uniq
apple
banana
apple
```

**注意**：`uniq` 只去除**连续**重复，不是所有重复。

### 为什么选这个项目？

实现 uniq-rs 时，我们会自然遇到所有权问题：
- 比较两行需要访问同一个变量
- 存储"上一行"需要转移所有权
- 这些问题逼着我们理解所有权

### 最终效果

```bash
$ cat data.txt | uniq-rs
apple
banana
apple
```

---

## 核心概念

### 1. 所有权规则

Rust 的所有权系统基于三条规则：

> 1. **每个值都有一个所有者（owner）**
> 2. **同一时刻只能有一个所有者**
> 3. **当所有者离开作用域，值被丢弃**

让我们逐条理解。

**规则 1：每个值都有一个所有者**

```rust
let s = String::from("hello");  // s 是 "hello" 的所有者
```

**规则 2：同一时刻只能有一个所有者**

```rust
let s1 = String::from("hello");
let s2 = s1;  // 所有权从 s1 移动到 s2

println!("{}", s1);  // 错误！s1 不再有效
```

**规则 3：离开作用域时丢弃**

```rust
{
    let s = String::from("hello");
    // s 在这里有效
}  // s 离开作用域，内存被释放
```

### 2. 移动（Move）

当你把一个值赋给另一个变量时，所有权**移动**了：

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 的所有权移动到 s2
// s1 不再有效！
```

**为什么要移动而不是复制？**

考虑 `String` 的结构：

```
栈上（Stack）          堆上（Heap）
┌─────────────┐       ┌───────────────┐
│ ptr ───────────────→│ h e l l o     │
│ len: 5      │       └───────────────┘
│ capacity: 5 │
└─────────────┘
```

如果允许两个变量同时指向同一块堆内存：
- 两个变量离开作用域时都会尝试释放 → **双重释放**！

所以 Rust 选择**移动**语义：转移所有权，让原变量失效。

**与 Java 对比**：

```java
// Java - 都是引用，GC 追踪
String s1 = new String("hello");
String s2 = s1;  // 两个引用指向同一个对象
// 都有效，GC 负责管理
```

```rust
// Rust - 所有权移动
let s1 = String::from("hello");
let s2 = s1;  // 所有权移动
// s1 失效，编译器保证安全
```

### 3. 复制（Copy）

不是所有类型都会移动。简单类型会**复制**：

```rust
let x = 5;
let y = x;  // 复制，不是移动
println!("x = {}, y = {}", x, y);  // 都有效！
```

**哪些类型会 Copy？**

- 所有整数类型（i32, u64 等）
- 布尔类型（bool）
- 浮点类型（f32, f64）
- 字符类型（char）
- 只包含 Copy 类型的元组

**规律**：存储在栈上、大小固定的简单类型会 Copy。

**命名解释**：Copy 是一个 trait（特性），实现了 Copy 的类型在赋值时会自动复制。

### 4. 克隆（Clone）

如果确实需要深拷贝，使用 `clone()`：

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // 显式深拷贝
println!("s1 = {}, s2 = {}", s1, s2);  // 都有效！
```

`clone()` 会复制堆上的数据，两个变量各自拥有独立的内存。

```
s1                      堆
┌─────────────┐       ┌───────────────┐
│ ptr ───────────────→│ h e l l o     │
│ len: 5      │       └───────────────┘
│ capacity: 5 │
└─────────────┘

s2                      堆
┌─────────────┐       ┌───────────────┐
│ ptr ───────────────→│ h e l l o     │（独立的副本）
│ len: 5      │       └───────────────┘
│ capacity: 5 │
└─────────────┘
```

**Clone 的代价**：有运行时开销，特别是大数据结构。

### 5. 借用（Borrowing）

**问题**：如果只是想"看看"数据，不想获取所有权怎么办？

**答案**：借用！

```rust
let s = String::from("hello");
let len = calculate_length(&s);  // 借用 s
println!("s = {}, len = {}", s, len);  // s 仍然有效

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

**命名解释**：
- `&`：引用（reference）符号
- `&s`：创建一个指向 s 的引用
- `&String`：String 的引用类型

**借用 vs 移动**：

```rust
// 移动 - 获取所有权
fn take_ownership(s: String) {
    // 函数拥有 s
}

// 借用 - 只是借来看看
fn borrow(s: &String) {
    // 只读访问 s
}
```

**借用的规则**：

- 引用不拥有数据
- 引用必须始终有效（不能悬垂）
- 借用期间，所有者不能移动值

```
s: String               堆
┌─────────────┐       ┌───────────────┐
│ ptr ───────────────→│ h e l l o     │
│ len: 5      │       └───────────────┘
│ capacity: 5 │
└─────────────┘
      ↑
      │
&s: &String
┌─────────────┐
│ ptr         │ （指向 s，不拥有数据）
└─────────────┘
```

---

## 逐步实现 uniq-rs

### 步骤 1：基本框架

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        println!("{}", line);
    }
}
```

### 步骤 2：尝试记住上一行（遇到问题！）

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut prev_line = String::new();

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if line != prev_line {
            println!("{}", line);
            prev_line = line;  // 移动 line 到 prev_line
        }
        // 如果相等，什么也不输出（去重）
    }
}
```

这样可以工作，但有个问题：每次都在移动 `line`。

### 步骤 3：理解发生了什么

```rust
prev_line = line;  // line 的所有权移动到 prev_line
```

这是移动操作。在这个场景下没问题，因为我们之后不再使用 `line`。

但如果我们想在移动后还使用 `line` 呢？

```rust
if line != prev_line {
    prev_line = line;  // 移动
    println!("{}", line);  // 错误！line 已被移动
}
```

### 步骤 4：使用 clone 解决

```rust
if line != prev_line {
    println!("{}", line);
    prev_line = line.clone();  // 克隆，line 仍有效
}
```

这样可以工作，但 `clone()` 有开销。我们来优化。

### 步骤 5：更好的解决方案

仔细分析：`println!` 只需要借用，所以先打印再移动：

```rust
if line != prev_line {
    println!("{}", line);  // 只是借用
    prev_line = line;       // 然后移动，之后不再用 line
}
```

编译器很聪明，它知道 `println!` 只是借用，移动发生在之后。

### 完整代码

```rust
use std::io::{self, BufRead};

fn main() {
    let stdin = io::stdin();
    let mut prev_line = String::new();
    let mut first = true;

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if first || line != prev_line {
            println!("{}", line);
            prev_line = line;
            first = false;
        }
    }
}
```

**为什么需要 `first` 标志？**

第一行时，`prev_line` 是空字符串。如果第一行恰好也是空行，不加 `first` 会错误地跳过它。

---

## 函数与所有权

### 传参时的所有权

```rust
fn main() {
    let s = String::from("hello");

    takes_ownership(s);  // s 的所有权移动到函数

    // println!("{}", s);  // 错误！s 不再有效
}

fn takes_ownership(s: String) {
    println!("{}", s);
}  // s 离开作用域，内存释放
```

### 返回值的所有权

```rust
fn main() {
    let s1 = gives_ownership();  // 函数返回值的所有权移动到 s1

    let s2 = String::from("hello");
    let s3 = takes_and_gives_back(s2);  // s2 移入，返回值移到 s3
}

fn gives_ownership() -> String {
    String::from("hello")  // 返回值的所有权移出
}

fn takes_and_gives_back(s: String) -> String {
    s  // 返回，所有权移出
}
```

### 借用参数

如果只需要读取数据，用引用：

```rust
fn main() {
    let s = String::from("hello");
    let len = calculate_length(&s);  // 借用
    println!("length of '{}' is {}", s, len);  // s 仍有效
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

---

## 运行与测试

```bash
# 创建测试数据
echo -e "apple\napple\nbanana\nbanana\nbanana\napple" > data.txt

# 测试
$ cat data.txt | cargo run
apple
banana
apple

# 测试空输入
$ echo "" | cargo run
<空行>

# 测试全相同
$ echo -e "a\na\na" | cargo run
a
```

---

## 与 Java 对比

### 内存管理模型

| Java | Rust |
|------|------|
| 堆分配 + GC | 栈优先，堆按需 |
| 运行时追踪 | 编译期分析 |
| 可能 GC 暂停 | 无暂停 |
| 所有对象都是引用 | 值类型优先 |

### 对象赋值

```java
// Java - 复制引用
String s1 = new String("hello");
String s2 = s1;  // 两个引用，一个对象
// 都有效
```

```rust
// Rust - 移动所有权
let s1 = String::from("hello");
let s2 = s1;  // 所有权移动
// s1 失效
```

### 心智模型转换

**Java 开发者需要理解**：

1. Rust 的变量默认"拥有"数据，不只是"引用"
2. 赋值默认是移动，不是复制引用
3. 借用 (`&`) 才是类似 Java 引用的概念
4. 编译器会强制你正确处理所有权

---

## 要点回顾

1. **所有权三规则**
   - 每个值有一个所有者
   - 同时只能有一个所有者
   - 所有者离开作用域，值被丢弃

2. **移动 vs 复制**
   - `String` 等堆类型：移动
   - 整数等简单类型：复制

3. **借用**
   - `&T` 创建引用，不获取所有权
   - 借用允许读取但不拥有

4. **克隆**
   - `clone()` 深拷贝
   - 有运行时开销

---

## 最佳实践

### 何时用什么？

| 场景 | 推荐方式 | 说明 |
|------|---------|------|
| 只需读取数据 | 借用 `&T` | 最高效 |
| 需要修改数据 | 可变借用 `&mut T` | 下一章 |
| 需要拥有数据 | 移动 | 所有权转移 |
| 需要保留原数据 | `clone()` | 有开销 |

### 函数签名设计

```rust
// 好：只读取，用引用
fn print_info(s: &String) {
    println!("{}", s);
}

// 好：需要拥有，取所有权
fn consume(s: String) {
    // 会消耗 s
}

// 不好：不必要地取所有权
fn print_info_bad(s: String) {  // 应该用 &String
    println!("{}", s);
}
```

### 避免不必要的 clone

```rust
// 不好：不必要的 clone
let s = get_string();
let s2 = s.clone();  // 如果之后不用 s，这个 clone 浪费了
process(s2);

// 好：直接移动
let s = get_string();
process(s);
```

### 常见新手错误

1. **忘记值被移动了**：
   ```rust
   let s = String::from("hello");
   let s2 = s;
   println!("{}", s);  // 错误！s 已移动
   ```

2. **过度 clone**：
   ```rust
   fn process(s: &String) { ... }

   let s = String::from("hello");
   process(&s.clone());  // 不必要！直接 process(&s)
   ```

3. **混淆移动和借用**：
   ```rust
   fn foo(s: String) { ... }  // 取所有权
   fn bar(s: &String) { ... } // 借用

   let s = String::from("hello");
   foo(s);   // s 移动了
   bar(&s);  // 错误！s 已失效
   ```

---

## 练习

### 练习 1：修复编译错误

以下代码有编译错误，请修复：

```rust
fn main() {
    let s = String::from("hello");
    print_it(s);
    print_it(s);  // 错误！
}

fn print_it(s: String) {
    println!("{}", s);
}
```

### 练习 2：添加 -c 选项

为 uniq-rs 添加 `-c` 选项，显示重复次数：

```bash
$ cat data.txt | uniq-rs -c
      2 apple
      3 banana
      1 apple
```

提示：需要一个计数器变量。

---

## 扩展阅读

- [Rust Book: Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Visualizing Rust's Ownership](https://rufflewind.com/2017-02-15/rust-move-copy-borrow)

---

## 下一章预告

我们学会了基本的借用，但还有一个重要问题：如何通过借用**修改**数据？

```rust
fn append_world(s: &String) {
    s.push_str(" world");  // 错误！不能修改借用的值
}
```

下一章，我们将学习**可变借用**和借用规则的完整图景，完成对所有权系统的理解。
