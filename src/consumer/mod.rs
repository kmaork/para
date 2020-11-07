mod map;
pub use map::Mapper;

use crate::scheduler::{Scheduler, Task};
use std::sync::Mutex;

pub trait Consumer<'a, D>: Sized + Sync {
    fn consume(&'a self, data: D, scheduler: &Scheduler<'a>);
}

impl<'a, D, F: Fn(D) + Sync> Consumer<'a, D> for F {
    fn consume(&'a self, data: D, _scheduler: &Scheduler<'a>) {
        self(data);
    }
}

impl<'a, D, F: FnMut(D) + Send> Consumer<'a, D> for Mutex<F> {
    fn consume(&'a self, data: D, _scheduler: &Scheduler<'a>) {
        (*self.lock().unwrap())(data);
    }
}

pub struct ConsumeTask<'a, D, C> {
    pub consumer: &'a C,
    pub data: D,
}

impl<'a, D, C: Consumer<'a, D>> Task<'a> for ConsumeTask<'a, D, C> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>) {
        self.consumer.consume(self.data, scheduler);
    }
}
