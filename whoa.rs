use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::thread;
use std::time::Duration;

fn main() {
    let w = Worker::new_lifo();
    let s = w.stealer();
    thread::scope(|sc| {
        sc.spawn(move |_| loop {
            for _ in 0..65 {
                w.push(Box::new(1));
            }
            for _ in 0..40 {
                w.pop();
            }
        });
        sc.spawn(move |_| loop {
            let w2 = Worker::new_fifo();
            s.steal_batch(&w2);
        });
    })
    .unwrap();
}
