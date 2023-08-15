use std::sync::{Condvar, Mutex};
use std::time::Duration;

pub struct Semaphore {
    lock: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    pub fn new(count: usize) -> Semaphore {
        Semaphore {
            lock: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut count = self.lock.lock().unwrap();
        while *count == 0 {
            count = self.cvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    pub fn acquire_timeout(&self, dur: Duration) -> bool {
        let mut count = self.lock.lock().unwrap();
        match self.cvar.wait_timeout(count, dur) {
            Ok((new_count, _)) => {
                count = new_count;
                if *count > 0 {
                    *count -= 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn release(&self) {
        *self.lock.lock().unwrap() += 1;
        self.cvar.notify_one();
    }

    pub fn get_value(&self) -> usize {
        *self.lock.lock().unwrap()
    }
}
