# 第 23 章：Web 框架实战（Axum）

## 本章目标

学完本章，你将能够：
- 理解 Axum 的设计理念和独特价值
- 使用 Axum 构建 REST API
- 掌握 Extractor 模式处理请求
- 实现中间件和错误处理
- 构建 link-short 短链接服务

---

## 前置知识

- 第 22 章：异步编程与 Tokio
- 第 14 章：Serde（JSON 序列化）

---

## Axum：类型安全的 Web 框架

### 背景介绍

**Axum** 由 **Tokio 团队**开发，是 Rust 生态中最新一代的 Web 框架。

**名字由来**：Axum 可能源自 "Axis"（轴心）的变体，寓意"核心、关键"。

**生态地位**：
- 由 Tokio 官方团队维护，与 Tokio 深度集成
- 2021 年首次发布，快速成长为最受欢迎的 Rust Web 框架之一
- 被 Cloudflare、Shopify 等公司采用
- 构建在 Tower 生态之上，可复用大量中间件

**为什么选择 Axum？**
- **类型安全**：编译时检查路由和参数
- **无宏魔法**：不依赖过程宏，IDE 支持好
- **Tower 生态**：复用现有的中间件和服务
- **人体工程学**：API 设计直观

---

### 设计理念

#### Extractor 模式

Axum 的核心创新是 **Extractor（提取器）模式**：

```rust
// 传统方式（如 Java Spring）
@PostMapping("/users")
public User createUser(@RequestBody UserDto dto, @RequestHeader("Auth") String token) {
    // 手动验证和转换
}

// Axum 的 Extractor 方式
async fn create_user(
    Json(dto): Json<UserDto>,           // 自动从 body 解析 JSON
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,  // 自动从 header 提取
) -> impl IntoResponse {
    // dto 和 auth 已经是正确的类型
}
```

**Extractor 的魔力**：
- **自动解析**：参数类型决定如何提取数据
- **编译时检查**：类型不匹配在编译时报错
- **组合性**：多个 Extractor 可以组合使用
- **可扩展**：自定义 Extractor 实现特定逻辑

#### Tower 服务抽象

Axum 构建在 **Tower** 之上，Tower 定义了通用的服务接口：

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, req: Request) -> Self::Future;
}
```

**好处**：
- 中间件可以在不同框架间复用
- 超时、重试、限流等功能开箱即用
- 服务可以组合和嵌套

---

### 为什么 Rust Web 框架很特别？

#### 编译时路由检查

```rust
// Axum - 路由参数类型在编译时检查
Router::new()
    .route("/users/:id", get(get_user))
    .route("/users", post(create_user));

async fn get_user(Path(id): Path<u32>) -> impl IntoResponse {
    // id 已经是 u32，解析失败会返回 400
}

// 如果 get_user 的签名不匹配路由，编译时报错
```

#### 零成本抽象

Axum 的 Extractor 在编译时生成代码，运行时没有反射开销：

```rust
// 这些 Extractor 都在编译时确定
async fn handler(
    Path(id): Path<u32>,
    Query(params): Query<SearchParams>,
    Json(body): Json<CreateRequest>,
) -> impl IntoResponse {
    // 所有解析逻辑在编译时生成
}
```

---

## 核心概念

### 1. 基本结构

```toml
# Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
use axum::{
    routing::{get, post},
    Router, Json,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 创建路由
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));

    // 启动服务器
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("监听端口 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn create_user(Json(payload): Json<CreateUser>) -> Json<User> {
    let user = User {
        id: 1,
        name: payload.name,
    };
    Json(user)
}

#[derive(serde::Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(serde::Serialize)]
struct User {
    id: u64,
    name: String,
}
```

### 2. Extractor 详解

#### Path - 路径参数

```rust
use axum::extract::Path;

// 单个参数
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}

// 多个参数
async fn get_post(Path((user_id, post_id)): Path<(u32, u32)>) -> String {
    format!("User {} Post {}", user_id, post_id)
}

// 使用结构体
#[derive(Deserialize)]
struct PostParams {
    user_id: u32,
    post_id: u32,
}

async fn get_post_v2(Path(params): Path<PostParams>) -> String {
    format!("User {} Post {}", params.user_id, params.post_id)
}

// 路由定义
Router::new()
    .route("/users/:id", get(get_user))
    .route("/users/:user_id/posts/:post_id", get(get_post))
```

#### Query - 查询参数

```rust
use axum::extract::Query;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

async fn list_users(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(10);
    format!("Page {} with {} items", page, per_page)
}

// GET /users?page=2&per_page=20
```

#### Json - 请求/响应体

```rust
use axum::Json;

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    email: String,
}

async fn create_user(Json(input): Json<CreateUser>) -> Json<User> {
    let user = User {
        id: 1,
        email: input.email,
    };
    Json(user)
}
```

#### State - 共享状态

```rust
use axum::extract::State;
use std::sync::Arc;

struct AppState {
    db_pool: PgPool,
    config: Config,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        db_pool: create_pool().await,
        config: load_config(),
    });

    let app = Router::new()
        .route("/users", get(list_users))
        .with_state(state);
}

async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<User>> {
    let users = query_users(&state.db_pool).await;
    Json(users)
}
```

### 3. 响应处理

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Redirect},
    Json,
};

// 简单响应
async fn plain_text() -> &'static str {
    "Hello"
}

// JSON 响应
async fn json_response() -> Json<User> {
    Json(User { id: 1, name: "Alice".into() })
}

// 带状态码
async fn with_status() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Created")
}

// 重定向
async fn redirect() -> Redirect {
    Redirect::to("https://example.com")
}

// 自定义响应
async fn custom_response() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("X-Custom", "value")
        .body("Custom body".into())
        .unwrap()
}
```

### 4. 错误处理

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// 定义应用错误类型
enum AppError {
    NotFound,
    BadRequest(String),
    Internal(String),
}

// 实现 IntoResponse，让错误可以作为响应返回
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
        };

        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

// 在 handler 中使用
async fn get_user(Path(id): Path<u32>) -> Result<Json<User>, AppError> {
    let user = find_user(id).ok_or(AppError::NotFound)?;
    Ok(Json(user))
}
```

### 5. 中间件

```rust
use axum::{
    middleware::{self, Next},
    extract::Request,
    response::Response,
};
use std::time::Instant;

// 日志中间件
async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    println!("{} {} - {:?}", method, uri, duration);

    response
}

// 使用中间件
let app = Router::new()
    .route("/", get(root))
    .layer(middleware::from_fn(logging_middleware));

// Tower 中间件
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/", get(root))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());
```

---

## 项目：link-short（短链接服务）

### API 设计

```
POST /links          创建短链接
GET  /:code          重定向到原始 URL
GET  /links/:code/stats  查看统计
```

### 完整实现

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

// 数据结构
#[derive(Clone)]
struct LinkRecord {
    url: String,
    clicks: u64,
}

struct AppState {
    links: RwLock<HashMap<String, LinkRecord>>,
}

// 请求/响应类型
#[derive(Deserialize)]
struct CreateLinkRequest {
    url: String,
}

#[derive(Serialize)]
struct CreateLinkResponse {
    code: String,
    short_url: String,
}

#[derive(Serialize)]
struct StatsResponse {
    code: String,
    url: String,
    clicks: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        links: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/links", post(create_link))
        .route("/:code", get(redirect_link))
        .route("/links/:code/stats", get(get_stats))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("link-short 启动，监听端口 3000");
    axum::serve(listener, app).await.unwrap();
}

// 创建短链接
async fn create_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLinkRequest>,
) -> impl IntoResponse {
    let code = generate_code();
    let record = LinkRecord {
        url: req.url,
        clicks: 0,
    };

    state.links.write().await.insert(code.clone(), record);

    let response = CreateLinkResponse {
        short_url: format!("http://localhost:3000/{}", code),
        code,
    };

    (StatusCode::CREATED, Json(response))
}

// 重定向
async fn redirect_link(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let mut links = state.links.write().await;

    if let Some(record) = links.get_mut(&code) {
        record.clicks += 1;
        let url = record.url.clone();
        Ok(Redirect::temporary(&url))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".into(),
            }),
        ))
    }
}

// 获取统计
async fn get_stats(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let links = state.links.read().await;

    if let Some(record) = links.get(&code) {
        Ok(Json(StatsResponse {
            code,
            url: record.url.clone(),
            clicks: record.clicks,
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".into(),
            }),
        ))
    }
}

fn generate_code() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}
```

---

## 与 Spring Boot 对比

| 概念 | Spring Boot | Axum |
|------|-------------|------|
| 路由定义 | `@GetMapping`/`@PostMapping` | `Router::new().route()` |
| 控制器 | `@RestController` | 普通函数 |
| 依赖注入 | `@Autowired` | `State<T>` Extractor |
| 请求体 | `@RequestBody` | `Json<T>` |
| 路径参数 | `@PathVariable` | `Path<T>` |
| 查询参数 | `@RequestParam` | `Query<T>` |
| 请求头 | `@RequestHeader` | `TypedHeader<T>` |
| 中间件/AOP | `@Aspect` + Spring AOP | Tower `Layer` |
| 错误处理 | `@ExceptionHandler` | `impl IntoResponse` |

**Axum 的优势**：
- 无运行时反射，编译时检查
- 无注解宏，IDE 支持更好
- 更细粒度的控制

**Spring 的优势**：
- 生态成熟，文档丰富
- 约定优于配置，上手快
- 更多开箱即用的功能

---

## 最佳实践

### 项目结构

```
src/
├── main.rs           # 入口，启动服务器
├── routes/           # 路由定义
│   ├── mod.rs
│   ├── users.rs
│   └── posts.rs
├── handlers/         # 处理函数
│   ├── mod.rs
│   └── ...
├── models/           # 数据模型
├── extractors/       # 自定义 Extractor
└── error.rs          # 错误处理
```

### 状态管理

| 场景 | 推荐 |
|------|------|
| 数据库连接池 | `State<Arc<Pool>>` |
| 配置 | `State<Arc<Config>>` |
| 缓存 | `State<Arc<RwLock<Cache>>>` |
| 多种状态 | `State<Arc<AppState>>` 包含所有 |

### 错误处理

```rust
// 使用 thiserror 定义错误
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 转换为 HTTP 响应
    }
}
```

### 常见陷阱

| 陷阱 | 问题 | 解决 |
|------|------|------|
| State 类型不匹配 | 编译错误 | 确保 `with_state` 类型与 Extractor 一致 |
| Extractor 顺序 | 某些 Extractor 消耗 body | `Json` 放最后 |
| 忘记 `#[derive(Deserialize)]` | 无法解析请求 | 为请求类型添加 derive |
| 路由冲突 | 静态路由被动态路由覆盖 | 静态路由放前面 |

---

## 要点回顾

1. **Axum 是 Tokio 官方的 Web 框架**
2. **Extractor 模式**：类型决定如何提取数据
3. **Tower 生态**：可复用大量中间件
4. **编译时安全**：路由和参数在编译时检查
5. **组合式设计**：Router、Handler、Middleware 可灵活组合

---

## 练习

1. **基础**：实现一个简单的 TODO API（增删改查）
2. **进阶**：为 link-short 添加数据库持久化（SQLite）
3. **挑战**：实现用户认证中间件（JWT）

---

## 扩展阅读

- [Axum 官方文档](https://docs.rs/axum)
- [Axum 示例代码](https://github.com/tokio-rs/axum/tree/main/examples)
- [Tower 服务生态](https://github.com/tower-rs/tower)
- [tower-http 中间件](https://docs.rs/tower-http)

---

## 下一章预告

服务端开发搞定了，下一章学习如何调用外部 API——使用 reqwest 构建 HTTP 客户端。
