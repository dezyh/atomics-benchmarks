#![allow(dead_code)]
#![allow(unused_must_use)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

const A: u64 = 1;
const B: u64 = 2;
const INIT: AtomicU64 = AtomicU64::new(0);

pub fn unsafe_ub(target: u64) -> bool {
    unsafe {
        static mut ARR: [u64; 64] = [0; 64];
        let p0 = &mut ARR;

        // Spawn two writer threads
        let p1 = &mut *(p0 as *mut [u64; 64]);
        let (tx1, rx1) = std::sync::mpsc::sync_channel::<bool>(1);
        thread::spawn(move || loop {
            match rx1.try_recv() {
                Ok(_) => return,
                Err(_) => {
                    for i in 0..64 {
                        p1[i] = A;
                    }
                }
            }
        });
        let p2 = &mut *(p0 as *mut [u64; 64]);
        let (tx2, rx2) = std::sync::mpsc::sync_channel::<bool>(1);
        thread::spawn(move || loop {
            match rx2.try_recv() {
                Ok(_) => return,
                Err(_) => {
                    for i in 0..64 {
                        p2[i] = B;
                    }
                }
            }
        });

        // Spawn a reader thread
        let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);
        let p3 = &mut *(p0 as *mut [u64; 64]);
        thread::spawn(move || {
            let mut good: u64 = 0;
            let mut bad: u64 = 0;
            loop {
                for i in 0..64 {
                    let x = p3[i];
                    let is_a = x == A;
                    let is_b = x == B;

                    if is_a || is_b {
                        good += 1;
                    } else {
                        bad += 1;
                    }
                }

                if bad > 0 {
                    tx1.send(false);
                    tx2.send(false);
                    tx.send(false);
                    return;
                }

                if good >= target {
                    tx1.send(true);
                    tx2.send(true);
                    tx.send(true);
                    return;
                }
            }
        });

        if let Ok(outcome) = rx.recv() {
            return outcome;
        }

        return false;
    }
}

pub fn atomics_seqcst(target: u64) -> bool {
    let arr: Arc<[AtomicU64; 64]> = Arc::new([INIT; 64]);

    // Spawn two writer threads
    let arr1 = Arc::clone(&arr);
    let (tx1, rx1) = std::sync::mpsc::sync_channel::<bool>(1);
    thread::spawn(move || loop {
        match rx1.try_recv() {
            Ok(_) => return,
            Err(_) => {
                for i in 0..64 {
                    arr1[i].store(A, Ordering::SeqCst);
                }
            }
        }
    });

    let arr2 = Arc::clone(&arr);
    let (tx2, rx2) = std::sync::mpsc::sync_channel::<bool>(1);
    thread::spawn(move || loop {
        match rx2.try_recv() {
            Ok(_) => return,
            Err(_) => {
                for i in 0..64 {
                    arr2[i].store(B, Ordering::SeqCst);
                }
            }
        }
    });

    let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);
    let arr3 = Arc::clone(&arr);
    thread::spawn(move || {
        let mut good: u64 = 0;
        let mut bad: u64 = 0;
        loop {
            for i in 0..64 {
                let x = arr3[i].load(Ordering::SeqCst);
                let is_a = x == A;
                let is_b = x == B;

                if is_a || is_b {
                    good += 1;
                } else {
                    bad += 1;
                }
            }

            if bad > 0 {
                tx1.send(false);
                tx2.send(false);
                tx.send(false);
                return;
            }

            if good >= target {
                tx1.send(true);
                tx2.send(true);
                tx.send(true);
                return;
            }
        }
    });

    if let Ok(outcome) = rx.recv() {
        return outcome;
    }

    false
}

pub fn atomics_relaxed(target: u64) -> bool {
    let arr: Arc<[AtomicU64; 64]> = Arc::new([INIT; 64]);

    // Spawn two writer threads
    let arr1 = Arc::clone(&arr);
    let (tx1, rx1) = std::sync::mpsc::sync_channel::<bool>(1);
    thread::spawn(move || loop {
        match rx1.try_recv() {
            Ok(_) => return,
            Err(_) => {
                for i in 0..64 {
                    arr1[i].store(A, Ordering::Relaxed);
                }
            }
        }
    });

    let arr2 = Arc::clone(&arr);
    let (tx2, rx2) = std::sync::mpsc::sync_channel::<bool>(1);
    thread::spawn(move || loop {
        match rx2.try_recv() {
            Ok(_) => return,
            Err(_) => {
                for i in 0..64 {
                    arr2[i].store(B, Ordering::Relaxed);
                }
            }
        }
    });

    let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);
    let arr3 = Arc::clone(&arr);
    thread::spawn(move || {
        let mut good: u64 = 0;
        let mut bad: u64 = 0;
        loop {
            for i in 0..64 {
                let x = arr3[i].load(Ordering::Relaxed);
                let is_a = x == A;
                let is_b = x == B;

                if is_a || is_b {
                    good += 1;
                } else {
                    bad += 1;
                }
            }

            if bad > 0 {
                tx1.send(false);
                tx2.send(false);
                tx.send(false);
                return;
            }

            if good >= target {
                tx1.send(true);
                tx2.send(true);
                tx.send(true);
                return;
            }
        }
    });

    if let Ok(outcome) = rx.recv() {
        return outcome;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_fn() {
        assert_eq!(true, unsafe_ub(100000000000));
    }
}
