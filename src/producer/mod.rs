mod iterator_producer;

use crate::consumer::{ConsumeTask, Consumer};
use crate::scheduler::{DynTask, Task, TaskGenerator, TaskManager};
pub use iterator_producer::IntoIteratorProducer;

pub trait Producer<'a>: Sized + Send {
    type Data: Send;
    type Consumer: Consumer<'a, Self::Data>;

    fn get_next_product(&mut self) -> Option<Self::Data>;
    fn consumer(&self) -> &'a Self::Consumer;
    fn produce(&'a mut self, manager: &mut TaskManager<'a>) {
        if let Some(data) = self.get_next_product() {
            manager.add_task(Box::new(ConsumeTask::new(self.consumer(), data)));
            manager.add_task(Box::new(ProduceTask { producer: self }))
        };
    }
}

pub struct ProduceTask<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data = D, Consumer = C>> {
    pub producer: &'a mut P,
}

impl<'a, D: Send, C: Consumer<'a, D>, P: Producer<'a, Data = D, Consumer = C>> Task<'a>
    for ProduceTask<'a, D, C, P>
{
    fn run(self: Box<Self>, manager: &mut TaskManager<'a>) {
        self.producer.produce(manager);
    }
}

impl<'a, P: Producer<'a>> TaskGenerator<'a> for P {
    fn first_task(&'a mut self) -> DynTask<'a> {
        Box::new(ProduceTask { producer: self })
    }
}
