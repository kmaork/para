use crate::consumer::{ConsumeTask, Consumer};
use crate::scheduler::Scheduler;
use rich_phantoms::PhantomCovariantAlwaysSendSync;

pub struct Map<'a, I, O, F: Fn(I) -> O, C: Consumer<'a, O>> {
    func: F,
    _i: PhantomCovariantAlwaysSendSync<I>,
    _o: PhantomCovariantAlwaysSendSync<O>,
    consumer: &'a C,
}

impl<'a, I: Send, O: Send, F: Fn(I) -> O + Sync, C: Consumer<'a, O>> Consumer<'a, I>
    for Map<'a, I, O, F, C>
{
    fn consume(&'a self, data: I, scheduler: &Scheduler<'a>) {
        scheduler.add_task(Box::new(ConsumeTask {
            data: (self.func)(data),
            consumer: self.consumer,
        }))
    }
}

pub trait IntoMap<'a, I, O, F: Fn(I) -> O> {
    fn pipe<C: Consumer<'a, O>>(self, consumer: &'a C) -> Map<'a, I, O, F, C>;
}

impl<'a, I, O, F: Fn(I) -> O> IntoMap<'a, I, O, F> for F {
    fn pipe<C: Consumer<'a, O>>(self, consumer: &'a C) -> Map<'a, I, O, F, C> {
        Map {
            func: self,
            _i: Default::default(),
            _o: Default::default(),
            consumer,
        }
    }
}
