# 第 12 章：集合与迭代器

## 本章目标

学完本章，你将能够：
- 熟练使用 Vec 和 HashMap
- 掌握迭代器适配器链式调用
- 理解惰性求值和零成本抽象
- 区分 iter/into_iter/iter_mut 的使用场景
- 实现 freq 词频统计工具

---

## 前置知识

- 第 11 章：闭包

---

## 项目：freq - 词频统计工具

### 功能概览

一个统计文本中单词出现频率的命令行工具：

```bash
$ freq article.txt
the     142
and     98
rust    67
is      45
a       38
```

### 为什么做这个项目？

1. **实用场景**：文本分析、日志统计
2. **综合练习**：结合 HashMap、Vec、迭代器
3. **性能思考**：体验迭代器的零成本抽象

---

## 核心概念

### 1. Vec 深入

`Vec<T>` 是 Rust 最常用的集合类型，类似 Java 的 `ArrayList`。

**命名解释**：Vec 是 Vector（向量）的缩写，表示可变长度的数组。

#### 创建 Vec

```rust
// 方式 1：使用 new()
let mut v: Vec<i32> = Vec::new();

// 方式 2：使用 vec! 宏
let v = vec![1, 2, 3];

// 方式 3：指定初始容量（避免频繁扩容）
let v: Vec<i32> = Vec::with_capacity(100);

// 方式 4：从迭代器创建
let v: Vec<i32> = (1..=5).collect();
```

#### 常用方法

```rust
let mut v = vec![1, 2, 3];

// 添加元素
v.push(4);              // 末尾添加
v.insert(0, 0);         // 指定位置插入

// 移除元素
v.pop();                // 移除并返回最后一个 Option<T>
v.remove(0);            // 移除指定位置的元素
v.clear();              // 清空

// 访问元素
v.get(0);               // Option<&T>，安全访问
v[0];                   // 直接索引，越界会 panic
v.first();              // Option<&T>，第一个元素
v.last();               // Option<&T>，最后一个元素

// 信息查询
v.len();                // 元素数量
v.is_empty();           // 是否为空
v.capacity();           // 当前容量
v.contains(&2);         // 是否包含元素

// 修改
v.sort();               // 排序（要求 T: Ord）
v.reverse();            // 反转
v.dedup();              // 去除连续重复
```

#### 安全访问 vs 直接索引

```rust
let v = vec![1, 2, 3];

// 推荐：使用 get，返回 Option
match v.get(10) {
    Some(x) => println!("值: {}", x),
    None => println!("索引越界"),
}

// 危险：直接索引，越界会 panic
// let x = v[10];  // panic: index out of bounds
```

### 2. HashMap

`HashMap<K, V>` 是键值对集合，类似 Java 的 `HashMap`。

```rust
use std::collections::HashMap;

// 创建
let mut map: HashMap<String, i32> = HashMap::new();

// 插入
map.insert("apple".to_string(), 3);
map.insert("banana".to_string(), 2);

// 访问
let count = map.get("apple");  // Option<&V>

// 带默认值访问
let count = map.get("orange").unwrap_or(&0);

// 检查是否存在
if map.contains_key("apple") {
    println!("有苹果");
}

// 移除
map.remove("apple");

// 遍历
for (key, value) in &map {
    println!("{}: {}", key, value);
}
```

#### Entry API

`Entry API` 是 HashMap 的强大功能，用于高效处理"存在则更新，不存在则插入"的场景：

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();

// or_insert：不存在则插入，返回值的可变引用
scores.entry("Blue").or_insert(50);

// or_insert_with：不存在则用闭包计算值
scores.entry("Red").or_insert_with(|| expensive_calculation());

// or_default：不存在则插入默认值（T: Default）
let count: &mut i32 = scores.entry("Green").or_default();

// 典型用法：词频统计
let text = "hello world hello rust";
let mut word_count = HashMap::new();

for word in text.split_whitespace() {
    *word_count.entry(word).or_insert(0) += 1;
}
// {"hello": 2, "world": 1, "rust": 1}
```

**Entry 的工作原理**：

```rust
// entry() 返回一个 Entry 枚举
enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),  // 键已存在
    Vacant(VacantEntry<'a, K, V>),      // 键不存在
}
```

### 3. 迭代器基础

迭代器是 Rust 处理序列数据的核心抽象，实现了 `Iterator` trait：

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // 还有很多默认方法...
}
```

#### 创建迭代器

```rust
let v = vec![1, 2, 3, 4, 5];

// 从集合创建
let iter = v.iter();           // 不可变引用迭代器
let iter = v.iter_mut();       // 可变引用迭代器
let iter = v.into_iter();      // 所有权迭代器

// 从范围创建
let iter = 1..10;              // Range 本身就是迭代器
let iter = 1..=10;             // 包含终点

// 无限迭代器
let iter = std::iter::repeat(42);     // 无限重复
let iter = (0..).take(10);            // 从 0 开始，取 10 个
```

#### iter vs into_iter vs iter_mut

这是 Rust 迭代器的重要区别：

| 方法 | 产出类型 | 所有权 | 使用场景 |
|------|---------|--------|---------|
| `iter()` | `&T` | 保留 | 只读遍历 |
| `iter_mut()` | `&mut T` | 保留 | 修改元素 |
| `into_iter()` | `T` | 转移 | 消耗集合 |

```rust
let v = vec![String::from("a"), String::from("b")];

// iter()：借用，v 仍可用
for s in v.iter() {
    println!("{}", s);  // s 是 &String
}
println!("v 仍有 {} 个元素", v.len());

// iter_mut()：可变借用
let mut v = vec![1, 2, 3];
for x in v.iter_mut() {
    *x *= 2;  // 直接修改
}
// v 现在是 [2, 4, 6]

// into_iter()：消耗所有权
let v = vec![String::from("a"), String::from("b")];
for s in v.into_iter() {
    println!("{}", s);  // s 是 String，拥有所有权
}
// v 已被消耗，不能再使用
```

**语法糖**：

```rust
let v = vec![1, 2, 3];

for x in &v { }      // 等价于 v.iter()
for x in &mut v { }  // 等价于 v.iter_mut()
for x in v { }       // 等价于 v.into_iter()
```

### 4. 迭代器适配器

适配器是返回新迭代器的方法，可以链式调用：

#### 转换类

```rust
let v = vec![1, 2, 3, 4, 5];

// map：转换每个元素
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
// [2, 4, 6, 8, 10]

// flat_map：转换并展平
let nested = vec![vec![1, 2], vec![3, 4]];
let flat: Vec<i32> = nested.into_iter().flat_map(|v| v).collect();
// [1, 2, 3, 4]

// enumerate：附加索引
for (i, x) in v.iter().enumerate() {
    println!("索引 {} 值 {}", i, x);
}

// zip：合并两个迭代器
let a = vec![1, 2, 3];
let b = vec!["a", "b", "c"];
let zipped: Vec<_> = a.iter().zip(b.iter()).collect();
// [(1, "a"), (2, "b"), (3, "c")]
```

#### 过滤类

```rust
let v = vec![1, 2, 3, 4, 5, 6];

// filter：过滤元素
let evens: Vec<i32> = v.iter().filter(|x| *x % 2 == 0).cloned().collect();
// [2, 4, 6]

// filter_map：过滤 + 转换（常用于处理 Option）
let strings = vec!["1", "two", "3", "four"];
let numbers: Vec<i32> = strings.iter()
    .filter_map(|s| s.parse().ok())
    .collect();
// [1, 3]

// take：取前 n 个
let first_three: Vec<i32> = v.iter().take(3).cloned().collect();
// [1, 2, 3]

// skip：跳过前 n 个
let after_two: Vec<i32> = v.iter().skip(2).cloned().collect();
// [3, 4, 5, 6]

// take_while / skip_while：条件取/跳
let until_four: Vec<i32> = v.iter().take_while(|x| **x < 4).cloned().collect();
// [1, 2, 3]
```

#### 聚合类（消费者）

这些方法会消费迭代器，产生最终结果：

```rust
let v = vec![1, 2, 3, 4, 5];

// collect：收集为集合
let vec: Vec<i32> = v.iter().cloned().collect();
let set: HashSet<i32> = v.iter().cloned().collect();

// sum / product：求和/求积
let sum: i32 = v.iter().sum();           // 15
let product: i32 = v.iter().product();   // 120

// count：计数
let count = v.iter().count();            // 5

// fold：累积（最通用）
let sum = v.iter().fold(0, |acc, x| acc + x);  // 15

// reduce：类似 fold，但用第一个元素作为初始值
let sum = v.iter().copied().reduce(|a, b| a + b);  // Some(15)

// max / min
let max = v.iter().max();                // Some(&5)
let min = v.iter().min();                // Some(&1)

// find：查找第一个满足条件的
let first_even = v.iter().find(|x| *x % 2 == 0);  // Some(&2)

// any / all：存在性/全称判断
let has_even = v.iter().any(|x| x % 2 == 0);      // true
let all_positive = v.iter().all(|x| *x > 0);      // true
```

### 5. 惰性求值

迭代器是惰性的——适配器调用不会立即执行，只有消费者调用时才真正计算：

```rust
let v = vec![1, 2, 3, 4, 5];

// 这三行不会执行任何计算！
let iter = v.iter()
    .map(|x| {
        println!("处理 {}", x);  // 不会打印
        x * 2
    })
    .filter(|x| x > &5);

// 直到调用消费者才执行
let result: Vec<i32> = iter.collect();  // 现在才打印
```

**惰性求值的优势**：
1. 避免不必要的计算
2. 支持无限序列
3. 减少中间集合的内存分配

```rust
// 处理无限序列
let first_10_squares: Vec<i32> = (1..)          // 无限迭代器
    .map(|x| x * x)
    .take(10)
    .collect();
// [1, 4, 9, 16, 25, 36, 49, 64, 81, 100]
```

### 6. 零成本抽象

Rust 的迭代器是"零成本抽象"——高级抽象编译后性能等同于手写循环：

```rust
// 迭代器风格
let sum: i32 = v.iter()
    .filter(|x| **x % 2 == 0)
    .map(|x| x * 2)
    .sum();

// 编译器生成的代码等效于：
let mut sum = 0;
for x in &v {
    if *x % 2 == 0 {
        sum += x * 2;
    }
}
```

**为什么能做到零成本？**
- 单态化：泛型在编译时具体化
- 内联：适配器方法被内联
- LLVM 优化：消除边界检查等

---

## 与 Java Stream 对比

Rust 迭代器与 Java 8 的 Stream API 非常相似：

```java
// Java Stream
List<Integer> result = list.stream()
    .filter(x -> x % 2 == 0)
    .map(x -> x * 2)
    .collect(Collectors.toList());
```

```rust
// Rust Iterator
let result: Vec<i32> = v.iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| x * 2)
    .collect();
```

**关键差异**：

| 方面 | Java Stream | Rust Iterator |
|------|-------------|---------------|
| 性能 | 有运行时开销 | 零成本抽象 |
| 所有权 | 无所有权概念 | iter/into_iter/iter_mut |
| 并行 | parallelStream() | rayon crate |
| 类型推断 | 需要 Collectors | collect() 根据目标类型推断 |
| 复用 | Stream 只能用一次 | 迭代器也只能用一次 |

---

## 实现 freq

### 完整实现

```rust
use std::collections::HashMap;
use std::env;
use std::fs;

fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        // 转小写，去除标点
        let word = word
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>();

        if !word.is_empty() {
            *counts.entry(word).or_insert(0) += 1;
        }
    }

    counts
}

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: freq <文件路径> [显示数量]");
        return;
    }

    let path = &args[1];
    let limit: usize = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // 读取文件
    let text = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
            return;
        }
    };

    // 统计词频
    let counts = count_words(&text);

    // 按频率排序
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));

    // 输出结果
    println!("{:15} {}", "单词", "频率");
    println!("{:-<20}", "");

    for (word, count) in items.iter().take(limit) {
        println!("{:15} {}", word, count);
    }

    println!("\n共计 {} 个不同单词", counts.len());
}
```

### 使用迭代器风格重写

```rust
fn count_words_iter(text: &str) -> HashMap<String, usize> {
    text.split_whitespace()
        .map(|word| {
            word.to_lowercase()
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .fold(HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        })
}

// 排序也可以用迭代器风格
fn top_words(counts: &HashMap<String, usize>, n: usize) -> Vec<(&String, &usize)> {
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    items.into_iter().take(n).collect()
}
```

---

## 最佳实践

### 选择合适的迭代方式

| 场景 | 推荐方法 | 原因 |
|------|---------|------|
| 只读遍历 | `iter()` / `&v` | 保留所有权 |
| 需要修改元素 | `iter_mut()` / `&mut v` | 原地修改 |
| 需要转移元素 | `into_iter()` / `v` | 元素移出集合 |
| 需要索引 | `enumerate()` | 比手动计数更清晰 |

### 迭代器 vs for 循环

```rust
// 推荐：简单遍历用 for
for item in &items {
    process(item);
}

// 推荐：转换收集用迭代器
let result: Vec<_> = items.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.transform())
    .collect();

// 不推荐：过于复杂的迭代器链
// 如果链条超过 5-6 个适配器，考虑拆分
```

### 常见陷阱

| 陷阱 | 问题 | 解决方案 |
|------|------|---------|
| 忘记 collect | 迭代器是惰性的，不会执行 | 添加消费者调用 |
| 双重引用 | `filter` 闭包参数是 `&&T` | 使用 `**x` 或 `|&x|` |
| into_iter 后使用原集合 | 所有权已转移 | 使用 iter() 或 clone |
| 迭代器复用 | 迭代器只能消费一次 | 重新创建或使用 clone |

```rust
// 双重引用问题
let v = vec![1, 2, 3];

// 错误理解：x 的类型是 &&i32
v.iter().filter(|x| **x > 1);

// 更清晰的写法
v.iter().filter(|&x| *x > 1);
// 或
v.iter().filter(|x| x > &&1);
```

### 性能建议

```rust
// 1. 预分配容量
let mut result = Vec::with_capacity(items.len());

// 2. 避免不必要的 clone
// 不好
items.iter().cloned().filter(|x| x > 5).collect()
// 好
items.iter().filter(|x| **x > 5).cloned().collect()

// 3. 使用 collect 的类型推断
let map: HashMap<_, _> = pairs.into_iter().collect();
```

---

## 要点回顾

1. **Vec 是最常用的集合**
   - 使用 `get()` 安全访问
   - 预分配容量提升性能

2. **HashMap 用 Entry API 更新**
   - `or_insert` 系列方法
   - 避免重复查找

3. **迭代器是惰性的**
   - 适配器不执行计算
   - 消费者触发执行

4. **iter/into_iter/iter_mut**
   - 理解所有权语义
   - 选择合适的方法

5. **零成本抽象**
   - 高级抽象不牺牲性能
   - 编译器优化到手写水平

---

## 练习

### 练习 1：平方和

使用迭代器计算一个 `Vec<i32>` 中所有正数的平方和。

```rust
fn sum_of_positive_squares(v: &[i32]) -> i32 {
    // 你的代码
}
```

### 练习 2：最常见元素

实现一个 `most_common` 函数，返回 HashMap 中出现次数最多的 n 个元素：

```rust
fn most_common<K: Clone, V: Ord + Clone>(
    map: &HashMap<K, V>,
    n: usize
) -> Vec<(K, V)> {
    // 你的代码
}
```

### 练习 3：停用词过滤

为 freq 添加忽略常见词（the, a, an, is, are 等）的功能。

### 练习 4：自定义迭代器

实现一个 `Fibonacci` 迭代器，生成斐波那契数列：

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        // 你的代码
    }
}
```

---

## 扩展阅读

- [The Rust Book - Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
- [The Rust Book - Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Iterator trait 文档](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [Rust By Example - Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html)
- [itertools crate](https://docs.rs/itertools) - 更多迭代器工具

---

## 下一章预告

项目越来越多，代码文件也越来越多。下一章学习 Cargo Workspace，将多个相关项目统一管理。
