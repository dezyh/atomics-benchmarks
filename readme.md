Testing the unsafe version which is UB. This seems to work every time.
```
❯ cargo test --release
    Finished release [optimized] target(s) in 0.02s
     Running unittests (target/release/deps/playground-621359cee22db8c6)

running 1 test
test tests::unsafe_fn ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.88s

   Doc-tests playground

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Benchmarking the variants
1. Using unsafe and u64 (fastest)
2. Using AtomicU64 with Relaxed ordering (slowest)
3. Using AtomicU64 with SeqCst ordering (middle)
```
❯ cargo bench
    Finished bench [optimized] target(s) in 0.02s
     Running unittests (target/release/deps/playground-621359cee22db8c6)

running 1 test
test tests::unsafe_fn ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests (target/release/deps/comparisons-ce0b61b0d98aed46)

relaxed atomic u64      time:   [354.97 us 357.73 us 360.50 us]
                        change: [-1.7974% -0.6551% +0.3996%] (p = 0.26 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild

unsafe ub u64           time:   [55.739 us 56.077 us 56.423 us]
                        change: [+0.5772% +1.6940% +2.8310%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild

seqcst atomic u64       time:   [274.18 us 277.04 us 280.05 us]
                        change: [-4.1724% -2.7916% -1.4354%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
```
