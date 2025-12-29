# 第 15 章：命令行应用进阶

## 本章目标

学完本章，你将能够：
- 使用 Clap 解析命令行参数
- 设计子命令结构
- 实现配置文件管理
- 完成生产级 task-cli

---

## 前置知识

- 第 14 章：Serde

---

## Clap：Rust CLI 生态的标准

### 背景介绍

**Clap**（Command Line Argument Parser）是 Rust 最流行的命令行参数解析库，由 Kevin K. 创建，现由社区维护。

**名字由来**：Clap 是 "Command Line Argument Parser" 的缩写，同时也是"鼓掌"的意思，寓意"为优秀的 CLI 体验鼓掌"。

**生态地位**：
- crates.io 下载量前十
- Rust 官方工具（如 cargo、rustup）的参数解析基础
- 几乎所有 Rust CLI 工具的首选
- 活跃开发超过 8 年，API 成熟稳定

**为什么选择 Clap？**

| 方案 | 优点 | 缺点 |
|------|------|------|
| 手动解析 `env::args` | 无依赖 | 繁琐、易错、无帮助生成 |
| `structopt` | 声明式 | 已合并入 Clap 4 |
| `clap` Builder API | 灵活 | 代码冗长 |
| **`clap` Derive** | 简洁、类型安全 | 编译稍慢 |

### 设计理念

Clap 的核心哲学是**声明式优于命令式**：

```rust
// 命令式（旧方式，繁琐）
let matches = Command::new("task")
    .arg(Arg::new("add")
        .short('a')
        .long("add")
        .value_name("TITLE")
        .help("添加任务"))
    .get_matches();

// 声明式（推荐，简洁）
#[derive(Parser)]
struct Cli {
    #[arg(short, long, help = "添加任务")]
    add: Option<String>,
}
```

**声明式的优势**：
- 代码即文档：看结构体就知道支持哪些参数
- 类型安全：编译器检查参数类型
- 自动完成：IDE 支持更好
- 减少样板代码

---

## 核心概念

### 1. 基本结构

```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "myapp")]
#[command(version = "1.0")]
#[command(about = "我的应用程序")]
struct Cli {
    /// 输入文件路径
    input: String,

    /// 启用详细输出
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    println!("输入: {}, 详细: {}", cli.input, cli.verbose);
}
```

### 2. 参数类型

#### 位置参数（Positional Arguments）

```rust
#[derive(Parser)]
struct Cli {
    /// 源文件
    source: String,

    /// 目标文件
    destination: String,
}
// 用法: myapp src.txt dst.txt
```

#### 选项参数（Options）

```rust
#[derive(Parser)]
struct Cli {
    /// 输出文件
    #[arg(short, long)]
    output: Option<String>,

    /// 重复次数
    #[arg(short = 'n', long = "count", default_value = "1")]
    count: u32,
}
// 用法: myapp -o out.txt -n 3
// 用法: myapp --output out.txt --count 3
```

#### 标志（Flags）

```rust
#[derive(Parser)]
struct Cli {
    /// 详细模式
    #[arg(short, long)]
    verbose: bool,

    /// 安静模式
    #[arg(short, long)]
    quiet: bool,
}
// 用法: myapp -v
// 用法: myapp --verbose --quiet
```

#### 多值参数

```rust
#[derive(Parser)]
struct Cli {
    /// 输入文件列表
    #[arg(required = true)]
    files: Vec<String>,

    /// 排除模式
    #[arg(short, long)]
    exclude: Vec<String>,
}
// 用法: myapp file1.txt file2.txt -e "*.tmp" -e "*.bak"
```

### 3. 子命令（Subcommands）

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "task")]
#[command(about = "命令行待办事项管理器")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 添加新任务
    Add {
        /// 任务标题
        title: String,

        /// 优先级 (low, medium, high)
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// 截止日期
        #[arg(short, long)]
        due: Option<String>,
    },

    /// 列出任务
    List {
        /// 按状态过滤
        #[arg(short, long)]
        status: Option<String>,

        /// 只显示数量
        #[arg(short, long)]
        count: bool,
    },

    /// 标记任务完成
    Done {
        /// 任务 ID
        id: u32,
    },

    /// 删除任务
    #[command(alias = "rm")]  // 别名
    Remove {
        /// 任务 ID
        id: u32,

        /// 强制删除，不确认
        #[arg(short, long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { title, priority, due } => {
            println!("添加任务: {} (优先级: {}, 截止: {:?})", title, priority, due);
        }
        Commands::List { status, count } => {
            println!("列出任务 (状态: {:?}, 只计数: {})", status, count);
        }
        Commands::Done { id } => {
            println!("完成任务 #{}", id);
        }
        Commands::Remove { id, force } => {
            println!("删除任务 #{} (强制: {})", id, force);
        }
    }
}
```

### 4. 常用属性详解

#### #[command(...)] 属性

```rust
#[derive(Parser)]
#[command(
    name = "task",                    // 程序名
    version = "1.0.0",                // 版本号（也可用 version 宏）
    author = "作者名",                 // 作者
    about = "简短描述",                // 简介
    long_about = "详细描述...",        // 长描述
    after_help = "更多信息请访问...",   // 帮助后的附加文本
    arg_required_else_help = true,    // 无参数时显示帮助
)]
struct Cli { ... }
```

#### #[arg(...)] 属性

```rust
#[derive(Parser)]
struct Cli {
    // 短选项和长选项
    #[arg(short = 'o', long = "output")]
    output: String,

    // 自动推断短选项（取首字母）
    #[arg(short, long)]
    verbose: bool,  // -v, --verbose

    // 默认值
    #[arg(default_value = "default.txt")]
    file: String,

    // 环境变量
    #[arg(env = "MY_CONFIG")]
    config: Option<String>,

    // 值验证
    #[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,

    // 可能的值
    #[arg(value_parser = ["debug", "info", "warn", "error"])]
    level: String,

    // 隐藏选项（不在帮助中显示）
    #[arg(hide = true)]
    secret: Option<String>,

    // 必需参数
    #[arg(required = true)]
    input: String,

    // 互斥参数
    #[arg(conflicts_with = "quiet")]
    verbose: bool,

    #[arg(conflicts_with = "verbose")]
    quiet: bool,
}
```

### 5. 全局参数

```rust
#[derive(Parser)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// 详细输出
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}
// 用法: task --verbose add "任务"
// 用法: task add --verbose "任务"  // global 参数可以放在子命令前后
```

---

## 自动生成的帮助

Clap 自动生成专业的帮助信息：

```bash
$ task --help
命令行待办事项管理器

Usage: task <COMMAND>

Commands:
  add     添加新任务
  list    列出任务
  done    标记任务完成
  remove  删除任务 [aliases: rm]
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  配置文件路径
  -v, --verbose          详细输出
  -h, --help             Print help
  -V, --version          Print version

$ task add --help
添加新任务

Usage: task add [OPTIONS] <TITLE>

Arguments:
  <TITLE>  任务标题

Options:
  -p, --priority <PRIORITY>  优先级 (low, medium, high) [default: medium]
  -d, --due <DUE>            截止日期
  -h, --help                 Print help
```

---

## 完整 task-cli 结构

### 项目结构

```
task-cli/
├── Cargo.toml
├── src/
│   ├── main.rs      # 入口点
│   ├── cli.rs       # Clap 定义
│   ├── task.rs      # Task 结构
│   ├── storage.rs   # 文件操作
│   └── config.rs    # 配置管理
└── config.toml      # 默认配置
```

### cli.rs

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "task")]
#[command(version, about = "命令行待办事项管理器")]
pub struct Cli {
    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 添加新任务
    Add {
        /// 任务标题
        title: String,

        #[arg(short, long, default_value = "medium")]
        priority: Priority,

        #[arg(short, long)]
        due: Option<String>,
    },

    /// 列出任务
    List {
        #[arg(short, long)]
        status: Option<Status>,

        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 标记任务完成
    Done { id: u32 },

    /// 删除任务
    #[command(alias = "rm")]
    Remove {
        id: u32,
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Clone, ValueEnum)]
pub enum Status {
    Pending,
    InProgress,
    Done,
}
```

### main.rs

```rust
mod cli;
mod task;
mod storage;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    // 加载配置
    let config_path = cli.config.as_deref().unwrap_or("~/.task-cli/config.toml");

    match cli.command {
        Commands::Add { title, priority, due } => {
            // 实现添加逻辑
        }
        Commands::List { status, limit } => {
            // 实现列表逻辑
        }
        Commands::Done { id } => {
            // 实现完成逻辑
        }
        Commands::Remove { id, force } => {
            // 实现删除逻辑
        }
    }
}
```

---

## 与手动解析对比

### 手动解析（第 6 章方式）

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                eprintln!("错误: 缺少任务标题");
                return;
            }
            // 解析可选参数...复杂！
        }
        "--help" | "-h" => print_help(),
        _ => eprintln!("未知命令"),
    }
}

fn print_help() {
    println!("用法: task <command> [options]");
    println!("命令:");
    println!("  add <title>  添加任务");
    // 手动维护帮助信息...容易过时
}
```

### Clap 方式

```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "任务管理器")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 添加任务
    Add { title: String },
}

fn main() {
    let cli = Cli::parse();
    // 类型安全，自动帮助，自动错误处理
}
```

**对比**：

| 方面 | 手动解析 | Clap |
|------|---------|------|
| 代码量 | 多 | 少 |
| 帮助信息 | 手动维护 | 自动生成 |
| 错误处理 | 手动 | 自动 |
| 类型安全 | 需要手动转换 | 编译时检查 |
| 补全脚本 | 无 | 自动生成 |
| 可维护性 | 差 | 好 |

---

## 最佳实践

### 参数设计

| 场景 | 推荐做法 |
|------|---------|
| 主要输入 | 位置参数 |
| 可选配置 | 长选项 `--config` |
| 常用开关 | 短选项 `-v` |
| 互斥选项 | 使用 `conflicts_with` |
| 默认值 | 使用 `default_value` |
| 复杂工具 | 使用子命令 |

### 子命令设计

```rust
// 好：动词形式，直观
enum Commands {
    Add { ... },
    List { ... },
    Remove { ... },
}

// 不好：名词形式，不直观
enum Commands {
    Task { ... },
    Tasks { ... },
}
```

### 帮助信息

```rust
#[derive(Parser)]
struct Cli {
    /// 简短描述（一行）
    #[arg(long, help = "详细描述（可以更长）")]
    option: String,
}
```

### 常见陷阱

| 陷阱 | 问题 | 解决 |
|------|------|------|
| 忘记 `derive` feature | 编译错误 | `features = ["derive"]` |
| 短选项冲突 | `-h` 被帮助占用 | 用其他字母或只用长选项 |
| 可选参数无默认值 | 运行时 panic | 用 `Option<T>` 或设默认值 |
| 位置参数顺序 | 混乱的用户体验 | 必需参数在前 |

---

## CLI 工具线在此汇合

到这里，task-cli 开发完成：
- 第 6 章：基础结构体
- 第 7 章：枚举状态
- 第 8 章：文件持久化
- 第 9 章：Display trait
- 第 11 章：闭包过滤
- 第 14 章：JSON 存储
- 第 15 章：Clap 完善

---

## 要点回顾

1. **Clap derive 模式**：声明式定义，代码即文档
2. **子命令**：使用枚举组织命令结构
3. **属性系统**：`#[arg(...)]` 和 `#[command(...)]` 配置行为
4. **自动生成**：帮助信息、错误消息、补全脚本
5. **类型安全**：编译时检查参数类型

---

## 练习

1. **基础**：为 task-cli 添加 `edit` 子命令，修改任务标题
2. **进阶**：添加 `--config` 全局参数，指定配置文件路径
3. **挑战**：实现命令补全脚本生成（`task completions bash`）

---

## 扩展阅读

- [Clap 官方文档](https://docs.rs/clap)
- [Clap derive 教程](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [Clap Cookbook](https://docs.rs/clap/latest/clap/_cookbook/index.html)
- [Command Line Applications in Rust](https://rust-cli.github.io/book/)

---

## 下一章预告

命令行工具做好了，下一章学习文件与 I/O，构建更多实用工具。
