use std::env;

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

impl Status {
    fn as_str(&self) -> &str {
        match self {
            Status::Pending => "待办",
            Status::InProgress => "进行中",
            Status::Done => "完成",
        }
    }
}

impl Priority {
    fn as_str(&self) -> &str {
        match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        }
    }
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

    fn start(&mut self) {
        self.status = Status::InProgress;
    }

    fn complete(&mut self) {
        self.status = Status::Done;
    }

    fn display(&self) {
        let due = match &self.due_date {
            Some(date) => date.as_str(),
            None => "-",
        };

        println!(
            "{:>3}  {:>4}  {:>6}  {:>10}  {}",
            self.id,
            self.priority.as_str(),
            self.status.as_str(),
            due,
            self.title
        );
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!(
        "{:>3}  {:>4}  {:>6}  {:>10}  任务",
        "ID", "优先级", "状态", "截止"
    );
    println!("{}", "-".repeat(60));
    for task in tasks {
        task.display();
    }
}

fn find_task_mut(tasks: &mut [Task], id: u32) -> Option<&mut Task> {
    tasks.iter_mut().find(|t| t.id == id)
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务内容>  添加任务");
    println!("  task list            列出任务");
    println!("  task start <ID>      开始任务");
    println!("  task done <ID>       完成任务");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 预添加演示任务
    let mut t1 = Task::new(next_id, String::from("安装 Rust"));
    t1.complete();
    tasks.push(t1);
    next_id += 1;

    tasks.push(Task::new(next_id, String::from("学习枚举")));
    next_id += 1;

    let mut t3 = Task::new(next_id, String::from("写代码"));
    t3.start();
    tasks.push(t3);
    next_id += 1;

    if args.is_empty() {
        print_help();
        return;
    }

    let command = &args[0];
    match command.as_str() {
        "add" => {
            if args.len() < 2 {
                println!("用法: task add <任务内容>");
                return;
            }
            let title = args[1..].join(" ");
            let task = Task::new(next_id, title.clone());
            println!("✓ 任务已添加 (ID: {}): {}", task.id, title);
            tasks.push(task);
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
                    task.start();
                    println!("✓ 任务 #{} 已开始: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            } else {
                println!("无效的 ID: {}", args[1]);
            }
        }
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }
            if let Ok(id) = args[1].parse::<u32>() {
                if let Some(task) = find_task_mut(&mut tasks, id) {
                    task.complete();
                    println!("✓ 任务 #{} 已完成: {}", id, task.title);
                } else {
                    println!("找不到任务 #{}", id);
                }
            } else {
                println!("无效的 ID: {}", args[1]);
            }
        }
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }
}
