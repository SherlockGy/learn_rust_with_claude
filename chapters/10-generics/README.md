# 第 10 章：泛型

## 本章目标

学完本章，你将能够：
- 定义泛型函数和结构体
- 使用 trait bound 约束泛型
- 理解单态化和零成本抽象
- 实现一个简化版动态数组 MiniVec

---

## 前置知识

- 第 9 章：Trait 基础

---

## 项目：mini-vec - 简化版动态数组

### 为什么做这个项目？

- 泛型最好的学习方式是实现一个泛型容器
- Vec 是最常用的集合，理解其原理很有价值
- 通过实现加深对所有权和借用的理解

### 最终效果

```rust
let mut vec: MiniVec<i32> = MiniVec::new();
vec.push(1);
vec.push(2);
vec.push(3);

assert_eq!(vec.get(0), Some(&1));
assert_eq!(vec.pop(), Some(3));
assert_eq!(vec.len(), 2);
```

---

## 核心概念

### 1. 泛型函数

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

**语法解释**：
- `<T>`：声明类型参数 T
- `T: PartialOrd`：trait bound，T 必须可比较
- 函数可以处理任何实现了 PartialOrd 的类型

### 2. 泛型结构体

```rust
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}

// 使用
let int_point = Point::new(5, 10);
let float_point = Point::new(1.0, 4.0);
```

### 3. 与 Java 泛型对比

```java
// Java - 类型擦除
List<Integer> list = new ArrayList<>();
// 运行时不知道是 Integer
```

```rust
// Rust - 单态化
let vec: Vec<i32> = Vec::new();
// 编译时生成专门的 Vec_i32 代码
```

**关键差异**：

| Java | Rust |
|------|------|
| 类型擦除 | 单态化 |
| 运行时有开销 | 零成本抽象 |
| 不能用基本类型 | 可以用任何类型 |

### 4. Trait Bound

约束泛型必须实现某些 trait：

```rust
// 语法 1：冒号
fn print<T: Display>(item: T) {
    println!("{}", item);
}

// 语法 2：where 子句（多个约束时更清晰）
fn process<T, U>(t: T, u: U)
where
    T: Display + Clone,
    U: Debug,
{
    // ...
}
```

### 5. 单态化

编译器为每个具体类型生成专门的代码：

```rust
fn identity<T>(x: T) -> T { x }

identity(5);      // 生成 identity_i32
identity("hi");   // 生成 identity_str
```

这意味着：
- 没有运行时开销
- 二进制可能变大
- 编译可能变慢

---

## 逐步实现 MiniVec

### 步骤 1：定义结构

```rust
struct MiniVec<T> {
    data: Vec<T>,  // 内部使用标准 Vec（简化实现）
}

impl<T> MiniVec<T> {
    fn new() -> MiniVec<T> {
        MiniVec { data: Vec::new() }
    }
}
```

### 步骤 2：基本方法

```rust
impl<T> MiniVec<T> {
    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
}
```

### 步骤 3：需要 trait bound 的方法

```rust
impl<T: Clone> MiniVec<T> {
    fn first_clone(&self) -> Option<T> {
        self.data.first().cloned()
    }
}

impl<T: PartialEq> MiniVec<T> {
    fn contains(&self, item: &T) -> bool {
        self.data.contains(item)
    }
}
```

### 完整实现

```rust
struct MiniVec<T> {
    data: Vec<T>,
}

impl<T> MiniVec<T> {
    fn new() -> MiniVec<T> {
        MiniVec { data: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<T: Clone> MiniVec<T> {
    fn to_vec(&self) -> Vec<T> {
        self.data.clone()
    }
}

impl<T: PartialEq> MiniVec<T> {
    fn contains(&self, item: &T) -> bool {
        self.data.contains(item)
    }
}
```

---

## 运行与测试

```rust
fn main() {
    let mut vec: MiniVec<i32> = MiniVec::new();

    vec.push(1);
    vec.push(2);
    vec.push(3);

    println!("长度: {}", vec.len());           // 3
    println!("第一个: {:?}", vec.get(0));       // Some(&1)
    println!("包含 2: {}", vec.contains(&2));   // true

    println!("弹出: {:?}", vec.pop());          // Some(3)
    println!("新长度: {}", vec.len());          // 2
}
```

---

## 要点回顾

1. **泛型让代码复用**
   - `<T>` 声明类型参数
   - 函数、结构体、枚举都可以泛型

2. **Trait bound 约束类型**
   - `T: Trait` 语法
   - where 子句更清晰

3. **单态化 = 零成本**
   - 编译时生成专门代码
   - 无运行时开销

---

## 最佳实践

### 泛型 vs 具体类型

```rust
// 好：需要灵活性时用泛型
fn process<T: Display>(item: T) { ... }

// 好：只需要一种类型时用具体类型
fn process_string(s: &str) { ... }
```

### 避免过度泛型化

```rust
// 过度：不需要这么多泛型
fn foo<A, B, C, D>(a: A, b: B, c: C, d: D) { ... }

// 适度：只在需要时使用
fn foo<T: Clone>(items: &[T]) -> Vec<T> { ... }
```

---

## 练习

### 练习 1：实现 Index trait

让 MiniVec 支持 `vec[0]` 语法。

### 练习 2：实现迭代器

让 MiniVec 支持 `for item in &vec` 语法。

---

## 可选练习：将所学应用到 task-cli

在 task-cli 中尝试使用泛型：
- 泛型过滤函数
- 泛型查找函数

---

## 扩展阅读

- [The Rust Book - Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [Rust By Example - Generics](https://doc.rust-lang.org/rust-by-example/generics.html)
- [泛型与 trait bound](https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods)

---

## 下一章预告

我们学会了泛型函数，但还没学过**闭包**——可以捕获环境的匿名函数。下一章将学习闭包，为迭代器章节做准备。
