# 第 24 章：HTTP 客户端

## 本章目标

学完本章，你将能够：
- 使用 reqwest 发送各种 HTTP 请求
- 处理 JSON 请求和响应
- 配置超时、重试和错误处理
- 实现 api-cli 命令行客户端

---

## 前置知识

- 第 14 章：Serde 序列化
- 第 22 章：异步编程基础

---

## reqwest：Rust 最流行的 HTTP 客户端

### 背景介绍

**reqwest** 由 Sean McArthur（Hyper 作者）开发，是 Rust 生态中下载量最高的 HTTP 客户端库。

**名字含义**：reqwest = request（请求）的变体拼写，暗示这是一个"更好的" request 库。

**生态地位**：
- 构建在 hyper（底层 HTTP 实现）之上
- 与 tokio 深度集成
- 支持同步和异步两种 API

### 设计理念

1. **开箱即用**：合理的默认配置
2. **类型安全**：编译时捕获错误
3. **异步优先**：原生支持 async/await
4. **可扩展**：支持中间件和自定义配置

### 与其他语言对比

```python
# Python - requests
import requests
response = requests.get('https://api.example.com')
data = response.json()
```

```javascript
// JavaScript - fetch
const response = await fetch('https://api.example.com');
const data = await response.json();
```

```rust
// Rust - reqwest
let response = reqwest::get("https://api.example.com").await?;
let data: MyStruct = response.json().await?;
```

**关键差异**：

| 方面 | Python/JS | Rust reqwest |
|------|-----------|--------------|
| 类型安全 | 运行时检查 | 编译时检查 |
| JSON 解析 | 动态类型 | 反序列化到具体类型 |
| 错误处理 | 异常 | Result |
| 异步模型 | 各有不同 | async/await |

---

## 项目：api-cli - 命令行 HTTP 客户端

### 功能概览

一个类似 curl 的命令行 HTTP 客户端：

```bash
# GET 请求
$ api-cli get https://api.github.com/users/rust-lang
{
  "login": "rust-lang",
  "id": 5430905,
  ...
}

# POST 请求带 JSON
$ api-cli post https://httpbin.org/post \
    --json '{"name": "Rust", "version": "1.75"}'

# 自定义 Headers
$ api-cli get https://api.example.com/protected \
    -H "Authorization: Bearer token123"

# 超时和重试
$ api-cli get https://slow.api.com --timeout 10 --retry 3
```

### 为什么做这个项目？

1. **实用工具**：API 调试和测试
2. **综合练习**：异步、CLI、JSON 处理
3. **理解 HTTP**：请求/响应/状态码

---

## 核心概念

### 1. 依赖配置

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "cookies"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
```

### 2. 基本请求

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // 方式 1：快捷函数（每次创建新连接）
    let body = reqwest::get("https://httpbin.org/get")
        .await?
        .text()
        .await?;

    // 方式 2：使用 Client（推荐，复用连接池）
    let client = Client::new();
    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await?;

    println!("状态: {}", response.status());
    println!("响应: {}", response.text().await?);

    Ok(())
}
```

### 3. Client 配置

```rust
use reqwest::Client;
use std::time::Duration;

// 创建配置好的 Client
let client = Client::builder()
    // 超时设置
    .timeout(Duration::from_secs(10))
    .connect_timeout(Duration::from_secs(5))

    // 重定向
    .redirect(reqwest::redirect::Policy::limited(5))

    // User-Agent
    .user_agent("api-cli/1.0")

    // 代理
    .proxy(reqwest::Proxy::http("http://proxy:8080")?)

    // TLS 配置
    .danger_accept_invalid_certs(false)  // 生产环境不要设为 true

    .build()?;
```

**为什么要复用 Client？**
- 内部维护连接池
- 避免重复 TCP 握手和 TLS 协商
- 显著提升性能

### 4. 各种 HTTP 方法

```rust
let client = Client::new();

// GET
let resp = client.get("https://api.example.com/users").send().await?;

// POST
let resp = client.post("https://api.example.com/users")
    .json(&new_user)
    .send()
    .await?;

// PUT
let resp = client.put("https://api.example.com/users/1")
    .json(&updated_user)
    .send()
    .await?;

// PATCH
let resp = client.patch("https://api.example.com/users/1")
    .json(&partial_update)
    .send()
    .await?;

// DELETE
let resp = client.delete("https://api.example.com/users/1").send().await?;

// HEAD（只获取响应头）
let resp = client.head("https://api.example.com/resource").send().await?;
```

### 5. 请求体

```rust
// JSON 请求体（最常用）
#[derive(Serialize)]
struct CreateUser {
    name: String,
    email: String,
}

let user = CreateUser {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

let resp = client.post(url)
    .json(&user)  // 自动设置 Content-Type: application/json
    .send()
    .await?;

// 表单数据
let params = [("username", "alice"), ("password", "secret")];
let resp = client.post(url)
    .form(&params)  // Content-Type: application/x-www-form-urlencoded
    .send()
    .await?;

// 原始字节
let resp = client.post(url)
    .body("raw bytes")
    .send()
    .await?;

// 多部分表单（文件上传）
use reqwest::multipart;

let form = multipart::Form::new()
    .text("field1", "value1")
    .file("file", "path/to/file.txt").await?;

let resp = client.post(url)
    .multipart(form)
    .send()
    .await?;
```

### 6. 请求头

```rust
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

// 方式 1：单个 header
let resp = client.get(url)
    .header("X-Custom-Header", "value")
    .header(AUTHORIZATION, "Bearer token123")
    .send()
    .await?;

// 方式 2：HeaderMap（多个 headers）
let mut headers = HeaderMap::new();
headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer token"));
headers.insert("X-API-Key", HeaderValue::from_static("key123"));

let resp = client.get(url)
    .headers(headers)
    .send()
    .await?;

// 方式 3：默认 headers（Client 级别）
let client = Client::builder()
    .default_headers(headers)
    .build()?;
```

### 7. 响应处理

```rust
let response = client.get(url).send().await?;

// 状态码
let status = response.status();
println!("状态码: {}", status);  // 200
println!("是否成功: {}", status.is_success());  // true

// 响应头
let headers = response.headers();
if let Some(content_type) = headers.get("content-type") {
    println!("Content-Type: {:?}", content_type);
}

// 响应体
// 方式 1：文本
let text = response.text().await?;

// 方式 2：JSON（需要 json feature）
#[derive(Deserialize)]
struct User {
    id: i32,
    name: String,
}
let user: User = response.json().await?;

// 方式 3：字节
let bytes = response.bytes().await?;

// 方式 4：流式读取（大文件）
let mut stream = response.bytes_stream();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // 处理 chunk...
}
```

### 8. 错误处理

```rust
use reqwest::Error;

async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;

    // 检查状态码
    let response = response.error_for_status()?;  // 4xx/5xx 转为错误

    Ok(response.text().await?)
}

// 更精细的错误处理
async fn robust_fetch(url: &str) -> Result<String, String> {
    let response = reqwest::get(url).await.map_err(|e| {
        if e.is_connect() {
            "连接失败".to_string()
        } else if e.is_timeout() {
            "请求超时".to_string()
        } else if e.is_request() {
            format!("请求错误: {}", e)
        } else {
            format!("未知错误: {}", e)
        }
    })?;

    match response.status().as_u16() {
        200..=299 => Ok(response.text().await.unwrap_or_default()),
        401 => Err("未授权，请检查认证信息".to_string()),
        404 => Err("资源不存在".to_string()),
        500..=599 => Err("服务器错误".to_string()),
        code => Err(format!("HTTP 错误: {}", code)),
    }
}
```

### 9. 重试机制

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn fetch_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, String> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok(resp.text().await.unwrap_or_default());
            }
            Ok(resp) if resp.status().is_server_error() && attempts < max_retries => {
                // 服务器错误，重试
                let delay = Duration::from_secs(2u64.pow(attempts));  // 指数退避
                eprintln!("服务器错误，{}秒后重试...", delay.as_secs());
                sleep(delay).await;
            }
            Ok(resp) => {
                return Err(format!("请求失败: {}", resp.status()));
            }
            Err(e) if e.is_timeout() && attempts < max_retries => {
                eprintln!("超时，重试...");
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(format!("请求错误: {}", e));
            }
        }
    }
}
```

---

## api-cli 完整实现

```rust
use clap::{Parser, Subcommand};
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use serde_json::Value;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "api-cli")]
#[command(about = "命令行 HTTP 客户端")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 超时时间（秒）
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// 自定义 Header（可多次使用）
    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    /// 不验证 SSL 证书
    #[arg(long)]
    insecure: bool,

    /// 输出详细信息
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 发送 GET 请求
    Get {
        url: String,
    },
    /// 发送 POST 请求
    Post {
        url: String,
        /// JSON 请求体
        #[arg(long)]
        json: Option<String>,
        /// 表单数据
        #[arg(long)]
        form: Option<String>,
    },
    /// 发送 PUT 请求
    Put {
        url: String,
        #[arg(long)]
        json: Option<String>,
    },
    /// 发送 DELETE 请求
    Delete {
        url: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // 构建 Client
    let client = Client::builder()
        .timeout(Duration::from_secs(cli.timeout))
        .danger_accept_invalid_certs(cli.insecure)
        .build()
        .expect("Failed to create HTTP client");

    // 解析 headers
    let headers = parse_headers(&cli.headers);

    // 执行请求
    let result = match cli.command {
        Commands::Get { url } => {
            execute_get(&client, &url, headers, cli.verbose).await
        }
        Commands::Post { url, json, form } => {
            execute_post(&client, &url, headers, json, form, cli.verbose).await
        }
        Commands::Put { url, json } => {
            execute_put(&client, &url, headers, json, cli.verbose).await
        }
        Commands::Delete { url } => {
            execute_delete(&client, &url, headers, cli.verbose).await
        }
    };

    match result {
        Ok(()) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}

fn parse_headers(headers: &[String]) -> HeaderMap {
    let mut map = HeaderMap::new();

    for h in headers {
        if let Some((key, value)) = h.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            if let (Ok(name), Ok(val)) = (
                HeaderName::try_from(key),
                HeaderValue::try_from(value),
            ) {
                map.insert(name, val);
            }
        }
    }

    map
}

async fn execute_get(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        eprintln!("GET {}", url);
    }

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    print_response(response, verbose).await
}

async fn execute_post(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    json: Option<String>,
    form: Option<String>,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        eprintln!("POST {}", url);
    }

    let mut request = client.post(url).headers(headers);

    if let Some(json_str) = json {
        let value: Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("无效的 JSON: {}", e))?;
        request = request.json(&value);
    } else if let Some(form_str) = form {
        // 解析 "key=value&key2=value2" 格式
        let params: Vec<(&str, &str)> = form_str
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .collect();
        request = request.form(&params);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    print_response(response, verbose).await
}

async fn execute_put(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    json: Option<String>,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        eprintln!("PUT {}", url);
    }

    let mut request = client.put(url).headers(headers);

    if let Some(json_str) = json {
        let value: Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("无效的 JSON: {}", e))?;
        request = request.json(&value);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    print_response(response, verbose).await
}

async fn execute_delete(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    verbose: bool,
) -> Result<(), String> {
    if verbose {
        eprintln!("DELETE {}", url);
    }

    let response = client
        .delete(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    print_response(response, verbose).await
}

async fn print_response(response: reqwest::Response, verbose: bool) -> Result<(), String> {
    let status = response.status();

    if verbose {
        eprintln!("状态: {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));

        eprintln!("\n响应头:");
        for (key, value) in response.headers() {
            eprintln!("  {}: {:?}", key, value);
        }
        eprintln!();
    }

    let body = response.text().await.map_err(|e| format!("读取响应失败: {}", e))?;

    // 尝试格式化 JSON
    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    } else {
        println!("{}", body);
    }

    if !status.is_success() {
        return Err(format!("请求失败: {}", status));
    }

    Ok(())
}
```

---

## 最佳实践

### 选择合适的方法

| 场景 | 推荐做法 |
|------|---------|
| 单次请求 | `reqwest::get()` 快捷方法 |
| 多次请求 | 复用 `Client` 实例 |
| 需要配置 | `Client::builder()` |
| 高并发 | 单例 Client + 连接池 |

### 错误处理

| 场景 | 推荐做法 |
|------|---------|
| API 调用 | 检查 status + error_for_status |
| 不可靠网络 | 重试 + 指数退避 |
| 超时敏感 | 设置合理的超时时间 |
| 调试 | 启用 verbose 日志 |

### 常见陷阱

| 陷阱 | 问题 | 解决方案 |
|------|------|---------|
| 每次创建 Client | 性能差 | 复用 Client |
| 忘记 await | 请求不会发送 | 检查 await |
| 忽略状态码 | 错误被忽略 | error_for_status |
| 无限等待 | 请求挂起 | 设置超时 |
| 敏感信息日志 | 安全风险 | 过滤敏感 headers |

---

## 要点回顾

1. **reqwest** 是 Rust 最流行的 HTTP 客户端
2. **复用 Client** 利用连接池提升性能
3. **json feature** 简化 JSON 请求/响应
4. **error_for_status** 将 4xx/5xx 转为错误
5. **超时和重试** 处理不可靠网络
6. **async/await** 处理异步 I/O

---

## 练习

### 练习 1：网站可用性检查

实现一个检查多个网站是否在线的工具：
```bash
$ site-check https://google.com https://github.com
google.com    ✓ 200 OK (234ms)
github.com    ✓ 200 OK (456ms)
```

### 练习 2：并发下载

实现并发下载多个 URL 并显示进度：
```bash
$ multi-download urls.txt --parallel 5
[1/10] https://example.com/file1.zip  ████████░░  80%
[2/10] https://example.com/file2.zip  ██████████ 100%
```

### 练习 3：API 测试工具

实现一个支持从文件读取测试用例的 API 测试工具：
```yaml
# tests.yaml
- name: "Get users"
  method: GET
  url: "/users"
  expect:
    status: 200
```

### 练习 4：OAuth 客户端

实现 OAuth 2.0 客户端流程：
- Authorization Code 流程
- Token 刷新
- 安全存储 token

---

## 扩展阅读

- [reqwest 官方文档](https://docs.rs/reqwest)
- [reqwest GitHub](https://github.com/seanmonstar/reqwest)
- [HTTP 状态码参考](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
- [Hyper 底层 HTTP 库](https://docs.rs/hyper)
- [surf - 另一个 HTTP 客户端](https://docs.rs/surf)

---

## 下一章预告

学完了 Web 框架和 HTTP 客户端，下一章综合所有知识，实现一个完整的项目。
