//! task-cli with Serde JSON storage

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pending,
    InProgress,
    Done,
}

impl Default for Status {
    fn default() -> Self { Status::Pending }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self { Priority::Medium }
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    #[serde(default)]
    status: Status,
    #[serde(default)]
    priority: Priority,
    #[serde(skip_serializing_if = "Option::is_none")]
    due_date: Option<String>,
}

const DATA_FILE: &str = "tasks.json";

fn load_tasks() -> Vec<Task> {
    fs::read_to_string(DATA_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_tasks(tasks: &[Task]) {
    let json = serde_json::to_string_pretty(tasks).unwrap();
    fs::write(DATA_FILE, json).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut tasks = load_tasks();

    if args.is_empty() {
        println!("task-cli v0.6 (with Serde)");
        println!("用法: task [add|list|done] ...");
        return;
    }

    match args[0].as_str() {
        "add" => {
            let next_id = tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            let title = args[1..].join(" ");
            tasks.push(Task {
                id: next_id,
                title: title.clone(),
                status: Status::Pending,
                priority: Priority::Medium,
                due_date: None,
            });
            println!("✓ 添加: {} (ID: {})", title, next_id);
        }
        "list" => {
            println!("{}", serde_json::to_string_pretty(&tasks).unwrap());
        }
        "done" => {
            if let Some(id) = args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
                    task.status = Status::Done;
                    println!("✓ 完成: {}", task.title);
                }
            }
        }
        _ => println!("未知命令"),
    }

    save_tasks(&tasks);
}
