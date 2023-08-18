#![allow(unused)]

mod aba_problem;
mod cigarette_smokers_problem;
mod dining_philosophers_problem;
mod producer_consumer_problem;
mod readers_writers_problem0;
mod readers_writers_problem1;
mod readers_writers_problem2;
mod readers_writers_problem3;
mod sleeping_barber_problem;
mod thread_pool;

mod semaphore;
mod sync_rand;

use aba_problem::aba_problem;
use cigarette_smokers_problem::cigarette_smokers_problem;
use dining_philosophers_problem::dining_philosophers_problem;
use producer_consumer_problem::producer_consumer_problem;
use readers_writers_problem0::readers_writers_problem0;
use readers_writers_problem1::readers_writers_problem1;
use readers_writers_problem2::readers_writers_problem2;
use readers_writers_problem3::readers_writers_problem3;
use sleeping_barber_problem::sleeping_barber_problem;
use thread_pool::thread_pool;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        match args[1].as_str() {
            "abap" => aba_problem(),
            "csp" => cigarette_smokers_problem(),
            "dpp" => dining_philosophers_problem(),
            "pcp" => producer_consumer_problem(),
            "rwp0" => readers_writers_problem0(),
            "rwp1" => readers_writers_problem1(),
            "rwp2" => readers_writers_problem2(),
            "rwp3" => readers_writers_problem3(),
            "sbp" => sleeping_barber_problem(),
            "tp" => thread_pool(),
            _ => print_help(),
        }
    } else {
        print_help();
    }
}

fn print_help() {
    println!("multithreading-practice");
    println!("Usage: <executable> EXAMPLE");
    println!("Examples:");
    println!("\tabap => ABA Problem");
    println!("\tcsp  => Cigarette Smokers Problem");
    println!("\tdpp  => Dining Philosophers Problem");
    println!("\tpcp  => Producer-Consumer Problem");
    println!("\trwp0 => Readers-Writers Problem (0)");
    println!("\trwp1 => Readers-Writers Problem (1)");
    println!("\trwp2 => Readers-Writers Problem (2)");
    println!("\trwp3 => Readers-Writers Problem (3)");
    println!("\tsbp  => Sleeping Barber Problem");
    println!("\ttp   => Thread Pool");
}
