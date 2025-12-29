# 附录 C：常用 Crate 速查表

## 本附录内容

按场景分类的常用 crate 速查，帮助你快速找到合适的工具。

---

## 序列化与数据格式

### serde - 序列化框架

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # JSON
toml = "0.8"        # TOML
serde_yaml = "0.9"  # YAML
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    port: u16,
}

// JSON
let json = serde_json::to_string(&config)?;
let config: Config = serde_json::from_str(&json)?;

// TOML
let toml = toml::to_string(&config)?;
let config: Config = toml::from_str(&toml)?;
```

### csv - CSV 文件处理

```toml
[dependencies]
csv = "1.3"
```

```rust
use csv::ReaderBuilder;

let mut rdr = ReaderBuilder::new()
    .has_headers(true)
    .from_path("data.csv")?;

for result in rdr.deserialize() {
    let record: MyRecord = result?;
    println!("{:?}", record);
}
```

---

## 命令行工具

### clap - CLI 参数解析

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
```

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "myapp", about = "A CLI tool")]
struct Cli {
    /// Input file
    #[arg(short, long)]
    input: String,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// Number of times
    #[arg(short, long, default_value_t = 1)]
    count: u32,
}

fn main() {
    let args = Cli::parse();
    println!("Input: {}", args.input);
}
```

### colored - 终端彩色输出

```toml
[dependencies]
colored = "2.1"
```

```rust
use colored::*;

println!("{}", "Success!".green().bold());
println!("{}", "Warning!".yellow());
println!("{}", "Error!".red().on_white());
```

### indicatif - 进度条

```toml
[dependencies]
indicatif = "0.17"
```

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("{bar:40.cyan/blue} {pos}/{len}")?);

for _ in 0..100 {
    pb.inc(1);
    std::thread::sleep(std::time::Duration::from_millis(10));
}
pb.finish_with_message("Done!");
```

### dialoguer - 交互式提示

```toml
[dependencies]
dialoguer = "0.11"
```

```rust
use dialoguer::{Confirm, Input, Select};

let name: String = Input::new()
    .with_prompt("Your name")
    .interact_text()?;

let proceed = Confirm::new()
    .with_prompt("Do you want to continue?")
    .interact()?;

let selection = Select::new()
    .with_prompt("Pick an option")
    .items(&["Option A", "Option B", "Option C"])
    .interact()?;
```

---

## 异步运行时

### tokio - 异步运行时

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

```rust
#[tokio::main]
async fn main() {
    let result = fetch_data().await;
    println!("{:?}", result);
}

async fn fetch_data() -> Result<String, reqwest::Error> {
    let resp = reqwest::get("https://api.example.com").await?;
    resp.text().await
}
```

**常用 features**：
- `rt` - 运行时
- `rt-multi-thread` - 多线程运行时
- `macros` - `#[tokio::main]` 等宏
- `time` - 时间相关功能
- `fs` - 异步文件系统
- `net` - 异步网络
- `full` - 全部功能

---

## 网络与 HTTP

### reqwest - HTTP 客户端

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

```rust
// GET 请求
let resp = reqwest::get("https://httpbin.org/get").await?;
let body = resp.text().await?;

// POST JSON
let client = reqwest::Client::new();
let resp = client.post("https://httpbin.org/post")
    .json(&serde_json::json!({"key": "value"}))
    .send()
    .await?;
```

### axum - Web 框架

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
```

```rust
use axum::{routing::get, Router, Json};

async fn hello() -> &'static str {
    "Hello, World!"
}

async fn json() -> Json<serde_json::Value> {
    Json(serde_json::json!({"message": "hello"}))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/json", get(json));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## 错误处理

### thiserror - 自定义错误

```toml
[dependencies]
thiserror = "2.0"
```

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Unknown error")]
    Unknown,
}
```

### anyhow - 简化错误处理

```toml
[dependencies]
anyhow = "1.0"
```

```rust
use anyhow::{Context, Result, bail};

fn read_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse config")?;

    if config.name.is_empty() {
        bail!("Config name cannot be empty");
    }

    Ok(config)
}
```

**选择建议**：
- **库代码**：用 `thiserror` 定义具体错误类型
- **应用代码**：用 `anyhow` 简化错误处理

---

## 日期与时间

### chrono - 日期时间处理

```toml
[dependencies]
chrono = "0.4"
```

```rust
use chrono::{DateTime, Local, Utc, Duration, NaiveDate};

// 当前时间
let now_utc: DateTime<Utc> = Utc::now();
let now_local: DateTime<Local> = Local::now();

// 格式化
let formatted = now_local.format("%Y-%m-%d %H:%M:%S").to_string();

// 解析
let parsed = NaiveDate::parse_from_str("2024-01-15", "%Y-%m-%d")?;

// 计算
let tomorrow = now_utc + Duration::days(1);
let diff = tomorrow - now_utc;
```

### time - 轻量日期时间

```toml
[dependencies]
time = { version = "0.3", features = ["formatting", "parsing"] }
```

```rust
use time::{OffsetDateTime, format_description};

let now = OffsetDateTime::now_utc();
let format = format_description::parse("[year]-[month]-[day]")?;
let formatted = now.format(&format)?;
```

---

## 日志

### tracing - 结构化日志

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
use tracing::{info, warn, error, debug, instrument, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // 初始化
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Application started");
    warn!(port = 8080, "Using default port");
    error!(?error, "Something went wrong");
}

#[instrument]
fn process_request(id: u32) {
    debug!("Processing request");
    // 自动记录函数进入/退出
}
```

### log + env_logger - 传统日志

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{info, warn, error};

fn main() {
    env_logger::init();  // RUST_LOG=info cargo run

    info!("Starting up");
    warn!("Something might be wrong");
    error!("Something went wrong!");
}
```

---

## 数据库

### sqlx - 异步 SQL

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
```

```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:pass@localhost/db").await?;

    // 编译时检查的查询
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    // 带参数
    let users = sqlx::query_as!(
        User,
        "SELECT id, name FROM users WHERE active = $1",
        true
    )
    .fetch_all(&pool)
    .await?;

    Ok(())
}
```

### sea-orm - ORM

```toml
[dependencies]
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-native-tls"] }
```

---

## 正则表达式

### regex - 正则表达式

```toml
[dependencies]
regex = "1.10"
```

```rust
use regex::Regex;

// 匹配
let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$")?;
assert!(re.is_match("2024-01-15"));

// 捕获
let re = Regex::new(r"(\w+)@(\w+\.\w+)")?;
if let Some(caps) = re.captures("user@example.com") {
    println!("User: {}", &caps[1]);    // user
    println!("Domain: {}", &caps[2]);  // example.com
}

// 替换
let result = re.replace_all(text, "[$1 at $2]");

// 查找所有
for cap in re.captures_iter(text) {
    println!("{}", &cap[0]);
}
```

---

## 随机数

### rand - 随机数生成

```toml
[dependencies]
rand = "0.8"
```

```rust
use rand::Rng;
use rand::seq::SliceRandom;

let mut rng = rand::thread_rng();

// 基础随机数
let n: u32 = rng.gen();
let n: f64 = rng.gen();

// 范围
let n: i32 = rng.gen_range(1..=100);

// 布尔
let b: bool = rng.gen_bool(0.5);

// 随机选择
let items = vec!["a", "b", "c"];
let choice = items.choose(&mut rng);

// 打乱
let mut v = vec![1, 2, 3, 4, 5];
v.shuffle(&mut rng);
```

---

## 文件系统

### walkdir - 目录遍历

```toml
[dependencies]
walkdir = "2.4"
```

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("/path/to/dir")
    .min_depth(1)
    .max_depth(3)
{
    let entry = entry?;
    if entry.file_type().is_file() {
        println!("{}", entry.path().display());
    }
}
```

### tempfile - 临时文件

```toml
[dependencies]
tempfile = "3.9"
```

```rust
use tempfile::{tempfile, tempdir, NamedTempFile};

// 临时文件（自动删除）
let mut file = tempfile()?;
writeln!(file, "Hello")?;

// 带名称的临时文件
let file = NamedTempFile::new()?;
println!("Path: {}", file.path().display());

// 临时目录
let dir = tempdir()?;
let file_path = dir.path().join("my_file.txt");
```

### glob - 文件模式匹配

```toml
[dependencies]
glob = "0.3"
```

```rust
use glob::glob;

for entry in glob("**/*.rs")? {
    let path = entry?;
    println!("{}", path.display());
}
```

---

## 并发与同步

### rayon - 数据并行

```toml
[dependencies]
rayon = "1.8"
```

```rust
use rayon::prelude::*;

// 并行迭代
let sum: i32 = (0..1000)
    .into_par_iter()
    .map(|x| x * x)
    .sum();

// 并行处理集合
let results: Vec<_> = items
    .par_iter()
    .map(|item| process(item))
    .collect();
```

### crossbeam - 并发原语

```toml
[dependencies]
crossbeam = "0.8"
```

```rust
use crossbeam::channel;
use crossbeam::scope;

// 多生产者多消费者 channel
let (tx, rx) = channel::unbounded();
tx.send("hello")?;
let msg = rx.recv()?;

// 作用域线程（可以借用栈数据）
crossbeam::scope(|s| {
    s.spawn(|_| println!("Thread 1"));
    s.spawn(|_| println!("Thread 2"));
}).unwrap();
```

### parking_lot - 高性能同步原语

```toml
[dependencies]
parking_lot = "0.12"
```

```rust
use parking_lot::{Mutex, RwLock};

let mutex = Mutex::new(0);
*mutex.lock() += 1;

let rwlock = RwLock::new(vec![]);
rwlock.read();  // 多个读者
rwlock.write(); // 独占写者
```

---

## 测试

### assert_cmd - 命令行测试

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
```

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli() {
    let mut cmd = Command::cargo_bin("myapp").unwrap();

    cmd.arg("--version")
       .assert()
       .success()
       .stdout(predicate::str::contains("1.0"));
}
```

### mockall - Mock 框架

```toml
[dev-dependencies]
mockall = "0.12"
```

```rust
use mockall::{automock, predicate::*};

#[automock]
trait Database {
    fn get(&self, key: &str) -> Option<String>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockDatabase::new();
    mock.expect_get()
        .with(eq("key"))
        .returning(|_| Some("value".into()));

    assert_eq!(mock.get("key"), Some("value".into()));
}
```

---

## 快速选择指南

| 场景 | 推荐 Crate |
|-----|-----------|
| JSON 处理 | `serde` + `serde_json` |
| CLI 参数 | `clap` |
| HTTP 客户端 | `reqwest` |
| Web 服务 | `axum` |
| 异步运行时 | `tokio` |
| 错误处理（库）| `thiserror` |
| 错误处理（应用）| `anyhow` |
| 日志 | `tracing` |
| 日期时间 | `chrono` |
| 数据库 | `sqlx` |
| 正则 | `regex` |
| 随机数 | `rand` |
| 并行计算 | `rayon` |
| 文件遍历 | `walkdir` |

---

## 发现更多 Crate

- [crates.io](https://crates.io) - 官方仓库
- [lib.rs](https://lib.rs) - 分类浏览
- [Blessed.rs](https://blessed.rs) - 社区推荐
- [Are We X Yet?](https://areweyet.rs) - 生态成熟度追踪
