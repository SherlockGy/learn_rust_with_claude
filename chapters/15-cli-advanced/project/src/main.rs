//! task-cli v1.0 - Production-ready CLI with Clap

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser)]
#[command(name = "task")]
#[command(about = "命令行待办事项管理器", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 添加新任务
    Add {
        /// 任务内容
        title: Vec<String>,
        /// 优先级 (low/medium/high)
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// 列出所有任务
    List {
        /// 按状态过滤 (pending/done/all)
        #[arg(short, long, default_value = "all")]
        status: String,
    },
    /// 开始任务
    Start { id: u32 },
    /// 完成任务
    Done { id: u32 },
    /// 删除任务
    Remove { id: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status { Pending, InProgress, Done }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Priority { Low, Medium, High }

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
}

const DATA_FILE: &str = "tasks.json";

fn load() -> Vec<Task> {
    fs::read_to_string(DATA_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(tasks: &[Task]) {
    fs::write(DATA_FILE, serde_json::to_string_pretty(tasks).unwrap()).ok();
}

fn main() {
    let cli = Cli::parse();
    let mut tasks = load();

    match cli.command {
        Commands::Add { title, priority } => {
            let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            let title = title.join(" ");
            let priority = match priority.as_str() {
                "low" => Priority::Low,
                "high" => Priority::High,
                _ => Priority::Medium,
            };
            tasks.push(Task { id: next_id, title: title.clone(), status: Status::Pending, priority });
            println!("✓ 添加: {} (ID: {})", title, next_id);
        }
        Commands::List { status } => {
            let filtered: Vec<_> = tasks.iter().filter(|t| {
                match status.as_str() {
                    "pending" => matches!(t.status, Status::Pending | Status::InProgress),
                    "done" => matches!(t.status, Status::Done),
                    _ => true,
                }
            }).collect();

            if filtered.is_empty() {
                println!("没有任务");
            } else {
                println!("{:>3}  {:>8}  {:>6}  任务", "ID", "状态", "优先级");
                println!("{}", "-".repeat(50));
                for t in filtered {
                    let status = match t.status {
                        Status::Pending => "待办",
                        Status::InProgress => "进行中",
                        Status::Done => "完成",
                    };
                    let priority = match t.priority {
                        Priority::Low => "低",
                        Priority::Medium => "中",
                        Priority::High => "高",
                    };
                    println!("{:>3}  {:>8}  {:>6}  {}", t.id, status, priority, t.title);
                }
            }
        }
        Commands::Start { id } => {
            if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
                t.status = Status::InProgress;
                println!("✓ 开始: {}", t.title);
            } else {
                println!("找不到任务 #{}", id);
            }
        }
        Commands::Done { id } => {
            if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
                t.status = Status::Done;
                println!("✓ 完成: {}", t.title);
            } else {
                println!("找不到任务 #{}", id);
            }
        }
        Commands::Remove { id } => {
            let len = tasks.len();
            tasks.retain(|t| t.id != id);
            if tasks.len() < len {
                println!("✓ 已删除任务 #{}", id);
            } else {
                println!("找不到任务 #{}", id);
            }
        }
    }

    save(&tasks);
}
