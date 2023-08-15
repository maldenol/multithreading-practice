use crate::{semaphore::Semaphore, sync_rand::sync_rand_range};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[derive(PartialEq)]
enum PhilosopherState {
    Thinking,
    Hungry,
    Eating,
}

pub fn dining_philosophers_problem() {
    const PHILOSOPHER_NUMBER: usize = 5;

    let states = Arc::new(Mutex::new(Vec::with_capacity(PHILOSOPHER_NUMBER)));
    let mut both_forks_available = Vec::with_capacity(PHILOSOPHER_NUMBER);
    let is_running = Arc::new(AtomicBool::new(true));

    for index in 0..PHILOSOPHER_NUMBER {
        states.lock().unwrap().push(PhilosopherState::Thinking);
        both_forks_available.push(Semaphore::new(0));
    }

    let mut philosophers = Vec::with_capacity(PHILOSOPHER_NUMBER);
    for index in 0..PHILOSOPHER_NUMBER {
        philosophers.push(thread::spawn({
            let states = Arc::clone(&states);
            let both_forks_available =
                unsafe { Box::from_raw(both_forks_available.as_mut() as *mut Vec<Semaphore>) };
            let is_running = Arc::clone(&is_running);
            move || {
                philosopher(
                    index,
                    PHILOSOPHER_NUMBER,
                    states,
                    both_forks_available,
                    is_running,
                )
            }
        }));
    }

    thread::sleep(Duration::from_secs(3));

    is_running.store(false, Ordering::Relaxed);

    for phil in philosophers {
        let _ = phil.join();
    }
}

fn philosopher(
    index: usize,
    total_number: usize,
    states: Arc<Mutex<Vec<PhilosopherState>>>,
    mut both_forks_available: Box<Vec<Semaphore>>,
    is_running: Arc<AtomicBool>,
) {
    let states = states.as_ref();
    let mut both_forks_available = Box::leak(both_forks_available);

    while is_running.load(Ordering::Relaxed) {
        think(index);
        take_forks(index, total_number, states, both_forks_available);
        eat(index);
        put_forks(index, total_number, states, both_forks_available);
    }
}

fn think(index: usize) {
    let dur = Duration::from_millis(sync_rand_range(100, 500));
    println!(
        "Philosopher {} is thinking for {} ms",
        index,
        dur.as_millis()
    );
    thread::sleep(dur);
}

fn eat(index: usize) {
    let dur = Duration::from_millis(sync_rand_range(100, 500));
    println!("Philosopher {} is eating for {} ms", index, dur.as_millis());
    thread::sleep(dur);
}

fn take_forks(
    index: usize,
    total_number: usize,
    states: &Mutex<Vec<PhilosopherState>>,
    both_forks_available: &mut Vec<Semaphore>,
) {
    {
        let mut states = states.lock().unwrap();
        states[index] = PhilosopherState::Hungry;
        println!("Philosopher {} is now hungry", index);
        update_state(index, total_number, &mut states, both_forks_available);
    }
    both_forks_available[index].acquire();
}

fn put_forks(
    index: usize,
    total_number: usize,
    states: &Mutex<Vec<PhilosopherState>>,
    both_forks_available: &mut Vec<Semaphore>,
) {
    let mut states = states.lock().unwrap();
    states[index] = PhilosopherState::Thinking;
    update_state(
        left(index, total_number),
        total_number,
        &mut states,
        both_forks_available,
    );
    update_state(
        right(index, total_number),
        total_number,
        &mut states,
        both_forks_available,
    );
}

fn update_state(
    index: usize,
    total_number: usize,
    states: &mut Vec<PhilosopherState>,
    both_forks_available: &mut Vec<Semaphore>,
) {
    if states[index] == PhilosopherState::Hungry
        && states[left(index, total_number)] != PhilosopherState::Eating
        && states[right(index, total_number)] != PhilosopherState::Eating
    {
        states[index] = PhilosopherState::Eating;
        both_forks_available[index].release();
    }
}

fn left(index: usize, total_number: usize) -> usize {
    (index + total_number - 1) % total_number
}

fn right(index: usize, total_number: usize) -> usize {
    (index + 1) % total_number
}
