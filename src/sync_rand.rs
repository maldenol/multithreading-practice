use std::sync::atomic::{AtomicU64, Ordering};

static mut SEED: AtomicU64 = AtomicU64::new(0);

pub fn sync_rand() -> u64 {
    let mut next = unsafe { SEED.load(Ordering::Relaxed) };
    let mut result;

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result = next.wrapping_div(65536).wrapping_rem(2048);

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result <<= 10;
    result ^= next.wrapping_div(65536).wrapping_rem(1024);

    next = next.wrapping_mul(1103515245);
    next = next.wrapping_add(12345);
    result <<= 10;
    result ^= next.wrapping_div(65536).wrapping_rem(1024);

    unsafe {
        SEED.store(next, Ordering::Relaxed);
    }

    result
}

pub fn sync_rand_range(min: u64, max: u64) -> u64 {
    map(sync_rand(), 0, i32::MAX as u64, min, max)
}

fn map(val: u64, in_min: u64, in_max: u64, out_min: u64, out_max: u64) -> u64 {
    let val = val as f64;
    let in_min = in_min as f64;
    let in_max = in_max as f64;
    let out_min = out_min as f64;
    let out_max = out_max as f64;

    (out_min + (((val - in_min) / (in_max - in_min)) * (out_max - out_min))) as u64
}
