mod iterator_producer;

use crate::consumer::{ConsumeTask, Consumer};
use crate::scheduler::{DynTask, Scheduler, Task, TaskGenerator};
pub use iterator_producer::IntoIteratorProducer;

pub trait Producer<'a>: Sized + Send {
    type Data: Send;
    type Consumer: Consumer<'a, Self::Data>;

    fn get_next_product(&mut self) -> Option<Self::Data>;
    fn consumer(&self) -> &'a Self::Consumer;
    fn produce(&'a mut self, scheduler: &mut Scheduler<'a>) {
        if let Some(data) = self.get_next_product() {
            scheduler.add_task(Box::new(ConsumeTask::new(self.consumer(), data)));
            scheduler.add_task(Box::new(ProduceTask { producer: self }))
        };
    }
}

pub struct ProduceTask<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data = D, Consumer = C>> {
    pub producer: &'a mut P,
}

impl<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data = D, Consumer = C>> Task<'a>
    for ProduceTask<'a, D, C, P>
{
    fn run(self: Box<Self>, scheduler: &mut Scheduler<'a>) {
        self.producer.produce(scheduler);
    }
}

impl<'a, P: Producer<'a>> TaskGenerator<'a> for P {
    fn first_task(&'a mut self) -> DynTask<'a> {
        Box::new(ProduceTask { producer: self })
    }
}
