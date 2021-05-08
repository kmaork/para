use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::thread;
use std::time::Duration;

fn main() {
    let w = Worker::<Box<u128>>::new_lifo();
    let s = w.stealer();
    let w2 = Worker::new_fifo();
    thread::scope(|sc| {
        sc.spawn(move |_| loop {
            for _ in 0..65 {
                w.push(Box::new(1));
            }
            for i in 0..40 {
                if let Some(y) = w.pop() {
                    if *y != 1 {
                        println!("{}", i);
                    }
                }
            }
        });
        sc.spawn(move |_| loop {
            for _ in 0..100 {
                s.steal_batch(&w2);
            }
            while !w2.is_empty() {
                if *w2.pop().unwrap() != 1 {
                    // println!("bad");
                }
            }
        });
    })
    .unwrap();
}
