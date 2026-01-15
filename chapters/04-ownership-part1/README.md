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

实现 uniq-rs 时，会自然接触到所有权的核心概念：
- 比较两行时需要访问同一个变量
- 存储"上一行"涉及所有权转移
- 这些场景是学习所有权的最佳切入点

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

**为什么 Rust 不自动 clone？**

Rust 的原则：**你不为你不用的东西付费**。
- 移动（move）是零成本——只是指针赋值
- 克隆（clone）有运行时开销——内存分配、数据复制
- 如果语言偷偷帮你 clone，你可能不知道每次赋值都在"花钱"

所以 Rust 选择：显式优于隐式。想 clone 就自己写 `.clone()`。

**为什么 `vec.push(item)` 要拿走 item 的所有权？**

```rust
let mut v = Vec::new();
let s = String::from("hello");
v.push(s);  // s 的所有权移动到 Vec 里
// s 不能再用了
```

因为 Vec 要**长期持有**这个数据。如果只是借用：
- `s` 的原所有者可能先离开作用域
- `s` 被释放后，Vec 里就是悬垂指针

**规则**：谁要长期存储数据，谁就要拥有所有权。

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

### 6. 方法签名：self vs &self

**如何知道调用方法后原值还能不能用？**

看方法签名：

```rust
fn unwrap(self) -> T        // self = 消费，调用后原值没了
fn len(&self) -> usize      // &self = 借用，调用后原值还在
fn push(&mut self, item: T) // &mut self = 可变借用，原值还在
```

这就是为什么：
```rust
let result: Result<String, Error> = Ok("hello".to_string());
let s = result.unwrap();  // unwrap 消费了 result
// result 不能再用了

let s = String::from("hello");
let len = s.len();        // len 只是借用
println!("{}", s);        // s 还能用
```

**命名惯例提示**：
- `into_*` 方法通常消费 self（如 `into_bytes()`、`into_iter()`）
- `as_*` 方法通常借用（如 `as_bytes()`、`as_str()`）
- `to_*` 方法通常借用并返回新值（如 `to_string()`、`to_vec()`）

**为什么要设计"消耗 self"的方法？**

初学者可能觉得"调用方法把自己消耗了"很奇怪——大多数方法确实用 `&self` 借用。但消耗 self 是 Rust 精心设计的功能，有几个重要场景：

**1. Builder 模式（链式调用）**

```rust
let request = RequestBuilder::new()
    .url("https://example.com")    // 消耗旧 self，返回新 self
    .header("Auth", "token")
    .build();                       // 最终消耗 builder，生成 Request
```

**2. 类型状态转换**

```rust
// 订单状态机
impl Unpaid {
    fn pay(self, amount: u64) -> Paid {  // 消耗 Unpaid，变成 Paid
        Paid { amount }
    }
}
// 付款后，Unpaid 不存在了——编译器强制你不能再用它！
```

**3. 资源的最终转换**

```rust
let bytes: Vec<u8> = my_string.into_bytes();  // String 变成字节数组
// my_string 已被消耗，不能再用
```

**为什么这是好设计？**

1. **防止误用**：转换后原对象不能再用，编译器帮你检查
2. **零成本**：不需要克隆，直接移动内存
3. **语义清晰**：`into_*` 命名明确告诉调用者"这会消耗资源"

### 7. 借用返回值与原值的关系

上面提到 `into_bytes()` 会消耗原 String。那如果你不想消耗原字符串，又想获取字节数据呢？

**三种获取字节的方式**

| 你想要的 | 方法 | 代价 |
|---------|------|------|
| 只读一下字节 | `as_bytes()` → `&[u8]` | 零成本 |
| 要新的 Vec，保留原 String | `as_bytes().to_vec()` | 一次内存分配 |
| 要 Vec，不再需要 String | `into_bytes()` | 零成本（直接移动）|

```rust
// 方式 1：只是借用
let s = String::from("hello");
let bytes: &[u8] = s.as_bytes();
println!("{}", s);  // ✅ s 还在

// 方式 2：复制一份
let s2 = String::from("hello");
let copied: Vec<u8> = s2.as_bytes().to_vec();
println!("{}", s2);  // ✅ s2 还在

// 方式 3：消耗原值
let s3 = String::from("hello");
let owned: Vec<u8> = s3.into_bytes();
// println!("{}", s3);  // ❌ s3 没了
```

**`as_bytes()` 返回后，`bytes` 和 `s` 是什么关系？**

`bytes` 实际上是指向 `s` 内部内存的一个"窗口"，它们**共享同一块内存**：

```
s: String
┌──────────────────┐
│ ptr ─────────────┼──────► [ h | e | l | l | o ]  ← 堆上的实际数据
│ len: 5           │                 ▲
│ capacity: 5      │                 │
└──────────────────┘                 │
                                     │
bytes: &[u8]                         │
┌──────────────────┐                 │
│ ptr ─────────────┼─────────────────┘  (指向同一块内存)
│ len: 5           │
└──────────────────┘
```

**借用期间，原值被"锁住"**

因为 `bytes` 借用着 `s`，在借用期间 `s` 受到限制：

```rust
// ✅ 可以读取 s
let s = String::from("hello");
let bytes = s.as_bytes();
println!("{}", s);  // 读取没问题
println!("{:?}", bytes);
```

```rust
// ❌ 不能修改 s
let mut s = String::from("hello");
let bytes = s.as_bytes();  // 开始借用
s.push_str(" world");  // 编译错误！bytes 还借用着它
println!("{:?}", bytes);
```

```rust
// ❌ 不能移动/消耗 s
let s = String::from("hello");
let bytes = s.as_bytes();
drop(s);  // 编译错误！不能在借用期间丢弃
println!("{:?}", bytes);
```

**为什么不能在借用期间修改？**

想象如果允许修改：

```rust
let mut s = String::from("hello");
let bytes = s.as_bytes();  // bytes 指向 "hello" 的内存

s.push_str(" world");      // String 可能重新分配内存！
                           // 原来的内存可能被释放了

println!("{:?}", bytes);   // 💥 bytes 指向已释放的内存！
```

Rust 的借用规则就是为了在编译期阻止这种危险。

**借用什么时候结束？**

编译器会追踪 `bytes` 的**最后一次使用**：

```rust
let mut s = String::from("hello");
let bytes = s.as_bytes();
println!("{:?}", bytes);  // bytes 最后一次使用
                          // ↓ 从这里开始，借用结束

s.push_str(" world");     // ✅ 现在可以修改了
println!("{}", s);
```

这个特性叫 **NLL（Non-Lexical Lifetimes）**，我们会在下一章详细讲解。

**Rust 的设计哲学**

其他语言可能这样写：

```java
// Java
byte[] bytes = s.getBytes();  // 复制了？没复制？谁知道呢
```

Rust 强迫你明确选择：

```rust
// Rust
let bytes = s.into_bytes();      // 我知道 s 没了
let bytes = s.as_bytes().to_vec(); // 我知道这会复制
```

所以当你发现 `into_*` 会消耗所有权时，Rust 其实在问你：

> "你真的不要原来那个了吗？还是你其实想要 `as_*` 或 `to_*`？"

这就是 API 设计在帮你思考。

---

## 逐步实现 uniq-rs

### 需求回顾

我们要实现一个类 Unix `uniq` 的去重工具：

**核心功能**：
- 从标准输入逐行读取文本
- 去除**连续**重复的行（非连续的重复不处理）
- 将结果输出到标准输出

**输入输出示例**：
```
输入:           输出:
apple           apple
apple     →     banana
banana          apple
banana
banana
apple
```

**技术要点**：
- 需要"记住"上一行，才能与当前行比较
- 这会触发所有权问题——正是我们要学习的内容

现在让我们一步步实现。

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

这样可以工作。每次循环都把 `line` 移动到 `prev_line`，这是正确的做法。

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

提示：需要一个计数器变量。这个练习会在第 5 章详细实现。

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
