// link-short: 短链接服务
// 使用 Axum 框架构建 REST API
//
// API:
//   POST /links          创建短链接
//   GET /:code           重定向到原始 URL
//   GET /links/:code/stats  查看统计

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

// 短链接记录
#[derive(Clone)]
struct LinkRecord {
    url: String,
    clicks: u64,
}

// 应用状态
struct AppState {
    links: RwLock<HashMap<String, LinkRecord>>,
    base_url: String,
}

// 请求/响应结构体
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
    // 创建共享状态
    let state = Arc::new(AppState {
        links: RwLock::new(HashMap::new()),
        base_url: "http://localhost:3000".to_string(),
    });

    // 构建路由
    // Axum 使用 Router 来定义路由
    // .route() 添加路由，第一个参数是路径，第二个是处理函数
    let app = Router::new()
        .route("/links", post(create_link))
        .route("/:code", get(redirect_link))
        .route("/links/:code/stats", get(get_stats))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("link-short 启动，监听 {}", addr);
    println!("\n使用示例:");
    println!("  创建短链接: curl -X POST http://localhost:3000/links -H 'Content-Type: application/json' -d '{{\"url\":\"https://github.com\"}}'");
    println!("  访问短链接: curl -L http://localhost:3000/<code>");
    println!("  查看统计:   curl http://localhost:3000/links/<code>/stats\n");

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 创建短链接
///
/// Axum 的 Extractor 模式：
/// - State<T>: 从应用状态中提取
/// - Json<T>: 从请求体解析 JSON
async fn create_link(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLinkRequest>,
) -> impl IntoResponse {
    // 生成随机短码
    let code = generate_code();

    // 创建记录
    let record = LinkRecord {
        url: req.url,
        clicks: 0,
    };

    // 存储
    state.links.write().await.insert(code.clone(), record);

    // 返回响应
    // Json 实现了 IntoResponse，自动设置 Content-Type
    let response = CreateLinkResponse {
        short_url: format!("{}/{}", state.base_url, code),
        code,
    };

    (StatusCode::CREATED, Json(response))
}

/// 重定向到原始 URL
///
/// Path<T>: 从 URL 路径中提取参数
async fn redirect_link(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    // 先尝试获取写锁来更新点击数
    let mut links = state.links.write().await;

    if let Some(record) = links.get_mut(&code) {
        record.clicks += 1;
        let url = record.url.clone();
        drop(links); // 释放锁

        // Redirect 是 Axum 提供的重定向响应
        Ok(Redirect::temporary(&url))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        ))
    }
}

/// 获取链接统计
async fn get_stats(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let links = state.links.read().await;

    if let Some(record) = links.get(&code) {
        Ok(Json(StatsResponse {
            code: code.clone(),
            url: record.url.clone(),
            clicks: record.clicks,
        }))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Link not found".to_string(),
            }),
        ))
    }
}

/// 生成 6 位随机短码
fn generate_code() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
