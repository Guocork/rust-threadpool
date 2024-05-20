use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::SystemTime,
};

type Job = Box<dyn FnOnce() + Send>;


struct Threadpool {
    workers: Vec<Worker>,  // 多个工作线程集合
    sender: mpsc::Sender<Job> // 一个sender
}

/// 将 thread::JoinHandle<()> 包装在 Option 中有以下几个设计上的优点：
///   延迟初始化：允许在稍后阶段初始化线程。
///   线程的可终止性：安全地表示线程已经终止或尚未启动。
///   表示线程的生命周期：明确线程的运行状态。
///   简化线程管理：方便在销毁线程池时终止和回收线程。
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Threadpool {
    fn new(size: usize) -> Self {
        assert!(size > 0); 
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size); // 用于存储所有的工作线程
        for id in 1..=size {
            let receiver = receiver.clone();
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Threadpool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static // 因为不知道什么时候这个job会被执行 所以 这里使用了静态生命周期
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Drop for Threadpool {
    fn drop(&mut self) {

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread  = thread::spawn(move || loop {
            // 从channel中拉取job执行
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job(); // 这里的job是一个闭包 这里调用这个闭包
        });

        Worker { id, thread: Some(thread) }
    }
}

fn main() {
    println!("Hello, world!");
}
