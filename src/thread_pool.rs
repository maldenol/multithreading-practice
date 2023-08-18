use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use std::thread;

pub fn thread_pool() {
    const WORKER_NUMBER: usize = 10;
    const TASK_NUMBER: usize = 1000;

    let (result_sender, result_receiver) = channel();
    let result_sender = Arc::new(Mutex::new(result_sender));

    println!("Begin spawning workers");
    let mut thread_pool = ThreadPool::new(WORKER_NUMBER);
    println!("End spawning workers");

    println!("Begin adding tasks");
    for index in 0..TASK_NUMBER {
        thread_pool.add(Task::new(
            Box::new(move || index + 1),
            Arc::clone(&result_sender),
        ));
    }
    println!("End adding tasks");

    drop(result_sender);

    println!("Begin receiving results");
    for res in result_receiver {
        println!("{}", res);
    }
    println!("End receiving results");
}

type Runnable<T> = Box<dyn FnOnce() -> T + Send>;

pub struct Task<T: Send> {
    runnable: Runnable<T>,
    result_sender: Arc<Mutex<Sender<T>>>,
}

impl<T: Send> Task<T> {
    pub fn new(runnable: Runnable<T>, result_sender: Arc<Mutex<Sender<T>>>) -> Task<T> {
        Task {
            runnable,
            result_sender,
        }
    }
}

struct Worker<T: Send> {
    task_sender: Mutex<Sender<Task<T>>>,
    is_ready: Arc<AtomicBool>,
}

impl<T: Send + 'static> Worker<T> {
    fn new(ready_sender: Sender<()>) -> Worker<T> {
        let (task_sender, task_receiver) = channel::<Task<T>>();
        let is_ready = Arc::new(AtomicBool::new(true));

        let _ = thread::spawn({
            let is_ready = Arc::clone(&is_ready);
            move || {
                for task in task_receiver {
                    task.result_sender
                        .lock()
                        .unwrap()
                        .send((task.runnable)())
                        .unwrap();
                    is_ready.store(true, Ordering::Relaxed);
                    ready_sender.send(()).unwrap();
                }
            }
        });

        Worker {
            task_sender: Mutex::new(task_sender),
            is_ready,
        }
    }

    fn send(&self, task: Task<T>) {
        if let Err(e) = self.task_sender.lock().unwrap().send(task) {
            println!("{}", e);
        }
    }
}

pub struct ThreadPool<T: Send> {
    workers: Arc<Vec<Worker<T>>>,
    tasks: Arc<Mutex<VecDeque<Task<T>>>>,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new(worker_number: usize) -> ThreadPool<T> {
        let (ready_sender, ready_receiver) = channel();

        let mut workers = Vec::with_capacity(worker_number);
        for _ in 0..worker_number {
            let ready_sender = ready_sender.clone();
            workers.push(Worker::new(ready_sender));
        }
        let workers = Arc::new(workers);

        let tasks = Arc::new(Mutex::new(VecDeque::new()));

        let _ = thread::spawn({
            let workers = Arc::clone(&workers);
            let tasks = Arc::clone(&tasks);
            move || {
                for _ in ready_receiver {
                    if let Some(task) = tasks.lock().unwrap().pop_front() {
                        if let Some(worker) = ThreadPool::find_free_worker(&workers) {
                            worker.send(task);
                        }
                    }
                }
            }
        });

        ThreadPool { workers, tasks }
    }

    pub fn add(&mut self, task: Task<T>) {
        if let Some(worker) = ThreadPool::find_free_worker(&self.workers) {
            worker.send(task);
        } else {
            self.tasks.lock().unwrap().push_back(task);
        }
    }

    fn find_free_worker(workers: &Vec<Worker<T>>) -> Option<&Worker<T>> {
        workers.iter().find(|w| {
            w.is_ready
                .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        })
    }
}
