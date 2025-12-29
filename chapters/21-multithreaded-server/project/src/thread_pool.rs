// 线程池实现
// 参考 The Rust Book 第 20 章

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// 线程池
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

/// 任务类型：可发送的、一次性的闭包
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// 创建线程池
    ///
    /// # Panics
    /// 如果 size 为 0 则 panic
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "线程池大小必须大于 0");

        // 创建通道
        let (sender, receiver) = mpsc::channel();

        // 多个 Worker 共享接收端，需要 Arc + Mutex
        let receiver = Arc::new(Mutex::new(receiver));

        // 创建 Worker
        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// 提交任务到线程池
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // 发送任务，忽略可能的错误（线程池关闭时）
        if let Some(sender) = &self.sender {
            sender.send(job).ok();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 关闭发送端，这会导致所有 Worker 的 recv() 返回错误
        drop(self.sender.take());

        // 等待所有 Worker 完成
        for worker in &mut self.workers {
            println!("关闭 Worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().ok();
            }
        }
    }
}

/// 工作线程
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// 创建 Worker，开始监听任务
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // 获取锁，然后接收任务
            // recv() 会阻塞直到有任务或通道关闭
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    // 通道关闭，退出循环
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn test_thread_pool() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = ThreadPool::new(4);

        for _ in 0..8 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        // 等待任务完成
        thread::sleep(Duration::from_millis(100));

        assert_eq!(counter.load(Ordering::SeqCst), 8);
    }
}
