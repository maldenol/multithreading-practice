use crate::semaphore::Semaphore;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

static mut PRODUCED: AtomicU32 = AtomicU32::new(0);
static mut CONSUMED: AtomicU32 = AtomicU32::new(0);

pub fn producer_consumer_problem() {
    const MAX_QUEUE_LENGTH: usize = 10;
    const PRODUCER_NUMBER: usize = 1000;
    const CONSUMER_NUMBER: usize = 1000;

    let item_queue = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_LENGTH)));
    let queue_length = Arc::new(Semaphore::new(0));
    let empty_number = Arc::new(Semaphore::new(MAX_QUEUE_LENGTH));
    let is_running = Arc::new(AtomicBool::new(true));

    let mut producers = Vec::with_capacity(PRODUCER_NUMBER);
    for _ in 0..PRODUCER_NUMBER {
        producers.push(thread::spawn({
            let item_queue = Arc::clone(&item_queue);
            let queue_length = Arc::clone(&queue_length);
            let empty_number = Arc::clone(&empty_number);
            let is_running = Arc::clone(&is_running);
            move || producer(item_queue, queue_length, empty_number, is_running)
        }));
    }

    let mut consumers = Vec::with_capacity(CONSUMER_NUMBER);
    for _ in 0..CONSUMER_NUMBER {
        consumers.push(thread::spawn({
            let item_queue = Arc::clone(&item_queue);
            let queue_length = Arc::clone(&queue_length);
            let empty_number = Arc::clone(&empty_number);
            let is_running = Arc::clone(&is_running);
            move || consumer(item_queue, queue_length, empty_number, is_running)
        }));
    }

    thread::sleep(Duration::from_secs(1));

    is_running.store(false, Ordering::Relaxed);

    for prod in producers {
        let _ = prod.join();
    }

    for cons in consumers {
        let _ = cons.join();
    }

    println!("Produced {} items", unsafe {
        PRODUCED.load(Ordering::Relaxed)
    });
    println!("Consumed {} items", unsafe {
        CONSUMED.load(Ordering::Relaxed)
    });
}

#[derive(Debug)]
struct Item(u32);

fn producer(
    item_queue: Arc<Mutex<VecDeque<Item>>>,
    queue_length: Arc<Semaphore>,
    empty_number: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
) {
    if queue_length.get_value() == empty_number.get_value() {
        return;
    }

    while is_running.load(Ordering::Relaxed) {
        let item = produce_item();

        empty_number.acquire();
        {
            let mut item_queue = item_queue.lock().unwrap();
            item_queue.push_back(item);
        }
        queue_length.release();
    }
}

fn consumer(
    item_queue: Arc<Mutex<VecDeque<Item>>>,
    queue_length: Arc<Semaphore>,
    empty_number: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::Relaxed) || queue_length.get_value() > 0 {
        let item;

        while !queue_length.acquire_timeout(Duration::from_millis(50)) {
            if !is_running.load(Ordering::Relaxed) && queue_length.get_value() == 0 {
                return;
            }
        }
        {
            let mut item_queue = item_queue.lock().unwrap();
            item = item_queue.pop_front().unwrap();
        }
        empty_number.release();

        consume_item(item);
    }
}

fn produce_item() -> Item {
    static mut INDEX: u32 = 0;
    unsafe {
        INDEX = INDEX.wrapping_add(1);
    }
    let item = unsafe { Item(INDEX) };
    println!("Produced {:?}", item);
    unsafe { PRODUCED.store(PRODUCED.load(Ordering::Relaxed) + 1, Ordering::Relaxed) };
    item
}

fn consume_item(item: Item) {
    println!("Consumed {:?}", item);
    unsafe { CONSUMED.store(CONSUMED.load(Ordering::Relaxed) + 1, Ordering::Relaxed) };
}
