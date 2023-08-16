use crate::{
    semaphore::Semaphore,
    sync_rand::{sync_rand, sync_rand_range},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

pub fn cigarette_smokers_problem() {
    const RES1_NAME: &str = "tobacco";
    const RES2_NAME: &str = "paper";
    const RES3_NAME: &str = "matches";

    let agent_sem = Arc::new(Semaphore::new(1));
    let res1_sem = Arc::new(Semaphore::new(0));
    let res2_sem = Arc::new(Semaphore::new(0));
    let res3_sem = Arc::new(Semaphore::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    println!("Spawning smokers");
    let mut smokers = Vec::new();
    smokers.push(thread::spawn({
        let agent_sem = Arc::clone(&agent_sem);
        let res_sems = (Arc::clone(&res2_sem), Arc::clone(&res3_sem));
        let res_names = (RES1_NAME, RES2_NAME, RES3_NAME);
        let is_running = Arc::clone(&is_running);
        move || smoker(agent_sem, res_sems, res_names, is_running)
    }));
    smokers.push(thread::spawn({
        let agent_sem = Arc::clone(&agent_sem);
        let res_sems = (Arc::clone(&res3_sem), Arc::clone(&res1_sem));
        let res_names = (RES2_NAME, RES3_NAME, RES1_NAME);
        let is_running = Arc::clone(&is_running);
        move || smoker(agent_sem, res_sems, res_names, is_running)
    }));
    smokers.push(thread::spawn({
        let agent_sem = Arc::clone(&agent_sem);
        let res_sems = (Arc::clone(&res1_sem), Arc::clone(&res2_sem));
        let res_names = (RES3_NAME, RES1_NAME, RES2_NAME);
        let is_running = Arc::clone(&is_running);
        move || smoker(agent_sem, res_sems, res_names, is_running)
    }));
    println!("Spawning an agent");
    let agent = thread::spawn({
        let agent_sem = Arc::clone(&agent_sem);
        let res_sems = (res1_sem, res2_sem, res3_sem);
        let res_names = (RES1_NAME, RES2_NAME, RES3_NAME);
        let is_running = Arc::clone(&is_running);
        move || agent(agent_sem, res_sems, res_names, is_running)
    });

    thread::sleep(Duration::from_millis(3000));
    println!("Finishing threads");
    is_running.store(false, Ordering::Relaxed);

    println!("Joining the agent");
    let _ = agent.join();
    println!("Joining smokers");
    for smok in smokers {
        let _ = smok.join();
    }
}

fn smoker(
    agent_sem: Arc<Semaphore>,
    res_sems: (Arc<Semaphore>, Arc<Semaphore>),
    res_names: (&str, &str, &str),
    is_running: Arc<AtomicBool>,
) {
    let (res1_sem, res2_sem) = res_sems;
    let (res_name, res1_name, res2_name) = res_names;

    while is_running.load(Ordering::Relaxed) {
        // Does not work
        // res1_sem.acquire();
        // //println!("Smoker with {} has taken {} and is waiting for {}", res_name, res1_name, res2_name);
        // res2_sem.acquire();
        // //println!("Smoker with {} has taken {}", res_name, res2_name);
        // agent_sem.release();
        // smoke(res_name);

        // Does work
        while !res1_sem.acquire_timeout(Duration::from_millis(1000)) {
            if !is_running.load(Ordering::Relaxed) {
                return;
            }
        }
        // println!("Smoker with {} has taken {} and is waiting for {}", res_name, res1_name, res2_name);
        if res2_sem.acquire_timeout(Duration::from_millis(0)) {
            // println!("Smoker with {} has taken {}", res_name, res2_name);
            agent_sem.release();
            smoke(res_name);
        } else {
            // println!("Smoker with {} cannot take {} and is returning {}", res_name, res2_name, res1_name);
            res1_sem.release();
        }
    }
}

// Do not modify
fn agent(
    agent_sem: Arc<Semaphore>,
    res_sems: (Arc<Semaphore>, Arc<Semaphore>, Arc<Semaphore>),
    res_names: (&str, &str, &str),
    is_running: Arc<AtomicBool>,
) {
    let (res1_sem, res2_sem, res3_sem) = res_sems;
    let (res1_name, res2_name, res3_name) = res_names;

    while is_running.load(Ordering::Relaxed) {
        while !agent_sem.acquire_timeout(Duration::from_millis(1000)) {
            if !is_running.load(Ordering::Relaxed) {
                return;
            }
        }

        let choice = sync_rand();
        match choice % 3 {
            0 => {
                println!("Providing {} and {}", res1_name, res2_name);
                res1_sem.release();
                res2_sem.release();
            }
            1 => {
                println!("Providing {} and {}", res2_name, res3_name);
                res2_sem.release();
                res3_sem.release();
            }
            2 => {
                println!("Providing {} and {}", res3_name, res1_name);
                res3_sem.release();
                res1_sem.release();
            }
            _ => (),
        };
    }
}

fn smoke(res_name: &str) {
    let dur = Duration::from_millis(sync_rand_range(100, 500));
    println!(
        "Smoker with {} is smoking for {} ms",
        res_name,
        dur.as_millis()
    );
    thread::sleep(dur);
}
