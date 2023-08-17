use crate::sync_rand::{sync_rand, sync_rand_range};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

pub fn aba_problem() {
    let value = Arc::new(AtomicU8::new(0));
    let dirty = Arc::new(AtomicBool::new(false));
    let is_running = Arc::new(AtomicBool::new(true));

    println!("Spawning a writer");
    let writer = thread::spawn({
        let value = Arc::clone(&value);
        let dirty = Arc::clone(&dirty);
        let is_running = Arc::clone(&is_running);
        move || writer(value, dirty, is_running)
    });
    println!("Spawning a reader");
    let reader = thread::spawn({
        let value = Arc::clone(&value);
        let dirty = Arc::clone(&dirty);
        let is_running = Arc::clone(&is_running);
        move || reader(value, dirty, is_running)
    });

    thread::sleep(Duration::from_millis(3000));
    println!("Finishing threads");
    is_running.store(false, Ordering::Relaxed);

    println!("Joining the writer");
    let _ = writer.join();
    println!("Joining the reader");
    let _ = reader.join();
}

fn writer(value: Arc<AtomicU8>, dirty: Arc<AtomicBool>, is_running: Arc<AtomicBool>) {
    while is_running.load(Ordering::Relaxed) {
        let new_value = (sync_rand() % 2) as u8;
        println!("Writing {}", new_value);
        value.store(new_value, Ordering::SeqCst);
        dirty.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(sync_rand_range(200, 500)));
    }
}

fn reader(value: Arc<AtomicU8>, dirty: Arc<AtomicBool>, is_running: Arc<AtomicBool>) {
    let mut prev_value = 0;
    while is_running.load(Ordering::Relaxed) {
        let read_value = value.load(Ordering::SeqCst);
        let read_dirty = dirty.load(Ordering::SeqCst);
        let verdict=
        // Bad test
        // if prev_value != read_value {
        //     prev_value = read_value;
        //     "has been rewritten"
        // Good test
        if read_dirty {
            dirty.store(false, Ordering::SeqCst);
            "has been rewritten"
        } else {
            "has not been rewritten"
        };
        println!("Reading {} which {}", read_value, verdict);
        thread::sleep(Duration::from_millis(sync_rand_range(100, 200)));
    }
}
