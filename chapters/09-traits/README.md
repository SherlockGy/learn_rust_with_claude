# 第 9 章：Trait 基础

## 本章目标

学完本章，你将能够：
- 定义和实现 trait
- 使用常用标准 trait（Display, Debug, Clone 等）
- 理解 derive 宏的作用
- 为 task-cli 添加漂亮的格式化输出

---

## 前置知识

- 第 8 章：错误处理、文件操作

---

## 项目：为 task-cli 实现漂亮的输出

### 本章目标

实现统一的格式化：

```bash
$ task list
┌─────┬────────┬────────┬──────────────────┐
│ ID  │ 状态   │ 优先级  │ 任务              │
├─────┼────────┼────────┼──────────────────┤
│  1  │ 待办   │  中     │ 学习 Trait       │
│  2  │ 完成   │  高     │ 安装 Rust        │
└─────┴────────┴────────┴──────────────────┘

$ task show 1
Task #1
  标题: 学习 Trait
  状态: 待办
  优先级: 中
```

---

## 核心概念

### 1. 什么是 Trait？

Trait 定义了一组行为（方法），类似 Java 的接口：

```rust
trait Summary {
    fn summarize(&self) -> String;
}
```

**命名解释**：Trait 意为"特性"、"特征"，表示类型具有的能力。

### 2. 实现 Trait

```rust
struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}...", self.title, &self.content[..50])
    }
}
```

### 3. 与 Java 接口对比

```java
// Java
interface Summary {
    String summarize();
}

class Article implements Summary {
    public String summarize() { ... }
}
```

```rust
// Rust
trait Summary {
    fn summarize(&self) -> String;
}

impl Summary for Article {
    fn summarize(&self) -> String { ... }
}
```

**关键差异**：

| Java | Rust |
|------|------|
| 只能为自己的类实现接口 | 可以为任何类型实现 trait |
| 接口不能有默认实现（Java 8 前） | trait 可以有默认实现 |
| 没有关联类型 | 支持关联类型 |

### 4. 常用标准 Trait

| Trait | 用途 | 方法 |
|-------|------|------|
| `Display` | 用户友好输出 | `fmt()` |
| `Debug` | 调试输出 | `fmt()` |
| `Clone` | 深拷贝 | `clone()` |
| `Copy` | 按位复制 | （标记 trait） |
| `PartialEq` | 相等比较 | `eq()` |
| `Default` | 默认值 | `default()` |

### 5. Display Trait

```rust
use std::fmt;

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.status.as_str(), self.title)
    }
}

// 使用
let task = Task::new(1, "Learn Rust".to_string());
println!("{}", task);  // 输出: [待办] Learn Rust
```

### 6. Debug Trait

```rust
// 方式 1：derive 自动实现
#[derive(Debug)]
struct Task { ... }

// 方式 2：手动实现
impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("title", &self.title)
            .finish()
    }
}

// 使用
println!("{:?}", task);   // 单行
println!("{:#?}", task);  // 多行美化
```

### 7. derive 宏

`#[derive(...)]` 自动实现常见 trait：

```rust
#[derive(Debug, Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
}
```

**可 derive 的 trait**：Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default

---

## 逐步为 task-cli 实现 Trait

### 步骤 1：实现 Display

```rust
use std::fmt;

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>3} │ {:^6} │ {:^4} │ {}",
            self.id, self.status, self.priority, self.title)
    }
}
```

### 步骤 2：实现 Debug

```rust
#[derive(Debug)]
enum Status { ... }

#[derive(Debug)]
enum Priority { ... }

#[derive(Debug)]
struct Task { ... }
```

### 步骤 3：改进列表输出

```rust
fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("┌─────┬────────┬──────┬────────────────────────┐");
    println!("│ ID  │  状态  │优先级│ 任务                   │");
    println!("├─────┼────────┼──────┼────────────────────────┤");

    for task in tasks {
        println!("│{}│", task);
    }

    println!("└─────┴────────┴──────┴────────────────────────┘");
}
```

---

## 要点回顾

1. **Trait 定义共享行为**
   - 类似接口但更灵活
   - 可以有默认实现

2. **Display vs Debug**
   - Display: 给用户看
   - Debug: 给开发者看

3. **derive 自动实现**
   - 减少样板代码
   - 适用于简单场景

---

## 最佳实践

| 场景 | 推荐 |
|------|------|
| 需要 `{}` 输出 | 实现 Display |
| 调试用 | derive Debug |
| 可能复制 | 评估是否需要 Clone |
| 比较相等 | derive PartialEq |

### 何时手动实现 vs derive

- **derive**：字段简单，默认行为够用
- **手动**：需要自定义格式或行为

---

## 练习

### 练习 1：实现 Default

为 Task 实现 Default trait。

### 练习 2：自定义 Debug

手动实现 Debug，隐藏某些字段。

---

## 扩展阅读

- [The Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Rust By Example - Traits](https://doc.rust-lang.org/rust-by-example/trait.html)
- [常用 trait 一览](https://doc.rust-lang.org/std/#traits)

---

## 下一章预告

我们已经学会了定义行为，但每个类型都要单独实现。下一章学习**泛型**，让代码能处理多种类型。
