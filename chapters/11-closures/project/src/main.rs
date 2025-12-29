//! 闭包演示：为 task-cli 添加过滤功能

#[derive(Debug, Clone, PartialEq)]
enum Status { Pending, InProgress, Done }

#[derive(Debug, Clone, PartialEq)]
enum Priority { Low, Medium, High }

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    title: String,
    status: Status,
    priority: Priority,
}

impl Task {
    fn new(id: u32, title: &str) -> Self {
        Task { id, title: title.to_string(), status: Status::Pending, priority: Priority::Medium }
    }
}

/// 使用闭包过滤任务
fn filter_tasks<F>(tasks: &[Task], predicate: F) -> Vec<&Task>
where
    F: Fn(&Task) -> bool,
{
    tasks.iter().filter(|t| predicate(t)).collect()
}

fn main() {
    let tasks = vec![
        Task { id: 1, title: "学习闭包".into(), status: Status::Pending, priority: Priority::High },
        Task { id: 2, title: "写代码".into(), status: Status::InProgress, priority: Priority::Medium },
        Task { id: 3, title: "安装 Rust".into(), status: Status::Done, priority: Priority::Low },
    ];

    println!("=== 闭包过滤演示 ===\n");

    // 闭包捕获外部变量
    let target_status = Status::Pending;
    let pending = filter_tasks(&tasks, |t| t.status == target_status);
    println!("待办任务: {:?}\n", pending.iter().map(|t| &t.title).collect::<Vec<_>>());

    // 高优先级任务
    let high_priority = filter_tasks(&tasks, |t| t.priority == Priority::High);
    println!("高优先级: {:?}\n", high_priority.iter().map(|t| &t.title).collect::<Vec<_>>());

    // 组合条件
    let urgent = filter_tasks(&tasks, |t| {
        t.priority == Priority::High && t.status == Status::Pending
    });
    println!("紧急任务: {:?}", urgent.iter().map(|t| &t.title).collect::<Vec<_>>());
}
