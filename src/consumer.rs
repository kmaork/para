use crate::scheduler::*;
use rich_phantoms::PhantomCovariantAlwaysSendSync;

pub trait Consumer<'a, D>: Sized + Sync {
    fn consume(&'a self, data: D, scheduler: &Scheduler<'a>);
}

impl<'a, D, F: Fn(D) + Sync> Consumer<'a, D> for F {
    fn consume(&'a self, data: D, _scheduler: &Scheduler<'a>) {
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

pub struct Map<'a, I, O, F: Fn(I) -> O, C: Consumer<'a, O>> {
    func: F,
    _i: PhantomCovariantAlwaysSendSync<I>,
    _o: PhantomCovariantAlwaysSendSync<O>,
    consumer: &'a C,
}

impl<'a, I: Send, O: Send, F: Fn(I) -> O + Sync, C: Consumer<'a, O>> Consumer<'a, I> for Map<'a, I, O, F, C> {
    fn consume(&'a self, data: I, scheduler: &Scheduler<'a>) {
        scheduler.add_task(Box::new(ConsumeTask { data: (self.func)(data), consumer: self.consumer }))
    }
}

pub trait IntoMap<'a, I, O, F: Fn(I) -> O> {
    fn pipe<C: Consumer<'a, O>>(self, consumer: &'a C) -> Map<'a, I, O, F, C>;
}

impl<'a, I, O, F: Fn(I) -> O> IntoMap<'a, I, O, F> for F {
    fn pipe<C: Consumer<'a, O>>(self, consumer: &'a C) -> Map<'a, I, O, F, C> {
        Map { func: self, _i: Default::default(), _o: Default::default(), consumer }
    }
}