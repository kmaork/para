use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::thread;

fn main() {
    let w = Worker::<Box<u128>>::new_lifo();
    let s = w.stealer();
    let w2 = Worker::new_fifo();
    let max = 200;
    eprintln!("max {}", max);
    thread::scope(|sc| {
        sc.spawn(move |_| loop {
            for x in 0..max {
                for i in 0..x {
                    for j in 0..x {
                        for _ in 0..i {
                            w.push(Box::new(1));
                        }
                        for _ in 0..j {
                            if let Some(y) = w.pop() {
                                if *y != 1 {
                                    println!("{}, {}, {}", x, i, j);
                                }
                            }
                        }
                    }
                }
            }
        });
        sc.spawn(move |_| loop {
            for _ in 0..max {
                s.steal_batch(&w2);
                for x in 0..5 {
                    w2.push(Box::new(1));
                    w2.pop();
                }
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
