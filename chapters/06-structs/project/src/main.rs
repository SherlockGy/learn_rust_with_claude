use std::env;

struct Task {
    id: u32,
    title: String,
    done: bool,
}

impl Task {
    fn new(id: u32, title: String) -> Task {
        Task {
            id,
            title,
            done: false,
        }
    }

    fn mark_done(&mut self) {
        self.done = true;
    }

    fn display(&self) {
        let status = if self.done { "✓" } else { "○" };
        println!("{:>3} [{}] {}", self.id, status, self.title);
    }
}

fn list_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("没有任务");
        return;
    }

    println!("{:>3} 状态 任务", "ID");
    println!("{}", "-".repeat(40));
    for task in tasks {
        task.display();
    }
}

fn print_help() {
    println!("task-cli - 命令行待办事项管理器");
    println!();
    println!("用法:");
    println!("  task add <任务内容>  添加任务");
    println!("  task list            列出任务");
    println!("  task done <ID>       标记完成");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut tasks: Vec<Task> = Vec::new();
    let mut next_id: u32 = 1;

    // 为了演示，预添加一些任务
    tasks.push(Task::new(next_id, String::from("安装 Rust")));
    next_id += 1;
    tasks.push(Task::new(next_id, String::from("学习所有权")));
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
        "done" => {
            if args.len() < 2 {
                println!("用法: task done <ID>");
                return;
            }

            match args[1].parse::<u32>() {
                Ok(id) => {
                    let mut found = false;
                    for task in &mut tasks {
                        if task.id == id {
                            task.mark_done();
                            println!("✓ 任务 #{} 已完成: {}", id, task.title);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        println!("找不到任务 #{}", id);
                    }
                }
                Err(_) => {
                    println!("无效的 ID: {}", args[1]);
                }
            }
        }
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }
}
