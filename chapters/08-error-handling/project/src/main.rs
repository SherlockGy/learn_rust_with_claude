use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    Pending,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Priority {
    Low,
    Medium,
    High,
}

struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
    due_date: Option<String>,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            status: Status::Pending,
            priority: Priority::Medium,
            due_date: None,
        }
    }

    fn to_line(&self) -> String {
        let status = match self.status {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        };
        let priority = match self.priority {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        let due = self.due_date.as_deref().unwrap_or("");
        format!("{}|{}|{}|{}|{}", self.id, status, priority, self.title, due)
    }

    fn from_line(line: &str) -> Option<Task> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 4 {
            return None;
        }

        let id: u32 = parts[0].parse().ok()?;
        let status = match parts[1] {
            "进行中" => Status::InProgress,
            "完成" => Status::Done,
            _ => Status::Pending,
        };
        let priority = match parts[2] {
            "低" => Priority::Low,
            "高" => Priority::High,
            _ => Priority::Medium,
        };
        let title = parts[3].to_string();
        let due_date = parts.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string());

        Some(Task {
            id,
            title,
            status,
            priority,
            due_date,
        })
    }

    fn display(&self) {
        let status = match self.status {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        };
        let priority = match self.priority {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        let due = self.due_date.as_deref().unwrap_or("-");
        println!(
            "{:>3}  {:>6}  {:>4}  {:>10}  {}",
            self.id, status, priority, due, self.title
        );
    }
}

const DATA_FILE: &str = "tasks.txt";

fn save_tasks(tasks: &[Task], path: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    for task in tasks {
        writeln!(file, "{}", task.to_line())?;
    }
    Ok(())
}

fn load_tasks(path: &str) -> io::Result<(Vec<Task>, u32)> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok((Vec::new(), 1));
        }
        Err(e) => return Err(e),
    };

    let reader = BufReader::new(file);
    let mut tasks = Vec::new();
    let mut max_id = 0u32;

    for line in reader.lines() {
        let line = line?;
        if let Some(task) = Task::from_line(&line) {
            if task.id > max_id {
                max_id = task.id;
            }
            tasks.push(task);
        }
    }

    Ok((tasks, max_id + 1))
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }
    println!("{:>3}  {:>6}  {:>4}  {:>10}  任务", "ID", "状态", "优先级", "截止");
    println!("{}", "-".repeat(50));
    for task in tasks {
        task.display();
    }
}

fn find_task_mut(tasks: &mut [Task], id: u32) -> Option<&mut Task> {
    tasks.iter_mut().find(|t| t.id == id)
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器 (v0.3)");
    println!();
    println!("用法:");
    println!("  task add <任务>      添加任务");
    println!("  task list            列出任务");
    println!("  task start <ID>      开始任务");
    println!("  task done <ID>       完成任务");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let (mut tasks, mut next_id) = load_tasks(DATA_FILE).unwrap_or_else(|e| {
        eprintln!("警告: 无法加载任务: {}", e);
        (Vec::new(), 1)
    });

    if args.is_empty() {
        print_help();
        return;
    }

    let command = &args[0];
    match command.as_str() {
        "add" => {
            if args.len() < 2 {
                println!("用法: task add <任务>");
                return;
            }
            let title = args[1..].join(" ");
            let task = Task::new(next_id, title.clone());
            println!("✓ 任务已添加 (ID: {}): {}", task.id, title);
            tasks.push(task);
            next_id += 1;
        }
        "list" => {
            list_tasks(&tasks);
        }
        "start" => {
            if args.len() < 2 {
                println!("用法: task start <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.status = Status::InProgress;
                    println!("✓ 任务 #{} 已开始", id);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.status = Status::Done;
                    println!("✓ 任务 #{} 已完成", id);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }

    if let Err(e) = save_tasks(&tasks, DATA_FILE) {
        eprintln!("保存失败: {}", e);
    }
}
