# 附录 D：智能指针简介

## 本附录内容

- Box\<T\> - 堆分配
- Rc\<T\> / Arc\<T\> - 引用计数
- RefCell\<T\> - 内部可变性
- 组合使用场景

---

## 什么是智能指针？

智能指针是实现了 `Deref` 和 `Drop` trait 的数据结构：
- **Deref**：允许像引用一样使用（`*ptr`）
- **Drop**：离开作用域时自动清理资源

**Java 对比**：
- Java 的所有对象引用都是"智能的"（自动 GC）
- Rust 需要显式选择内存管理策略

---

## Box\<T\> - 堆分配

### 什么是 Box？

`Box<T>` 是最简单的智能指针，将数据分配在堆上。

```rust
// 栈上分配
let x = 5;           // 4 bytes on stack

// 堆上分配
let boxed = Box::new(5);  // 8 bytes pointer on stack, 4 bytes data on heap
```

### 使用场景

**1. 大型数据避免栈拷贝**

```rust
// 大结构体
struct LargeData {
    data: [u8; 1_000_000],  // 1MB
}

// 栈上分配 - 可能栈溢出
// let data = LargeData { data: [0; 1_000_000] };

// 堆上分配 - 安全
let data = Box::new(LargeData { data: [0; 1_000_000] });
```

**2. 递归类型**

```rust
// ✗ 编译错误：无限大小
// enum List {
//     Cons(i32, List),
//     Nil,
// }

// ✓ 使用 Box 打破递归
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = List::Cons(1,
    Box::new(List::Cons(2,
        Box::new(List::Cons(3,
            Box::new(List::Nil))))));
```

**3. trait 对象**

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

impl Animal for Cat {
    fn speak(&self) { println!("Meow!"); }
}

// Box<dyn Trait> 实现多态
let animals: Vec<Box<dyn Animal>> = vec![
    Box::new(Dog),
    Box::new(Cat),
];

for animal in &animals {
    animal.speak();
}
```

### Box 的行为

```rust
let boxed = Box::new(42);

// 自动解引用（Deref）
let value: i32 = *boxed;
println!("{}", boxed);  // 隐式解引用

// 离开作用域自动释放（Drop）
{
    let temp = Box::new("hello");
}  // temp 在此被释放
```

---

## Rc\<T\> - 引用计数（单线程）

### 什么是 Rc？

`Rc<T>`（Reference Counted）允许多个所有者共享同一数据。

```rust
use std::rc::Rc;

let a = Rc::new(5);
let b = Rc::clone(&a);  // 增加引用计数，不深拷贝
let c = Rc::clone(&a);

println!("引用计数: {}", Rc::strong_count(&a));  // 3
```

### 使用场景

**共享不可变数据**

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<Rc<Node>>,
}

// 多个节点共享同一个后继
let shared = Rc::new(Node { value: 10, next: None });

let a = Node {
    value: 1,
    next: Some(Rc::clone(&shared)),
};

let b = Node {
    value: 2,
    next: Some(Rc::clone(&shared)),
};

// a -> shared <- b
```

### Rc 的限制

```rust
use std::rc::Rc;

let data = Rc::new(5);

// ✗ 不能获取可变引用
// let mut_ref = &mut *data;

// ✗ 不能跨线程
// std::thread::spawn(move || {
//     println!("{}", data);
// });

// Rc 只能用于单线程的不可变共享
```

### Weak\<T\> - 弱引用

```rust
use std::rc::{Rc, Weak};

struct Node {
    value: i32,
    parent: Option<Weak<Node>>,    // 弱引用，不增加计数
    children: Vec<Rc<Node>>,        // 强引用
}

// 解决循环引用问题
// 父节点用 Weak，子节点用 Rc
```

---

## Arc\<T\> - 原子引用计数（多线程）

### 什么是 Arc？

`Arc<T>`（Atomically Reference Counted）是 `Rc<T>` 的线程安全版本。

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);

let handles: Vec<_> = (0..3).map(|i| {
    let data = Arc::clone(&data);
    thread::spawn(move || {
        println!("Thread {}: {:?}", i, data);
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

### Rc vs Arc

| 特性 | Rc\<T\> | Arc\<T\> |
|-----|--------|---------|
| 线程安全 | ✗ 否 | ✓ 是 |
| 性能开销 | 低 | 稍高（原子操作）|
| 使用场景 | 单线程共享 | 多线程共享 |

**选择原则**：默认用 `Rc`，需要跨线程时用 `Arc`。

---

## RefCell\<T\> - 内部可变性

### 什么是内部可变性？

Rust 默认在编译时检查借用规则。`RefCell<T>` 将检查推迟到运行时。

```rust
use std::cell::RefCell;

let data = RefCell::new(5);

// 运行时借用
{
    let mut borrowed = data.borrow_mut();
    *borrowed += 1;
}

println!("{}", data.borrow());  // 6
```

### 借用规则（运行时检查）

```rust
use std::cell::RefCell;

let data = RefCell::new(5);

let r1 = data.borrow();     // 不可变借用
let r2 = data.borrow();     // ✓ 可以有多个不可变借用

drop(r1);
drop(r2);

let mut w = data.borrow_mut();  // 可变借用
// let r3 = data.borrow();      // ✗ panic! 已有可变借用
```

### 使用场景

**结合 Rc 实现可变共享**

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    children: RefCell<Vec<Rc<Node>>>,
}

let parent = Rc::new(Node {
    value: 1,
    children: RefCell::new(vec![]),
});

let child = Rc::new(Node {
    value: 2,
    children: RefCell::new(vec![]),
});

// 可以修改 children
parent.children.borrow_mut().push(Rc::clone(&child));

println!("{:?}", parent);
```

### Cell vs RefCell

```rust
use std::cell::{Cell, RefCell};

// Cell<T>：适用于 Copy 类型，无借用概念
let cell = Cell::new(5);
cell.set(10);
let value = cell.get();

// RefCell<T>：适用于任意类型，运行时借用检查
let refcell = RefCell::new(String::from("hello"));
refcell.borrow_mut().push_str(" world");
```

| 特性 | Cell\<T\> | RefCell\<T\> |
|-----|----------|--------------|
| 类型要求 | T: Copy | 任意类型 |
| 获取方式 | get/set（复制）| borrow/borrow_mut（引用）|
| 检查 | 无 | 运行时借用检查 |
| 开销 | 更低 | 稍高 |

---

## 组合使用

### Rc\<RefCell\<T\>\> - 单线程可变共享

```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared = Rc::new(RefCell::new(vec![1, 2, 3]));

let a = Rc::clone(&shared);
let b = Rc::clone(&shared);

// a 和 b 都可以修改
a.borrow_mut().push(4);
b.borrow_mut().push(5);

println!("{:?}", shared.borrow());  // [1, 2, 3, 4, 5]
```

### Arc\<Mutex\<T\>\> - 多线程可变共享

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap());  // 10
```

---

## 选择指南

```
需要堆分配？
├── 否 → 直接使用栈变量
└── 是 → 需要共享所有权？
    ├── 否 → Box<T>
    └── 是 → 需要跨线程？
        ├── 否 → Rc<T>
        │   └── 需要内部可变？
        │       ├── 否 → Rc<T>
        │       └── 是 → Rc<RefCell<T>>
        └── 是 → Arc<T>
            └── 需要内部可变？
                ├── 否 → Arc<T>
                └── 是 → Arc<Mutex<T>> 或 Arc<RwLock<T>>
```

### 常用组合速查

| 场景 | 类型 |
|-----|------|
| 堆分配，单一所有者 | `Box<T>` |
| 单线程共享，不可变 | `Rc<T>` |
| 单线程共享，可变 | `Rc<RefCell<T>>` |
| 多线程共享，不可变 | `Arc<T>` |
| 多线程共享，可变 | `Arc<Mutex<T>>` |
| 多线程共享，读多写少 | `Arc<RwLock<T>>` |

---

## 与 Java 对比

| 概念 | Rust | Java |
|-----|------|------|
| 堆分配 | `Box<T>` | 所有对象默认堆上 |
| 共享引用 | `Rc<T>` / `Arc<T>` | 默认行为 |
| 内部可变 | `RefCell<T>` | 默认可变 |
| 互斥锁 | `Mutex<T>` | `synchronized` |
| 引用计数 | 显式 | GC 自动管理 |

**核心区别**：
- Java 通过 GC 自动管理，开发者不用关心
- Rust 让开发者显式选择内存管理策略
- Rust 的方案在编译时能发现更多错误

---

## 最佳实践

### 选择原则

1. **默认用栈分配**：除非有特殊需求
2. **优先 Box**：单一所有者最简单
3. **Rc 仅限单线程**：多线程必须用 Arc
4. **RefCell 是最后手段**：能用编译时检查就不用运行时

### 避免的模式

```rust
// ✗ 避免：不必要的 Box
let x = Box::new(5);  // i32 已经很小，没必要

// ✗ 避免：到处使用 Rc
let x = Rc::new(5);  // 如果不需要共享，别用

// ✗ 避免：Rc 跨线程（编译会报错）
// ✗ 避免：忽略 RefCell 的 panic 可能
```

### 推荐的模式

```rust
// ✓ 递归类型用 Box
enum Tree {
    Node(i32, Box<Tree>, Box<Tree>),
    Leaf,
}

// ✓ trait 对象用 Box
fn get_animal() -> Box<dyn Animal> {
    Box::new(Dog)
}

// ✓ 图结构用 Rc
struct Graph {
    nodes: Vec<Rc<Node>>,
}

// ✓ 多线程共享状态用 Arc<Mutex<T>>
struct SharedState {
    data: Arc<Mutex<HashMap<String, String>>>,
}
```

---

## 延伸阅读

- [Rust Book - Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [Rust Book - Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [std::rc 文档](https://doc.rust-lang.org/std/rc/)
- [std::sync 文档](https://doc.rust-lang.org/std/sync/)
