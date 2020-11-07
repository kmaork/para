use crate::consumer::{ConsumeTask, Consumer};
use crate::scheduler::Scheduler;
use rich_phantoms::PhantomCovariantAlwaysSendSync;
use std::sync::Mutex;

pub trait Mapper<I, O>: Sized {
    fn map(&self, input: I) -> O;
    fn pipe<C: Consumer<'a, O>>(self, consumer: &'a C) -> Map<'a, I, O, Self, C> {
        Map {
            mapper: self,
            _i: Default::default(),
            _o: Default::default(),
            consumer,
        }
    }
}

impl<I, O, F: Fn(I) -> O> Mapper<I, O> for F {
    fn map(&self, input: I) -> O {
        self(input)
    }
}

impl<I, O, F: FnMut(I) -> O> Mapper<I, O> for Mutex<F> {
    fn map(&self, input: I) -> O {
        (*self.lock().unwrap())(input)
    }
}

pub struct Map<'a, I, O, M: Mapper<I, O>, C: Consumer<'a, O>> {
    mapper: M,
    // We need PhantomData for I and O because they are unused, but PhantomData acts
    // as if they were really a part of our struct, which means if they are
    // Send/Sync, our struct won't be as well.
    // To prevent this we use PhantomCovariantAlwaysSendSync, which is ok because we don't actually
    // have I or O as a part of our struct.
    _i: PhantomCovariantAlwaysSendSync<I>,
    _o: PhantomCovariantAlwaysSendSync<O>,
    consumer: &'a C,
}

impl<'a, I: Send, O: Send, M: Mapper<I, O> + Sync, C: Consumer<'a, O>> Consumer<'a, I>
    for Map<'a, I, O, M, C>
{
    fn consume(&'a self, data: I, scheduler: &Scheduler<'a>) {
        scheduler.add_task(Box::new(ConsumeTask {
            data: (self.mapper).map(data),
            consumer: self.consumer,
        }))
    }
}
