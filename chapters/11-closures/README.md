# 第 11 章：闭包

## 本章目标

学完本章，你将能够：
- 定义和使用闭包
- 理解闭包的三种捕获模式
- 区分 Fn、FnMut、FnOnce trait
- 掌握闭包在实际代码中的应用场景
- 为 task-cli 添加灵活的过滤功能

---

## 前置知识

- 第 9 章：Trait（理解 trait bound）
- 第 10 章：泛型（泛型函数参数）

---

## 项目：为 task-cli 添加过滤功能

### 功能概览

为 task-cli 添加灵活的任务过滤功能，让用户可以按各种条件筛选任务。

### 最终效果

```bash
$ task list --status pending
# 只显示待办任务

$ task list --priority high
# 只显示高优先级任务

$ task list --status pending --priority high
# 组合过滤
```

### 为什么需要闭包？

如果用普通函数实现过滤：

```rust
// 需要为每种过滤条件写一个函数
fn filter_by_status(tasks: &[Task], status: Status) -> Vec<&Task> { ... }
fn filter_by_priority(tasks: &[Task], priority: Priority) -> Vec<&Task> { ... }
fn filter_by_status_and_priority(tasks: &[Task], status: Status, priority: Priority) -> Vec<&Task> { ... }
// 组合条件越多，函数越多...
```

用闭包：

```rust
// 一个通用函数，传入任意条件
fn filter_tasks<F>(tasks: &[Task], predicate: F) -> Vec<&Task>
where F: Fn(&Task) -> bool
{
    tasks.iter().filter(|t| predicate(t)).collect()
}

// 任意条件都能传入
let result = filter_tasks(&tasks, |t| t.status == Status::Pending && t.priority == Priority::High);
```

---

## 核心概念

### 1. 什么是闭包

**闭包**（Closure）是一种可以捕获其环境中变量的匿名函数。

```rust
// 普通函数
fn add(x: i32, y: i32) -> i32 {
    x + y
}

// 闭包
let add = |x: i32, y: i32| -> i32 { x + y };

// 简化形式（类型推断 + 单表达式省略花括号）
let add = |x, y| x + y;
```

**命名解释**：
- **Closure（闭包）**：源自数学中的"闭包"概念，表示函数"封闭"了对外部环境的引用
- 也叫 **Lambda 表达式**（Java/Python）或**匿名函数**

### 2. 闭包语法详解

```rust
// 完整形式
let closure = |参数: 类型, ...| -> 返回类型 {
    函数体
};

// 各种简化
let add = |x: i32, y: i32| -> i32 { x + y };  // 完整
let add = |x: i32, y: i32| x + y;              // 省略返回类型和花括号
let add = |x, y| x + y;                         // 省略所有类型（需要使用时推断）

// 无参数闭包
let say_hi = || println!("Hi!");

// 捕获环境变量
let name = "Alice";
let greet = || println!("Hello, {}!", name);  // 捕获 name
```

### 3. 闭包捕获环境

这是闭包最重要的特性——可以"记住"定义时的环境：

```rust
fn main() {
    let factor = 2;
    let multiply = |x| x * factor;  // 捕获 factor

    println!("{}", multiply(5));  // 10
    println!("{}", multiply(3));  // 6
}
```

**与 Java Lambda 对比**：

```java
// Java - 只能捕获 "effectively final" 的变量
int factor = 2;
// factor = 3;  // 编译错误！捕获后不能修改
Function<Integer, Integer> multiply = x -> x * factor;
```

```rust
// Rust - 可以捕获可变变量，但有借用规则限制
let mut count = 0;
let mut increment = || {
    count += 1;  // 可变借用 count
    count
};

println!("{}", increment());  // 1
println!("{}", increment());  // 2
// 闭包持有可变借用期间，不能有其他访问
```

### 4. 三种捕获模式与 Fn trait

Rust 编译器根据闭包如何使用捕获的变量，自动推断实现哪个 trait：

| 捕获方式 | 实现的 trait | 调用方式 | 说明 |
|---------|-------------|---------|------|
| 不可变借用 | `Fn` | `&self` | 可多次调用，不修改环境 |
| 可变借用 | `FnMut` | `&mut self` | 可多次调用，会修改环境 |
| 获取所有权 | `FnOnce` | `self` | 只能调用一次 |

**trait 继承关系**：
```
FnOnce      ← 所有闭包都实现
   ↑
FnMut       ← 不消耗环境的闭包实现
   ↑
Fn          ← 不修改环境的闭包实现
```

#### Fn - 不可变借用

```rust
let s = String::from("hello");
let print = || println!("{}", s);  // 不可变借用 s

print();  // 可以调用
print();  // 可以再次调用
println!("{}", s);  // s 仍然可用
```

#### FnMut - 可变借用

```rust
let mut total = 0;
let mut add_to = |x: i32| {
    total += x;  // 可变借用 total
};

add_to(5);
add_to(3);
println!("Total: {}", total);  // 8

// 注意：闭包本身需要声明为 mut
```

#### FnOnce - 获取所有权

```rust
let s = String::from("hello");
let consume = || {
    let moved = s;  // 移动 s 到闭包内部
    println!("{}", moved);
};

consume();
// consume();  // 错误！FnOnce 只能调用一次
// println!("{}", s);  // 错误！s 已被移动
```

### 5. move 关键字

`move` 强制闭包获取所有权，而不是借用：

```rust
let s = String::from("hello");

// 默认：借用
let closure1 = || println!("{}", s);

// move：获取所有权
let closure2 = move || println!("{}", s);
// println!("{}", s);  // 错误！s 已移动

// move 在跨线程时必须使用
use std::thread;
let s = String::from("hello");
let handle = thread::spawn(move || {
    println!("{}", s);  // s 必须 move 到新线程
});
```

**什么时候用 move？**
- 闭包需要比创建它的作用域存活更久（如传给线程）
- 需要闭包完全拥有数据
- 闭包会被返回或存储

### 6. 闭包作为函数参数

```rust
// 接受任何 Fn(i32) -> i32 类型的闭包
fn apply<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(x)
}

let double = |x| x * 2;
let add_ten = |x| x + 10;

println!("{}", apply(double, 5));   // 10
println!("{}", apply(add_ten, 5));  // 15

// 也可以传普通函数
fn triple(x: i32) -> i32 { x * 3 }
println!("{}", apply(triple, 5));   // 15
```

**选择 trait bound 的原则**：

| 如果需要... | 使用 |
|------------|------|
| 多次调用，不修改状态 | `Fn` |
| 多次调用，可能修改状态 | `FnMut` |
| 至少调用一次（最宽松） | `FnOnce` |

```rust
// 选择最宽松的约束，提高灵活性
fn call_once<F, T>(f: F) -> T
where
    F: FnOnce() -> T,  // 接受所有闭包
{
    f()
}
```

### 7. 闭包作为返回值

```rust
// 使用 impl Trait 语法
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n  // 需要 move，否则 n 会悬垂
}

let add_5 = make_adder(5);
let add_10 = make_adder(10);

println!("{}", add_5(3));   // 8
println!("{}", add_10(3));  // 13
```

### 8. 常用闭包模式

#### 回调函数

```rust
fn process_with_callback<F>(data: &str, callback: F)
where
    F: Fn(&str),
{
    println!("Processing: {}", data);
    callback(data);
}

process_with_callback("hello", |s| {
    println!("Callback received: {}", s);
});
```

#### 惰性求值

```rust
fn expensive_computation() -> i32 {
    println!("Computing...");
    42
}

// 使用闭包延迟计算
fn get_or_compute<F>(cache: Option<i32>, compute: F) -> i32
where
    F: FnOnce() -> i32,
{
    cache.unwrap_or_else(compute)  // 只在 None 时调用
}

let result = get_or_compute(Some(10), || expensive_computation());
// 不会打印 "Computing..."，因为有缓存
```

---

## 项目实现：task-cli 过滤功能

### 通用过滤函数

```rust
/// 根据条件过滤任务
///
/// # 参数
/// - tasks: 任务列表
/// - predicate: 过滤条件，返回 true 的任务会被保留
fn filter_tasks<F>(tasks: &[Task], predicate: F) -> Vec<&Task>
where
    F: Fn(&Task) -> bool,
{
    tasks.iter().filter(|t| predicate(t)).collect()
}
```

### 使用示例

```rust
// 按状态过滤
let pending = filter_tasks(&tasks, |t| t.status == Status::Pending);

// 按优先级过滤
let high_priority = filter_tasks(&tasks, |t| t.priority == Priority::High);

// 组合条件
let urgent = filter_tasks(&tasks, |t| {
    t.status == Status::Pending && t.priority == Priority::High
});

// 动态构建过滤器
fn build_filter(status: Option<Status>, priority: Option<Priority>) -> impl Fn(&Task) -> bool {
    move |task| {
        let status_ok = status.as_ref().map_or(true, |s| &task.status == s);
        let priority_ok = priority.as_ref().map_or(true, |p| &task.priority == p);
        status_ok && priority_ok
    }
}

let filter = build_filter(Some(Status::Pending), None);
let result = filter_tasks(&tasks, filter);
```

---

## 最佳实践

### 何时使用闭包 vs 函数

| 场景 | 推荐 | 原因 |
|------|------|------|
| 需要捕获环境变量 | 闭包 | 函数不能捕获 |
| 一次性使用的短小逻辑 | 闭包 | 更简洁 |
| 可复用的命名逻辑 | 函数 | 更清晰 |
| 需要类型签名明确 | 函数 | 闭包类型无法命名 |
| 传给迭代器方法 | 闭包 | 惯用方式 |

### 选择 Fn trait 的建议

| 需求 | 使用 | 示例 |
|------|------|------|
| 读取状态，多次调用 | `Fn` | 迭代器的 `filter` |
| 修改状态，多次调用 | `FnMut` | 迭代器的 `for_each` |
| 单次调用，可能消耗值 | `FnOnce` | `Option::unwrap_or_else` |
| 不确定时 | `FnOnce` | 最宽松，兼容所有 |

### 常见陷阱

| 陷阱 | 问题 | 解决 |
|------|------|------|
| 忘记 `mut` | 无法修改捕获的变量 | 闭包变量加 `mut` |
| 借用冲突 | 闭包借用时外部不能访问 | 调整作用域或使用 `move` |
| 类型推断失败 | 闭包参数类型不明确 | 显式标注类型 |
| 生命周期问题 | 返回闭包时引用悬垂 | 使用 `move` |

### move 使用指南

```rust
// 需要 move 的场景：

// 1. 闭包返回
fn make_closure() -> impl Fn() {
    let s = String::from("hello");
    move || println!("{}", s)  // 必须 move
}

// 2. 跨线程
let s = String::from("data");
thread::spawn(move || {
    println!("{}", s);  // 必须 move
});

// 3. 闭包存活更久
let closure: Box<dyn Fn()> = {
    let s = String::from("hello");
    Box::new(move || println!("{}", s))  // 必须 move
};
```

---

## 与 Java Lambda 对比

| 特性 | Rust 闭包 | Java Lambda |
|------|----------|-------------|
| 捕获方式 | 借用或移动 | 仅捕获引用（effectively final） |
| 可变捕获 | 支持 `FnMut` | 不支持 |
| 消耗捕获 | 支持 `FnOnce` | 不支持 |
| 内存管理 | 自动，无 GC | GC 管理 |
| 类型 | 每个闭包唯一类型 | 函数式接口 |
| 性能 | 零成本抽象 | 可能装箱 |

```java
// Java - 必须 effectively final
final int factor = 2;
Function<Integer, Integer> mul = x -> x * factor;
// factor = 3;  // 编译错误
```

```rust
// Rust - 可以更灵活
let mut factor = 2;
let mul = |x| x * factor;  // 借用
factor = 3;  // 可以，但之后不能再用 mul（借用规则）

// 或者
let factor = 2;
let mul = move |x| x * factor;  // 移动（复制）
// factor 仍可用（i32 实现了 Copy）
```

---

## 要点回顾

1. **闭包是可以捕获环境的匿名函数**
2. **三种 trait**：`Fn`（借用）、`FnMut`（可变借用）、`FnOnce`（移动）
3. **move 关键字**：强制闭包获取所有权
4. **选择最宽松的约束**：优先用 `FnOnce`，需要多次调用再收紧
5. **闭包是迭代器的基础**：下一章会大量使用

---

## 练习

1. **基础**：写一个函数 `apply_twice`，接受一个闭包和一个值，对值应用闭包两次
2. **进阶**：实现一个计数器闭包工厂，每次调用返回递增的值
3. **挑战**：为 task-cli 实现排序功能，支持按不同字段排序（使用闭包）

---

## 扩展阅读

- [The Rust Book - Closures](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [Rust by Example - Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html)
- [Fn trait 详解](https://doc.rust-lang.org/std/ops/trait.Fn.html)

---

## 下一章预告

学会了闭包，就能理解迭代器的强大。下一章学习集合与迭代器，体验函数式风格的数据处理——`map`、`filter`、`fold` 等方法将让你的代码更加优雅。
