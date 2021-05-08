use crossbeam::deque::{Steal, Stealer, Worker};
use crossbeam::thread;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

thread_local!(static FOO: RefCell<HashSet<u64>> = RefCell::new(HashSet::new()));
struct Yay {
    i: u64,
    m: Arc<Mutex<HashSet<u64>>>,
}
impl Yay {
    fn new(i: u64, m: Arc<Mutex<HashSet<u64>>>) -> Self {
        Self { i, m }
    }
}
impl Drop for Yay {
    fn drop(&mut self) {
        FOO.with(|f| {
            assert!(f.borrow_mut().insert(self.i));
        });
        if !self.m.lock().unwrap().insert(self.i) {
            println!("{}", self.i);
        }
    }
}
impl Deref for Yay {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.i
    }
}

fn main() {
    let w = Worker::new_lifo();
    let s = w.stealer();
    let mut i = 0;
    thread::scope(|sc| {
        let m = Arc::new(Mutex::new(HashSet::new()));
        sc.spawn(move |_| loop {
            for _ in 0..65 {
                w.push(Yay::new(i, m.clone()));
                i += 1;
            }
            for _ in 0..40 {
                w.pop();
            }
        });
        sc.spawn(move |_| loop {
            let w2 = Worker::new_fifo();
            for _ in 0..100 {
                s.steal_batch(&w2);
            }
        });
    })
    .unwrap();
}
