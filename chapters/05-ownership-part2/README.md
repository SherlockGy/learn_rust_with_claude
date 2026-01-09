# 第 5 章：所有权（下）

## 本章目标

学完本章，你将能够：
- 使用可变引用修改借用的数据
- 理解借用规则及其背后的原因
- 初步了解生命周期的概念
- 写出安全且高效的 Rust 代码

---

## 前置知识

- 第 4 章：所有权基础、移动、不可变借用

---

## 项目：升级 uniq-rs 支持计数

### 本章目标

为 uniq-rs 添加 `-c` 选项，显示每行重复的次数：

```bash
$ cat data.txt | uniq-rs -c
      2 apple
      3 banana
      1 apple
```

这个功能需要**可变引用**：我们要在循环中累加计数器。

---

## 核心概念

### 1. 可变引用

上一章我们学了不可变引用 `&T`，只能读取。如果需要修改数据，使用可变引用 `&mut T`：

```rust
fn main() {
    let mut s = String::from("hello");
    append_world(&mut s);  // 传递可变引用
    println!("{}", s);     // 输出: hello world
}

fn append_world(s: &mut String) {
    s.push_str(" world");
}
```

**语法要点**：
- 变量必须声明为 `mut`
- 创建可变引用用 `&mut`
- 函数参数类型是 `&mut T`

**命名解释**：`&mut` 是 `& mutable` 的组合，表示"可变引用"。

### 2. 借用规则

Rust 有严格的借用规则：

> **在任意给定时刻，你只能拥有以下其中之一：**
> - 一个可变引用
> - 任意数量的不可变引用

**为什么有这个规则？**

防止数据竞争。数据竞争发生在：
1. 两个或更多指针同时访问同一数据
2. 至少一个指针在写入
3. 没有同步机制

Rust 在编译期就阻止这种情况。

**规则示例**：

```rust
// 可以：多个不可变引用
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{}, {}", r1, r2);

// 可以：一个可变引用
let mut s = String::from("hello");
let r = &mut s;
r.push_str(" world");

// 不可以：同时存在可变和不可变引用
let mut s = String::from("hello");
let r1 = &s;        // 不可变引用
let r2 = &mut s;    // 错误！已有不可变引用
println!("{}", r1);

// 不可以：多个可变引用
let mut s = String::from("hello");
let r1 = &mut s;
let r2 = &mut s;    // 错误！已有可变引用
```

### 3. 非词法作用域生命周期（NLL）

好消息：Rust 编译器很聪明。引用的作用域从声明开始，到**最后一次使用**结束：

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;
println!("{}, {}", r1, r2);
// r1 和 r2 在这里不再使用，作用域结束

let r3 = &mut s;  // 可以！r1, r2 已结束
r3.push_str(" world");
println!("{}", r3);
```

这个特性叫 **NLL（Non-Lexical Lifetimes）**，让借用规则更灵活。

**命名解释**：Non-Lexical Lifetimes = 非词法作用域生命周期。"词法"指代码的文本结构（如 `{}`），NLL 让编译器能更智能地分析引用的实际使用范围。

### 4. 生命周期初步

**什么是生命周期？**

生命周期是 Rust 确保引用有效的方式。每个引用都有一个生命周期——它有效的范围。

```rust
{
    let r;                  // r 的生命周期开始
    {
        let x = 5;          // x 的生命周期开始
        r = &x;             // r 引用 x
    }                       // x 的生命周期结束
    println!("{}", r);      // 错误！r 引用了无效的 x
}
```

编译器会检测这种"悬垂引用"并拒绝编译。

**大多数时候不需要手动标注**

Rust 编译器有"生命周期省略规则"，大部分情况会自动推断：

```rust
// 不需要标注
fn first_word(s: &str) -> &str {
    // 编译器推断：返回值的生命周期与参数相同
}
```

**本章边界**：
- 本章必修内容：生命周期是什么、为什么需要
- 显式标注语法（`'a`）：下文有简单示例，本章「深入理解」小节有完整讲解
- 后续章节会按需使用，不再专门讲解

**结构体存引用的情况**

如果结构体要存引用，必须声明生命周期：

```rust
// 错误：缺少生命周期标注
struct Person {
    name: &str,  // ❌ 编译错误
}

// 正确：声明生命周期
struct Person<'a> {
    name: &'a str,  // ✅ 'a 表示：name 引用的数据必须比 Person 活得久
}
```

为什么？编译器需要确保：Person 实例不会比它引用的字符串活得更久，否则就会出现悬垂引用。

**现阶段建议**：如果不确定，结构体字段用 `String`（拥有所有权）而不是 `&str`（借用）。

> 想深入理解 `'a` 的本质？继续阅读下方的「深入理解」小节。

### 深入理解：生命周期标注的本质

> 💡 **阅读建议**：本节解释生命周期标注的核心原理。如果你刚接触 Rust，可以先跳过，等实际遇到 `'a` 标注时再回来阅读——本节是教程中关于这个话题最详细的讲解。

当你看到这样的代码时：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

你可能会困惑：这个 `'a` 到底是什么？是一种"提醒"？还是一种"描述"？编译器又是如何使用它的？

**核心答案：生命周期标注是约束声明（Constraint Declaration）**

`'a` **不是**告诉编译器这些引用实际能活多久，**而是**声明一个约束规则，让编译器去验证代码是否满足这个规则。

上面的代码声明的约束是：
> "返回值的生命周期不能超过 x 和 y 中较短的那个"

编译器会在**两个地方**使用这个约束进行验证：

**1. 函数体内（实现侧）**：检查返回值是否真的来自标注了 `'a` 的参数

```rust
// 能通过：返回值确实来自 'a 标注的输入
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }  // ✅ x 或 y 都有 'a
}

// 不能通过：返回值不是来自 'a 标注的输入
fn bad<'a>(x: &'a str, y: &'a str) -> &'a str {
    let s = String::from("hello");
    &s  // ❌ 错误！返回值来自局部变量，不是 'a 标注的输入
}
```

**2. 调用点（使用侧）**：检查调用方是否满足这个约束

```rust
let result;
{
    let s1 = String::from("hello");
    let s2 = String::from("world");
    result = longest(&s1, &s2);  // 编译器推断：'a 是 s1 和 s2 生命周期的交集
}  // s1, s2 在这里死亡
println!("{}", result);  // ❌ 错误！result 可能指向已死亡的数据
```

**类比理解**

生命周期标注类似于泛型约束：

```rust
fn foo<T: Clone>(x: T)      // T 必须实现 Clone
fn bar<'a>(x: &'a str)      // 引用 x 必须在 'a 内有效
```

两者都是告诉编译器"这里有一个约束，请帮我检查调用方是否满足"。

#### 快速记忆口诀

一条核心规则可以概括所有生命周期场景：

> **借来的东西，不能比主人活得久**

或者更技术地说：**引用的生命周期 ≤ 数据的生命周期**

#### 函数与结构体的约束对象

这里是最容易困惑的地方——同样是 `'a`，在函数和结构体中"约束谁"是不同的：

| 场景 | 被约束的对象 | 辅助绑定 |
|------|-------------|---------|
| **函数** | 返回值 | 输入参数 |
| **结构体** | 结构体实例 | 字段中的引用 |

**函数：返回值 ≤ 输入参数**

```rust
fn get_ref<'a>(x: &'a str) -> &'a str
//             ↑ 数据来源      ↑ 返回的引用
//        约束：返回值不能比输入活得久
```

当有多个同名生命周期参数时，返回值必须 ≤ 所有输入中最短的那个：

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str
// 返回值可能来自 x 或 y，所以必须 ≤ min(x的生命周期, y的生命周期)
```

**结构体：结构体实例 ≤ 它借用的数据**

```rust
struct Holder<'a> {
    data: &'a str,  // Holder 借用了别人的 str
}
// 约束：Holder 实例不能比它借的 str 活得久
```

实际例子：

```rust
let holder;
{
    let s = String::from("hi");
    holder = Holder { data: &s };
}  // s 死了
// holder 还想用？不行！借的东西没了
```

#### 三句话总结

1. **生命周期标注是约束声明**，不是描述实际生命周期
2. **编译器用它来验证**代码是否安全，分别检查实现侧和调用侧
3. **记忆口诀**：借来的东西不能比主人活得久——函数约束返回值，结构体约束实例

### 5. 借用规则的实际意义

**场景：迭代时修改集合**

```rust
let mut v = vec![1, 2, 3];

// 错误！不能在迭代时修改
for item in &v {
    v.push(*item * 2);  // 错误！已有不可变借用
}
```

**为什么这是问题？**

想象 `v` 内部用数组实现，`push` 可能需要扩容（分配新内存、复制数据）。如果迭代器还在用旧内存，就会访问无效数据。

**解决方案**：

```rust
let mut v = vec![1, 2, 3];
let mut new_items = Vec::new();

for item in &v {
    new_items.push(*item * 2);
}

v.extend(new_items);
```

---

## 逐步实现 uniq-rs -c

### 步骤 1：解析 -c 参数

```rust
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    let count_mode = args.len() > 1 && args[1] == "-c";

    // ... 后续逻辑
}
```

### 步骤 2：添加计数逻辑

```rust
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    let count_mode = args.len() > 1 && args[1] == "-c";

    let stdin = io::stdin();
    let mut prev_line = String::new();
    let mut count: usize = 0;
    let mut first = true;

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if first {
            prev_line = line;
            count = 1;
            first = false;
        } else if line == prev_line {
            count += 1;  // 需要可变引用来修改 count
        } else {
            print_line(&prev_line, count, count_mode);
            prev_line = line;
            count = 1;
        }
    }

    // 输出最后一组
    if !first {
        print_line(&prev_line, count, count_mode);
    }
}

fn print_line(line: &str, count: usize, count_mode: bool) {
    if count_mode {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}
```

**代码分析**：
- `count += 1`：这里 `count` 是可变变量，直接修改
- `print_line(&prev_line, ...)`: 借用 `prev_line` 打印
- 借用只发生在 `print_line` 调用期间，之后可以修改 `prev_line`

### 步骤 3：理解借用时机

让我们分析关键行：

```rust
print_line(&prev_line, count, count_mode);
prev_line = line;  // 这行能工作吗？
```

能！因为：
1. `&prev_line` 在 `print_line` 调用时借用
2. `print_line` 返回后，借用结束
3. 然后才执行 `prev_line = line`（移动）

NLL 让编译器能理解这种"借用-然后-修改"的模式。

### 完整代码

```rust
use std::env;
use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = env::args().collect();
    let count_mode = args.len() > 1 && args[1] == "-c";

    let stdin = io::stdin();
    let mut prev_line = String::new();
    let mut count: usize = 0;
    let mut first = true;

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if first {
            prev_line = line;
            count = 1;
            first = false;
        } else if line == prev_line {
            count += 1;
        } else {
            print_line(&prev_line, count, count_mode);
            prev_line = line;
            count = 1;
        }
    }

    // 输出最后一组
    if !first {
        print_line(&prev_line, count, count_mode);
    }
}

fn print_line(line: &str, count: usize, count_mode: bool) {
    if count_mode {
        println!("{:>7} {}", count, line);
    } else {
        println!("{}", line);
    }
}
```

---

## 运行与测试

```bash
# 无选项（基本去重）
$ echo -e "apple\napple\nbanana\nbanana\nbanana\napple" | cargo run
apple
banana
apple

# -c 选项（显示计数）
$ echo -e "apple\napple\nbanana\nbanana\nbanana\napple" | cargo run -- -c
      2 apple
      3 banana
      1 apple

# 空输入
$ echo -n "" | cargo run -- -c
（无输出）

# 单行
$ echo "single" | cargo run -- -c
      1 single
```

---

## 与 Java 对比

### 可变性控制

```java
// Java - 任意时刻都可以修改
List<String> list = new ArrayList<>();
for (String s : list) {
    list.add(s);  // 运行时 ConcurrentModificationException
}
```

```rust
// Rust - 编译期阻止
let mut v = vec!["a".to_string()];
for s in &v {
    v.push(s.clone());  // 编译错误！
}
```

Rust 在**编译期**就发现问题，Java 需要**运行时**才会报错。

### 引用安全

```java
// Java - 可能返回无效引用（逻辑错误）
class Container {
    private String data;

    public String getData() {
        return data;  // 可能返回 null
    }

    public void clearData() {
        data = null;  // getData() 返回的引用变得"悬垂"
    }
}
```

```rust
// Rust - 编译器保证引用有效
struct Container {
    data: String,
}

impl Container {
    fn get_data(&self) -> &str {
        &self.data  // 保证有效
    }
    // 无法在借用期间清除 data
}
```

---

## 要点回顾

1. **可变引用**
   - `&mut T` 允许修改借用的值
   - 变量必须声明为 `mut`

2. **借用规则**
   - 同一时刻：一个可变引用 **或** 多个不可变引用
   - 防止数据竞争

3. **NLL**
   - 引用的作用域到最后使用处结束
   - 不是到 `}` 结束

4. **生命周期**
   - 确保引用始终有效
   - 大多数情况自动推断
   - 显式标注 `'a` 是**约束声明**（详见本章「深入理解」小节）

---

## 最佳实践

### 借用范围最小化

```rust
// 好：尽早结束借用
let mut data = String::from("hello");
{
    let len = data.len();  // 借用发生在这
}  // 借用结束
data.push_str(" world");  // 可以修改

// 也好：利用 NLL
let mut data = String::from("hello");
let len = data.len();  // 借用
// len 使用结束后，借用自动结束
data.push_str(" world");  // 可以修改
```

### 优先使用不可变引用

```rust
// 好：只需读取时用 &
fn print_info(s: &String) { ... }

// 只有需要修改时才用 &mut
fn append(s: &mut String) { ... }
```

### 避免不必要的可变性

```rust
// 不好：不需要 mut
let mut x = 5;  // 警告：变量不需要可变

// 好
let x = 5;
```

### 常见新手错误

1. **同时借用可变和不可变**：
   ```rust
   let mut s = String::from("hello");
   let r1 = &s;
   let r2 = &mut s;  // 错误！
   println!("{}", r1);
   ```

2. **在借用期间移动所有权**：
   ```rust
   let s = String::from("hello");
   let r = &s;
   let s2 = s;        // 错误！s 被借用中
   println!("{}", r);
   ```

3. **返回局部变量的引用**：
   ```rust
   fn bad() -> &String {
       let s = String::from("hello");
       &s  // 错误！s 在函数结束时销毁
   }
   ```

---

## 练习

### 练习 1：修复借用错误

以下代码有借用错误，请修复：

```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &s;
    s.push_str(" world");
    println!("{}", r1);
}
```

### 练习 2：添加 -d 选项

为 uniq-rs 添加 `-d` 选项，只输出重复的行（出现 2 次以上）：

```bash
$ echo -e "apple\napple\nbanana\napple" | uniq-rs -d
apple
```

### 练习 3：支持文件输入

让 uniq-rs 支持从文件读取：

```bash
$ uniq-rs data.txt
$ uniq-rs -c data.txt
```

---

## 扩展阅读

- [Rust Book: References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Understanding Rust's Borrow Checker](https://blog.logrocket.com/introducing-rust-borrow-checker/)

---

## 下一章预告

我们已经掌握了 Rust 最核心的所有权系统。现在是时候开始构建更复杂的数据结构了。

下一章，我们将学习**结构体（Struct）**，并开始构建 task-cli——一个贯穿多章的命令行待办事项管理器。

准备好创建你自己的数据类型了吗？
