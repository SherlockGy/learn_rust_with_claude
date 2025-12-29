// api-cli: HTTP API 命令行客户端
// 使用 reqwest 发送 HTTP 请求
//
// 用法:
//   api-cli get <URL>
//   api-cli post <URL> --json '{"key": "value"}'
//   api-cli get <URL> -H "Authorization: Bearer token"

use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "api-cli")]
#[command(about = "HTTP API 命令行客户端")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 发送 GET 请求
    Get {
        /// 请求 URL
        url: String,

        /// 自定义请求头 (格式: "Name: Value")
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },

    /// 发送 POST 请求
    Post {
        /// 请求 URL
        url: String,

        /// JSON 请求体
        #[arg(long)]
        json: Option<String>,

        /// 自定义请求头
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },

    /// 发送 PUT 请求
    Put {
        /// 请求 URL
        url: String,

        /// JSON 请求体
        #[arg(long)]
        json: Option<String>,

        /// 自定义请求头
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },

    /// 发送 DELETE 请求
    Delete {
        /// 请求 URL
        url: String,

        /// 自定义请求头
        #[arg(short = 'H', long = "header")]
        headers: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // reqwest::Client 是可复用的，内部维护连接池
    let client = Client::new();

    let result = match cli.command {
        Commands::Get { url, headers } => do_get(&client, &url, &headers).await,

        Commands::Post { url, json, headers } => do_post(&client, &url, json, &headers).await,

        Commands::Put { url, json, headers } => do_put(&client, &url, json, &headers).await,

        Commands::Delete { url, headers } => do_delete(&client, &url, &headers).await,
    };

    if let Err(e) = result {
        eprintln!("请求失败: {}", e);
        std::process::exit(1);
    }
}

/// 发送 GET 请求
async fn do_get(client: &Client, url: &str, headers: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_headers = parse_headers(headers);

    let mut req = client.get(url);

    for (name, value) in &parsed_headers {
        req = req.header(name.as_str(), value.as_str());
    }

    let response = req.send().await?;

    print_response(response).await
}

/// 发送 POST 请求
async fn do_post(
    client: &Client,
    url: &str,
    json: Option<String>,
    headers: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_headers = parse_headers(headers);

    let mut req = client.post(url);

    for (name, value) in &parsed_headers {
        req = req.header(name.as_str(), value.as_str());
    }

    if let Some(body) = json {
        let value: Value = serde_json::from_str(&body)?;
        req = req.json(&value);
    }

    let response = req.send().await?;

    print_response(response).await
}

/// 发送 PUT 请求
async fn do_put(
    client: &Client,
    url: &str,
    json: Option<String>,
    headers: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_headers = parse_headers(headers);

    let mut req = client.put(url);

    for (name, value) in &parsed_headers {
        req = req.header(name.as_str(), value.as_str());
    }

    if let Some(body) = json {
        let value: Value = serde_json::from_str(&body)?;
        req = req.json(&value);
    }

    let response = req.send().await?;

    print_response(response).await
}

/// 发送 DELETE 请求
async fn do_delete(client: &Client, url: &str, headers: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_headers = parse_headers(headers);

    let mut req = client.delete(url);

    for (name, value) in &parsed_headers {
        req = req.header(name.as_str(), value.as_str());
    }

    let response = req.send().await?;

    print_response(response).await
}

/// 解析请求头
fn parse_headers(headers: &[String]) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|h| {
            let parts: Vec<&str> = h.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// 打印响应
async fn print_response(response: reqwest::Response) -> Result<(), Box<dyn std::error::Error>> {
    let status = response.status();

    println!("Status: {}", status);
    println!();

    // 尝试解析为 JSON 并美化输出
    let text = response.text().await?;

    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{}", text);
    }

    Ok(())
}
