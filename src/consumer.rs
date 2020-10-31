use crate::scheduler::*;

pub trait Consumer<'a, D>: Sized {
    fn consume(&self, data: D, scheduler: &Scheduler<'a>);
}

impl<'a, D, F: Fn(D)> Consumer<'a, D> for F {
    fn consume(&self, data: D, _scheduler: &Scheduler<'a>) {
        self(data);
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