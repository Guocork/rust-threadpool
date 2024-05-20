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

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>
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
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        let thread  = thread::spawn(move || loop {
            // 从channel中拉取job执行
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job();
        });

        Worker { id, thread }
    }
}

fn main() {
    println!("Hello, world!");
}
