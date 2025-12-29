use std::env;
use std::fmt;
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

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
    due_date: Option<String>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let due = self.due_date.as_deref().unwrap_or("-");
        write!(
            f,
            "{:>3} │ {:^6} │ {:^4} │ {:^10} │ {}",
            self.id, self.status, self.priority, due, self.title
        )
    }
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
        let due = self.due_date.as_deref().unwrap_or("");
        format!("{}|{}|{}|{}|{}", self.id, self.status, self.priority, self.title, due)
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

        Some(Task {
            id,
            title: parts[3].to_string(),
            status,
            priority,
            due_date: parts.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        })
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

    println!("┌─────┬────────┬──────┬────────────┬────────────────────────┐");
    println!("│ ID  │  状态  │优先级│   截止     │ 任务                   │");
    println!("├─────┼────────┼──────┼────────────┼────────────────────────┤");
    for task in tasks {
        println!("│{}│", task);
    }
    println!("└─────┴────────┴──────┴────────────┴────────────────────────┘");
}

fn show_task(task: &Task) {
    println!("Task #{}", task.id);
    println!("  标题: {}", task.title);
    println!("  状态: {}", task.status);
    println!("  优先级: {}", task.priority);
    if let Some(due) = &task.due_date {
        println!("  截止: {}", due);
    }
    println!();
    println!("Debug 输出: {:?}", task);
}

fn find_task_mut(tasks: &mut [Task], id: u32) -> Option<&mut Task> {
    tasks.iter_mut().find(|t| t.id == id)
}

fn find_task(tasks: &[Task], id: u32) -> Option<&Task> {
    tasks.iter().find(|t| t.id == id)
}

fn print_help() {
    println!("task-cli v0.4 - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务>      添加任务");
    println!("  task list            列出任务");
    println!("  task show <ID>       显示任务详情");
    println!("  task start <ID>      开始任务");
    println!("  task done <ID>       完成任务");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let (mut tasks, mut next_id) = load_tasks(DATA_FILE).unwrap_or_else(|e| {
        eprintln!("警告: {}", e);
        (Vec::new(), 1)
    });

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
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
        "list" => list_tasks(&tasks),
        "show" => {
            if args.len() < 2 {
                println!("用法: task show <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                match find_task(&tasks, id) {
                    Some(task) => show_task(task),
                    None => println!("找不到任务 #{}", id),
                }
            }
        }
        "start" => {
            if let Some(id) = args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.status = Status::InProgress;
                    println!("✓ 任务 #{} 已开始", id);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }
        "done" => {
            if let Some(id) = args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.status = Status::Done;
                    println!("✓ 任务 #{} 已完成", id);
                } else {
                    println!("找不到任务 #{}", id);
                }
            }
        }
        _ => {
            println!("未知命令");
            print_help();
        }
    }

    let _ = save_tasks(&tasks, DATA_FILE);
}
