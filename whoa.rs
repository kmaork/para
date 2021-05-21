use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::thread;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

fn main() {
    let w = Worker::new_fifo();
    let s = w.stealer();
    let mut i = 0;
    let h = Arc::new(Mutex::new(HashSet::new()));
    let h2 = h.clone();
    thread::scope(|sc| {
        sc.spawn(move |_| loop {
            for _ in 0..65 {
                w.push(i);
                i += 1;
            }
            for _ in 0..40 {
                if let Some(p) = w.pop() {
                    if !h.lock().unwrap().insert(p) {
                        println!("yo {}", p);
                    }
                }
            }
        });
        let w2 = Worker::new_fifo();
        sc.spawn(move |_| loop {
            for _ in 0..100 {
                s.steal_batch(&w2);
            }
            while !w2.is_empty() {
                if let Some(p) = w2.pop() {
                    if !h2.lock().unwrap().insert(p) {
                        println!("ya {}", p);
                    }
                }
            }
        });
    })
    .unwrap();
}
