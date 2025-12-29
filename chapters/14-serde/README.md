# 第 14 章：Serde 与序列化

## 本章目标

学完本章，你将能够：
- 理解 Serde 的设计哲学和独特价值
- 使用 Serde 序列化/反序列化数据
- 掌握 JSON 和 TOML 格式处理
- 熟练使用 Serde 属性控制序列化行为
- 为 task-cli 添加 JSON 持久化存储

---

## 前置知识

- 第 9 章：Trait（理解 derive 和 trait 实现）
- 第 13 章：Workspace（添加和管理依赖）

---

## Serde：Rust 生态中最独特的序列化方案

### 背景介绍

**Serde** 由 **David Tolnay** 开发和维护。David Tolnay 是 Rust 生态中最活跃的贡献者之一，他还维护着 `syn`、`quote`、`proc-macro2`（过程宏三件套）、`anyhow`、`thiserror` 等核心 crate。

**名字由来**：Serde = **Ser**ialize + **De**serialize（序列化 + 反序列化）

**生态地位**：
- crates.io 下载量第一的 crate（超过 3 亿次下载）
- 几乎所有需要数据序列化的 Rust 项目都依赖它
- 被 Rust 官方、AWS、Google、Microsoft 等广泛使用
- 稳定维护超过 8 年（2016 年首次发布）

**为什么 Serde 如此重要？** 因为它不只是一个库，而是定义了 Rust 生态中序列化的"协议标准"。

---

### 设计理念：框架与格式分离

Serde 的核心设计理念是 **数据结构与序列化格式完全解耦**。

#### 传统方案的问题

在大多数语言中，序列化库是"格式绑定"的：

```java
// Java - Jackson 就是 JSON 库
ObjectMapper mapper = new ObjectMapper();
String json = mapper.writeValueAsString(user);

// 要支持 YAML？换一个库，API 完全不同
YAMLFactory factory = new YAMLFactory();
ObjectMapper yamlMapper = new ObjectMapper(factory);
```

```python
# Python - json 模块只能处理 JSON
import json
json.dumps(user)

# 要 YAML？换库，API 不同
import yaml
yaml.dump(user)
```

**问题**：
1. 每种格式需要学习不同的 API
2. 数据结构的序列化逻辑与格式耦合
3. 切换格式需要大量代码修改

#### Serde 的解决方案

Serde 将序列化分为三个独立层次：

```
┌─────────────────────────────────────────────────┐
│                  你的数据结构                      │
│        #[derive(Serialize, Deserialize)]         │
│              struct User { ... }                 │
└─────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────┐
│              Serde 核心（协议层）                  │
│         Serialize / Deserialize trait           │
│              与格式完全无关！                      │
└─────────────────────────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │serde_json│  │  toml    │  │serde_yaml│
    │   JSON   │  │  TOML    │  │   YAML   │
    └──────────┘  └──────────┘  └──────────┘
```

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

let user = User { name: "Alice".into(), age: 30 };

// 同一个结构，不同格式，API 完全一致！
let json = serde_json::to_string(&user)?;      // {"name":"Alice","age":30}
let toml = toml::to_string(&user)?;            // name = "Alice"\nage = 30
let yaml = serde_yaml::to_string(&user)?;      // name: Alice\nage: 30
let msgpack = rmp_serde::to_vec(&user)?;       // 二进制格式
```

---

### 为什么其他语言做不到？

Serde 的设计之所以可行，依赖 Rust 的三个独特特性：

#### 1. Trait 系统（定义通用协议）

```rust
// Serde 定义了两个核心 trait
pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}
```

这些 trait 定义了"如何描述数据"，而不是"如何编码数据"。

**与 Java 接口的区别**：
- Java 的接口无法表达"反序列化返回 Self 类型"
- Rust 的关联类型和泛型让 trait 可以描述更复杂的协议

#### 2. 过程宏（自动生成实现）

```rust
#[derive(Serialize, Deserialize)]  // 这是过程宏！
struct Task {
    id: u32,
    title: String,
}
```

`derive` 宏在编译时分析你的结构体，自动生成完整的 `Serialize` 和 `Deserialize` 实现。

**生成的代码类似于**：
```rust
impl Serialize for Task {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Task", 2)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("title", &self.title)?;
        state.end()
    }
}
```

你不需要手写这些代码，宏帮你完成了一切。

#### 3. 零成本抽象（编译时优化）

Serde 的所有抽象在编译时完全内联展开，运行时没有任何开销。

```rust
// 这行代码
let json = serde_json::to_string(&task)?;

// 编译后的效率等同于手写的 JSON 序列化代码
// 没有反射，没有动态分发，没有运行时类型检查
```

**对比 Java Jackson**：
- Jackson 使用反射在运行时分析对象结构
- 每次序列化都有反射开销
- 需要缓存反射信息来优化性能

**Serde 的方式**：
- 编译时生成所有代码
- 运行时直接执行，无需分析
- 性能接近手写代码

---

## 核心概念

### 1. 基本使用

**添加依赖**：

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**命名解释**：
- `features = ["derive"]`：启用 `#[derive(Serialize, Deserialize)]` 宏
- 默认不启用是为了减少不需要 derive 时的编译时间

**基本示例**：

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let task = Task {
        id: 1,
        title: "Learn Serde".into(),
        done: false,
    };

    // 序列化：结构体 -> JSON 字符串
    let json = serde_json::to_string(&task)?;
    println!("JSON: {}", json);
    // 输出: {"id":1,"title":"Learn Serde","done":false}

    // 美化输出（带缩进）
    let pretty = serde_json::to_string_pretty(&task)?;
    println!("Pretty:\n{}", pretty);

    // 反序列化：JSON 字符串 -> 结构体
    let parsed: Task = serde_json::from_str(&json)?;
    println!("Parsed: {:?}", parsed);

    Ok(())
}
```

### 2. 常用属性详解

Serde 通过属性（attributes）控制序列化行为。属性分为三个级别：

#### 容器属性（用于 struct/enum）

```rust
// rename_all: 批量重命名所有字段
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // snake_case -> camelCase
struct UserProfile {
    user_name: String,      // 序列化为 "userName"
    email_address: String,  // 序列化为 "emailAddress"
}

// 常用的 rename_all 值：
// - "camelCase"     : userName
// - "PascalCase"    : UserName
// - "snake_case"    : user_name
// - "SCREAMING_SNAKE_CASE": USER_NAME
// - "kebab-case"    : user-name
```

```rust
// deny_unknown_fields: 严格模式，遇到未知字段报错
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    port: u16,
    host: String,
}
// 如果 JSON 中有 "unknown_field"，反序列化会失败
```

#### 字段属性

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    // rename: 单独重命名一个字段
    #[serde(rename = "serverPort")]
    port: u16,

    // default: 缺失时使用 Default::default()
    #[serde(default)]
    timeout: u32,  // 缺失时为 0

    // default = "函数名": 缺失时调用指定函数
    #[serde(default = "default_host")]
    host: String,

    // skip: 完全跳过此字段
    #[serde(skip)]
    internal_cache: HashMap<String, String>,

    // skip_serializing_if: 条件性跳过
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,  // None 时不输出此字段

    // alias: 反序列化时接受多个名称
    #[serde(alias = "userName", alias = "user")]
    username: String,

    // flatten: 展平嵌套结构
    #[serde(flatten)]
    metadata: Metadata,
}

fn default_host() -> String {
    "localhost".to_string()
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    created_at: String,
    updated_at: String,
}
// flatten 后，created_at 和 updated_at 会直接出现在 Config 的 JSON 中
```

#### 枚举属性

```rust
// 默认：外部标记 {"status": {"Active": {...}}}
#[derive(Serialize, Deserialize)]
enum Status {
    Active,
    Inactive,
    Pending { reason: String },
}

// 内部标记：{"status": "Active"} 或 {"type": "Pending", "reason": "..."}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: String },
}

// 相邻标记：{"t": "Click", "c": {"x": 1, "y": 2}}
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Message {
    Text(String),
    Image { url: String },
}

// 无标记（仅靠内容区分）
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrInt {
    Str(String),
    Int(i32),
}
```

### 3. 处理多种格式

#### JSON（最常用）

```rust
use serde_json::{json, Value};

// 使用 json! 宏构造动态 JSON
let data = json!({
    "name": "Alice",
    "tags": ["rust", "serde"],
    "metadata": {
        "version": 1
    }
});

// Value 类型：动态 JSON
let v: Value = serde_json::from_str(r#"{"key": "value"}"#)?;
if let Some(key) = v.get("key") {
    println!("key = {}", key);
}
```

#### TOML（配置文件）

```toml
# config.toml
[server]
host = "localhost"
port = 8080

[database]
url = "postgres://localhost/mydb"
max_connections = 10
```

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

### 4. 错误处理

```rust
use serde_json::Error as JsonError;

fn parse_task(json: &str) -> Result<Task, JsonError> {
    serde_json::from_str(json)
}

fn main() {
    match parse_task(r#"{"id": "not a number"}"#) {
        Ok(task) => println!("{:?}", task),
        Err(e) => {
            // Serde 错误信息非常详细
            eprintln!("解析失败: {}", e);
            // 输出: 解析失败: invalid type: string "not a number", expected u32 at line 1 column 18
        }
    }
}
```

---

## 项目：升级 task-cli（JSON 持久化）

现在让我们用 Serde 改进 task-cli，从简单的文本格式升级到 JSON 格式。

### 更新 Cargo.toml

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 更新数据结构

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub priority: Priority,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}
```

### 实现存储功能

```rust
use std::error::Error;
use std::fs;
use std::path::Path;

const DATA_FILE: &str = "tasks.json";

/// 保存任务列表到 JSON 文件
pub fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn Error>> {
    // to_string_pretty 生成格式化的 JSON，便于人类阅读
    let json = serde_json::to_string_pretty(tasks)?;
    fs::write(DATA_FILE, json)?;
    Ok(())
}

/// 从 JSON 文件加载任务列表
pub fn load_tasks() -> Result<Vec<Task>, Box<dyn Error>> {
    // 文件不存在时返回空列表
    if !Path::new(DATA_FILE).exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(DATA_FILE)?;

    // 空文件也返回空列表
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let tasks = serde_json::from_str(&json)?;
    Ok(tasks)
}
```

### 生成的 JSON 格式

```json
[
  {
    "id": 1,
    "title": "学习 Rust 所有权",
    "status": "done",
    "priority": "high"
  },
  {
    "id": 2,
    "title": "完成 Serde 章节",
    "status": "inprogress",
    "priority": "medium"
  }
]
```

---

## 最佳实践

### 何时使用 Serde

| 场景 | 推荐 | 说明 |
|------|------|------|
| 配置文件 | TOML | 人类可读，注释友好 |
| API 交互 | JSON | 通用标准 |
| 数据存储 | JSON / MessagePack | JSON 调试方便，MessagePack 更紧凑 |
| 日志结构化 | JSON | 便于日志分析工具处理 |

### 属性使用建议

| 需求 | 推荐属性 |
|------|----------|
| 与 JavaScript 交互 | `#[serde(rename_all = "camelCase")]` |
| 可选字段 | `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]` |
| 带默认值的字段 | `#[serde(default)]` 或 `#[serde(default = "fn")]` |
| 版本兼容 | `#[serde(alias = "old_name")]` |
| 敏感字段 | `#[serde(skip)]` |

### 常见陷阱

| 陷阱 | 正确做法 |
|------|----------|
| 忘记 `features = ["derive"]` | 确保 Cargo.toml 中启用 |
| `Option<T>` 序列化为 `null` | 添加 `skip_serializing_if` 跳过 None |
| 枚举默认外部标记格式 | 根据需求选择 `tag`/`untagged` |
| 反序列化失败无提示 | 使用 `?` 传播错误，打印详细信息 |
| 字段顺序依赖 | Serde 默认不保证顺序，如需要使用 `preserve_order` feature |

### 性能优化

```rust
// 1. 避免重复解析，缓存结果
let config: Config = serde_json::from_str(&content)?;

// 2. 大数据使用流式处理
use serde_json::Deserializer;
let stream = Deserializer::from_reader(file).into_iter::<Task>();
for task in stream {
    process(task?);
}

// 3. 已知结构优先用强类型，避免 Value
// 好：直接反序列化到结构体
let task: Task = serde_json::from_str(&json)?;

// 避免：先解析为 Value 再提取
let v: Value = serde_json::from_str(&json)?;
let id = v["id"].as_u64().unwrap();
```

---

## 要点回顾

1. **Serde = 框架与格式分离**：定义数据结构一次，支持多种格式
2. **derive 宏**：`#[derive(Serialize, Deserialize)]` 自动生成实现
3. **零成本抽象**：编译时生成代码，运行时无反射开销
4. **丰富的属性**：`rename`、`default`、`skip`、`flatten` 等控制行为
5. **这是 Rust trait 系统 + 过程宏的完美体现**

---

## 练习

1. **基础**：为一个 `Book` 结构体（title, author, year, isbn）添加 Serde 支持，实现 JSON 读写
2. **进阶**：创建一个配置文件解析器，支持从 TOML 加载应用配置
3. **挑战**：实现一个 JSON 与 TOML 的转换工具

---

## 扩展阅读

- [Serde 官方文档](https://serde.rs/)
- [Serde 属性完整列表](https://serde.rs/attributes.html)
- [David Tolnay 的 GitHub](https://github.com/dtolnay)
- [Understanding Serde](https://www.joshmcguigan.com/blog/understanding-serde/) - 深入理解 Serde 内部机制

---

## 下一章预告

数据存储搞定了，下一章用 Clap 让 task-cli 的命令行界面更专业——带有子命令、参数验证、帮助信息的完整 CLI 体验。
