use crate::consumer::*;
use crate::scheduler::*;

pub trait Producer<'a>: Sized + Send {
    // TODO: remove
    type Data: Send;
    type Consumer: Consumer<'a, Self::Data>;

    fn get_next_product(&mut self) -> Option<Self::Data>;
    fn consumer(&self) -> &'a Self::Consumer;
    fn produce(&'a mut self, scheduler: &Scheduler<'a>) {
        if let Some(data) = self.get_next_product() {
            scheduler.add_task(Box::new(ConsumeTask { consumer: self.consumer(), data }));
            scheduler.add_task(Box::new(ProduceTask { producer: self }));
            // self.produce(scheduler); // TODO: maybe if we have a thread-local task queue we could add it there
        };
    }
}

pub trait IntoIteratorProducer<'a, D, I: Iterator<Item=D>> {
    fn pipe<C: Consumer<'a, D>>(self, consumer: &'a C) -> IteratorProducer<'a, D, I, C>;
}

impl<'a, D, I: IntoIterator<Item=D>> IntoIteratorProducer<'a, D, I::IntoIter> for I {
    fn pipe<C: Consumer<'a, D>>(self, consumer: &'a C) -> IteratorProducer<'a, D, I::IntoIter, C> {
        IteratorProducer { iter: self.into_iter(), consumer }
    }
}

pub struct IteratorProducer<'a, D, I: Iterator<Item=D>, C: Consumer<'a, D>> {
    iter: I,
    consumer: &'a C,
}

impl<'a, D: Send, I: Iterator<Item=D> + Send, C: Consumer<'a, D>> Producer<'a> for IteratorProducer<'a, D, I, C> {
    type Data = D;
    type Consumer = C;

    fn get_next_product(&mut self) -> Option<Self::Data> {
        self.iter.next()
    }

    fn consumer(&self) -> &'a Self::Consumer {
        self.consumer
    }
}

pub struct ProduceTask<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data=D, Consumer=C>> {
    pub producer: &'a mut P
}

impl<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data=D, Consumer=C>> Task<'a> for ProduceTask<'a, D, C, P> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>) {
        self.producer.produce(scheduler);
    }
}