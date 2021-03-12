mod fanout;
mod map;

pub use fanout::Fanout;
pub use map::Mapper;

use crate::scheduler::{Scheduler, Task};
use std::sync::Mutex;

pub trait Consumer<'a, D>: Sync {
    fn consume(&'a self, data: D, scheduler: &mut Scheduler<'a>);
}

impl<'a, D, F: Fn(D) + Sync> Consumer<'a, D> for F {
    fn consume(&'a self, data: D, _scheduler: &mut Scheduler<'a>) {
        self(data);
    }
}

impl<'a, D, F: FnMut(D) + Send> Consumer<'a, D> for Mutex<F> {
    fn consume(&'a self, data: D, _scheduler: &mut Scheduler<'a>) {
        (*self.lock().unwrap())(data);
    }
}

pub struct ConsumeTask<'a, D, C: Consumer<'a, D> + ?Sized> {
    consumer: &'a C,
    data: D,
}

impl<'a, D, C: Consumer<'a, D> + ?Sized> ConsumeTask<'a, D, C> {
    pub fn new(consumer: &'a C, data: D) -> Self {
        Self { consumer, data }
    }
}

impl<'a, D, C: Consumer<'a, D> + ?Sized> Task<'a> for ConsumeTask<'a, D, C> {
    fn run(self: Box<Self>, scheduler: &mut Scheduler<'a>) {
        self.consumer.consume(self.data, scheduler);
    }
}
