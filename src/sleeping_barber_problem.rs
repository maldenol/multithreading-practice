use crate::{semaphore::Semaphore, sync_rand::sync_rand_range};
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

pub fn sleeping_barber_problem() {
    const CUSTOMER_NUMBER: usize = 10;
    const CHAIR_NUMBER: usize = 3;

    let customer_ready = Arc::new(Semaphore::new(0));
    let customer_queue = Arc::new(Mutex::new(VecDeque::with_capacity(CHAIR_NUMBER)));
    let barber_ready = Arc::new(Semaphore::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    println!("Spawning a barber");
    let barber = thread::spawn({
        let customer_ready = Arc::clone(&customer_ready);
        let customer_queue = Arc::clone(&customer_queue);
        let barber_ready = Arc::clone(&barber_ready);
        let is_running = Arc::clone(&is_running);
        move || barber(customer_ready, customer_queue, barber_ready, is_running)
    });
    println!("Spawning customers");
    let mut customers = Vec::with_capacity(CUSTOMER_NUMBER);
    for index in 0..CUSTOMER_NUMBER {
        customers.push(thread::spawn({
            let customer_ready = Arc::clone(&customer_ready);
            let customer_queue = Arc::clone(&customer_queue);
            let barber_ready = Arc::clone(&barber_ready);
            let is_running = Arc::clone(&is_running);
            move || {
                customer(
                    index,
                    customer_ready,
                    customer_queue,
                    barber_ready,
                    is_running,
                )
            }
        }));
    }

    thread::sleep(Duration::from_millis(3000));
    println!("Finishing threads");
    is_running.store(false, Ordering::Relaxed);

    println!("Joining customers");
    for cust in customers {
        let _ = cust.join();
    }
    println!("Joining the barber");
    let _ = barber.join();
}

fn customer(
    index: usize,
    customer_ready: Arc<Semaphore>,
    customer_queue: Arc<Mutex<VecDeque<usize>>>,
    barber_ready: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::Relaxed) {
        let sit;

        {
            let mut customer_queue = customer_queue.lock().unwrap();
            if customer_queue.capacity() > customer_queue.len() {
                customer_queue.push_back(index);

                sit = true;
            } else {
                sit = false;
            }
        }

        if sit {
            println!("Customer {} is sitting down", index);

            customer_ready.release();

            while !barber_ready.acquire_timeout(Duration::from_millis(1000)) {
                if !is_running.load(Ordering::Relaxed) {
                    return;
                }
            }
        } else {
            //println!("Customer {} is walking away", index);
        }

        let dur = Duration::from_millis(sync_rand_range(1, 10));
        //println!("Customer {} is walking for {} ms", index, dur.as_millis());
        thread::sleep(dur);
    }
}

fn barber(
    customer_ready: Arc<Semaphore>,
    customer_queue: Arc<Mutex<VecDeque<usize>>>,
    barber_ready: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
) {
    while is_running.load(Ordering::Relaxed) || !customer_queue.lock().unwrap().is_empty() {
        while !customer_ready.acquire_timeout(Duration::from_millis(1000)) {
            if !is_running.load(Ordering::Relaxed) && customer_queue.lock().unwrap().is_empty() {
                return;
            }
        }

        let index;

        {
            let mut customer_queue = customer_queue.lock().unwrap();
            index = customer_queue.pop_front().unwrap();
        }

        let dur = Duration::from_millis(sync_rand_range(1, 10));
        println!(
            "Barber is doing a haircut for customer {} for {} ms",
            index,
            dur.as_millis()
        );
        thread::sleep(dur);

        barber_ready.release();
    }
}
