use crate::sync_rand::sync_rand_range;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

// Readers will wait for another reader to finish reading which is unnecessary
pub fn readers_writers_problem0() {
    const READER_NUMBER: usize = 10;
    const WRITER_NUMBER: usize = 10;

    let resource = Arc::new(Mutex::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    let mut readers = Vec::with_capacity(READER_NUMBER);
    for _ in 0..READER_NUMBER {
        readers.push(thread::spawn({
            let resource = Arc::clone(&resource);
            let is_running = Arc::clone(&is_running);
            || reader(resource, is_running)
        }));
    }

    let mut writers = Vec::with_capacity(WRITER_NUMBER);
    for _ in 0..READER_NUMBER {
        writers.push(thread::spawn({
            let resource = Arc::clone(&resource);
            let is_running = Arc::clone(&is_running);
            || writer(resource, is_running)
        }));
    }

    thread::sleep(Duration::from_millis(3000));

    is_running.store(false, Ordering::Relaxed);

    for read in readers {
        let _ = read.join();
    }

    for writ in writers {
        let _ = writ.join();
    }
}

fn reader(resource: Arc<Mutex<i32>>, is_running: Arc<AtomicBool>) {
    while is_running.load(Ordering::Relaxed) {
        {
            let resource = resource.lock().unwrap();
            println!("Reading {} with 1 reader at a time", resource);
        }

        thread::sleep(Duration::from_millis(sync_rand_range(0, 200)));
    }
}

fn writer(resource: Arc<Mutex<i32>>, is_running: Arc<AtomicBool>) {
    while is_running.load(Ordering::Relaxed) {
        {
            let mut resource = resource.lock().unwrap();
            *resource += 1;
            println!("Writing {}", resource);
        }

        thread::sleep(Duration::from_millis(sync_rand_range(0, 200)));
    }
}
