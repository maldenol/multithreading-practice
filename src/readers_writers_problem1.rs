use crate::{semaphore::Semaphore, sync_rand::sync_rand_range};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

// Writers would starve (readers-preference)
pub fn readers_writers_problem1() {
    const READER_NUMBER: usize = 10;
    const WRITER_NUMBER: usize = 10;

    let mut resource = Box::new(0);
    let readers_writer_mtx = Arc::new(Semaphore::new(1));
    let reader_count = Arc::new(Mutex::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    println!("Spawning readers");
    let mut readers = Vec::with_capacity(READER_NUMBER);
    for _ in 0..READER_NUMBER {
        readers.push(thread::spawn({
            let resource = unsafe { Box::from_raw(resource.as_mut() as *mut i32) };
            let readers_writer_mtx = Arc::clone(&readers_writer_mtx);
            let reader_count = Arc::clone(&reader_count);
            let is_running = Arc::clone(&is_running);
            move || reader(resource, readers_writer_mtx, reader_count, is_running)
        }));
    }
    println!("Spawning writers");
    let mut writers = Vec::with_capacity(WRITER_NUMBER);
    for _ in 0..READER_NUMBER {
        writers.push(thread::spawn({
            let resource = unsafe { Box::from_raw(resource.as_mut() as *mut i32) };
            let readers_writer_mtx = Arc::clone(&readers_writer_mtx);
            let is_running = Arc::clone(&is_running);
            move || writer(resource, readers_writer_mtx, is_running)
        }));
    }

    thread::sleep(Duration::from_millis(3000));
    println!("Finishing threads");
    is_running.store(false, Ordering::Relaxed);

    println!("Joining readers");
    for read in readers {
        let _ = read.join();
    }
    println!("Joining writers");
    for writ in writers {
        let _ = writ.join();
    }
}

fn reader(
    resource: Box<i32>,
    readers_writer_mtx: Arc<Semaphore>,
    reader_count: Arc<Mutex<i32>>,
    is_running: Arc<AtomicBool>,
) {
    let resource = Box::leak(resource) as &i32;

    while is_running.load(Ordering::Relaxed) {
        let curr_reader_count; // approximately

        {
            let mut reader_count = reader_count.lock().unwrap();
            if *reader_count == 0 {
                readers_writer_mtx.acquire();
            }
            *reader_count += 1;

            curr_reader_count = *reader_count;
        }

        println!(
            "Reading {} with {} reader(s) at a time",
            resource, curr_reader_count
        );

        {
            let mut reader_count = reader_count.lock().unwrap();
            *reader_count -= 1;
            if *reader_count == 0 {
                readers_writer_mtx.release();
            }
        }

        //thread::sleep(Duration::from_millis(sync_rand_range(0, 200))); // okay
        thread::sleep(Duration::from_millis(0)); // writers starvation
    }
}

fn writer(mut resource: Box<i32>, readers_writer_mtx: Arc<Semaphore>, is_running: Arc<AtomicBool>) {
    let resource = Box::leak(resource);

    while is_running.load(Ordering::Relaxed) {
        readers_writer_mtx.acquire();

        *resource += 1;
        println!("Writing {}", resource);

        readers_writer_mtx.release();

        //thread::sleep(Duration::from_millis(sync_rand_range(0, 200))); // okay
        thread::sleep(Duration::from_millis(0)); // writers starvation
    }
}
