use crossbeam;
use crossbeam::deque::{Steal, Stealer, Worker};
use std::thread;
use std::time::Duration;

fn main() {
    let t = 1;
    let w = Worker::<Box<u128>>::new_lifo();
    let ss: Vec<_> = (0..t).map(|_| w.stealer()).collect();
    crossbeam::thread::scope(|sc| {
        sc.spawn(move |_| loop {
            for x in 0..1000 {
                for i in 0..x {
                    for j in 0..x {
                        for _ in 0..i {
                            w.push(Box::new(1));
                        }
                        for _ in 0..j {
                            if let Some(y) = w.pop() {
                                assert_eq!(*y, 1);
                            }
                        }
                    }
                }
            }
        });
        for s in ss {
            let w2 = Worker::new_lifo();
            sc.spawn(move |_| loop {
                for i in 0..1000 {
                    for _ in 0..i {
                        s.steal_batch(&w2);
                    }
                    while !w2.is_empty() {
                        assert_eq!(*w2.pop().unwrap(), 1);
                    }
                }
            });
        }
    })
    .unwrap();
}
