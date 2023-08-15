use crate::{semaphore::Semaphore, sync_rand::sync_rand_range};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

// Readers would starve (writers-preference)
pub fn readers_writers_problem2() {
    const READER_NUMBER: usize = 10;
    const WRITER_NUMBER: usize = 10;

    let mut resource = Box::new(0);
    let readers_writer_mtx = Arc::new(Semaphore::new(1));
    let try_read_mtx = Arc::new(Semaphore::new(1));
    let reader_count = Arc::new(Mutex::new(0));
    let writer_count = Arc::new(Mutex::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    let mut readers = Vec::with_capacity(READER_NUMBER);
    for _ in 0..READER_NUMBER {
        readers.push(thread::spawn({
            let resource = unsafe { Box::from_raw(resource.as_mut() as *mut i32) };
            let readers_writer_mtx = Arc::clone(&readers_writer_mtx);
            let try_read_mtx = Arc::clone(&try_read_mtx);
            let reader_count = Arc::clone(&reader_count);
            let is_running = Arc::clone(&is_running);
            || {
                reader(
                    resource,
                    readers_writer_mtx,
                    try_read_mtx,
                    reader_count,
                    is_running,
                )
            }
        }));
    }

    let mut writers = Vec::with_capacity(WRITER_NUMBER);
    for _ in 0..READER_NUMBER {
        writers.push(thread::spawn({
            let resource = unsafe { Box::from_raw(resource.as_mut() as *mut i32) };
            let readers_writer_mtx = Arc::clone(&readers_writer_mtx);
            let try_read_mtx = Arc::clone(&try_read_mtx);
            let writer_count = Arc::clone(&writer_count);
            let is_running = Arc::clone(&is_running);
            || {
                writer(
                    resource,
                    readers_writer_mtx,
                    try_read_mtx,
                    writer_count,
                    is_running,
                )
            }
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

fn reader(
    resource: Box<i32>,
    readers_writer_mtx: Arc<Semaphore>,
    try_read_mtx: Arc<Semaphore>,
    reader_count: Arc<Mutex<i32>>,
    is_running: Arc<AtomicBool>,
) {
    let resource = Box::leak(resource) as &i32;

    while is_running.load(Ordering::Relaxed) {
        let curr_reader_count; // approximately

        {
            try_read_mtx.acquire();
            let mut reader_count = reader_count.lock().unwrap();
            if *reader_count == 0 {
                readers_writer_mtx.acquire();
            }
            *reader_count += 1;
            try_read_mtx.release();

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
        thread::sleep(Duration::from_millis(0)); // readers starvation
    }
}

fn writer(
    resource: Box<i32>,
    readers_writer_mtx: Arc<Semaphore>,
    try_read_mtx: Arc<Semaphore>,
    writer_count: Arc<Mutex<i32>>,
    is_running: Arc<AtomicBool>,
) {
    let resource = Box::leak(resource);

    while is_running.load(Ordering::Relaxed) {
        let curr_writer_count; // approximately

        {
            let mut writer_count = writer_count.lock().unwrap();
            if *writer_count == 0 {
                try_read_mtx.acquire();
            }
            *writer_count += 1;

            curr_writer_count = *writer_count;
        }
        readers_writer_mtx.acquire();

        *resource += 1;
        println!(
            "Writing {} with {} writer(s) waiting",
            resource,
            curr_writer_count - 1
        );

        readers_writer_mtx.release();
        {
            let mut writer_count = writer_count.lock().unwrap();
            *writer_count -= 1;
            if *writer_count == 0 {
                try_read_mtx.release();
            }
        }

        //thread::sleep(Duration::from_millis(sync_rand_range(0, 200))); // okay
        thread::sleep(Duration::from_millis(0)); // readers starvation
    }
}
