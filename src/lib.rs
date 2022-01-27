#![allow(dead_code)]
#![allow(unused_must_use)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

const A: u64 = 1;
const B: u64 = 2;

const A2: u128 = 1;
const B2: u128 = 2;
const INIT: AtomicU64 = AtomicU64::new(0);

// We want to allow multiple writers and multiple readers reading from and writing to the same
// array. We do not care about missing writes, writing in the wrong order or reading in the wrong
// order. We only care that the read value is either one of two values which could have been set
// (no partial writes of bytes) and that the program doesn't crash / leak memory?). This function
// achieves this with no overhead by using unsafe to create multiple mutable references to the same
// array.
pub fn unsafe_ub(target: u64) -> bool {
    unsafe {
        static mut ARR: [u128; 64] = [0; 64];
        let p0 = &mut ARR;

        // Spawn writer thread #1
        let p1 = &mut *(p0 as *mut [u128; 64]);
        let (tx1, rx1) = std::sync::mpsc::sync_channel::<bool>(1);
        thread::spawn(move || loop {
            // Check if the reader has notified that we've reached the target and should stop
            match rx1.try_recv() {
                Ok(_) => return,
                Err(_) => {
                    for i in 0..64 {
                        p1[i] = A2;
                    }
                }
            }
        });

        // Spawn writer thread #2
        let p2 = &mut *(p0 as *mut [u128; 64]);
        let (tx2, rx2) = std::sync::mpsc::sync_channel::<bool>(1);
        thread::spawn(move || loop {
            // Check if the reader has notified that we've reached the target and should stop
            match rx2.try_recv() {
                Ok(_) => return,
                Err(_) => {
                    for i in 0..64 {
                        p2[i] = B2;
                    }
                }
            }
        });

        // Spawn a reader thread
        let (tx, rx) = std::sync::mpsc::sync_channel::<bool>(1);
        let p3 = &mut *(p0 as *mut [u128; 64]);
        thread::spawn(move || {
            // Keep track of the number of good and bad read values
            // A good value is any of A2 or B2. Anything else inbetween signifies that only some of
            // the bytes were written and the value is between two possible valid states.
            let mut good: u64 = 0;
            let mut bad: u64 = 0;
            loop {
                for i in 0..64 {
                    // Read the value and check if it matches any of A2 or B2
                    let x = p3[i];
                    let is_a = x == A2;
                    let is_b = x == B2;

                    if is_a || is_b {
                        good += 1;
                    } else {
                        bad += 1;
                    }
                }

                // If we find any bad values abort
                if bad > 0 {
                    tx1.send(false);
                    tx2.send(false);
                    tx.send(false);
                    return;
                }

                // When we reach the target we're done
                if good >= target {
                    tx1.send(true);
                    tx2.send(true);
                    tx.send(true);
                    return;
                }
            }
        });

        // Wait for an outcome
        if let Ok(outcome) = rx.recv() {
            return outcome;
        }

        // If we made it here there was an issue somewhere above
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
        assert_eq!(true, unsafe_ub(100000000000000000));
    }
}
