use crate::consumer::{ConsumeTask, Consumer};
use crate::scheduler::TaskManager;
use crate::util::repeat;

pub struct Fanout<'a, D: Clone + Send> {
    consumers: Vec<&'a dyn Consumer<'a, D>>,
}

impl<'a, D: Clone + Send> Fanout<'a, D> {
    pub fn new(consumers: Vec<&'a dyn Consumer<'a, D>>) -> Self {
        Self { consumers }
    }
}

impl<'a, D: Clone + Send> Consumer<'a, D> for Fanout<'a, D> {
    fn consume(&'a self, data: D, manager: &mut TaskManager<'a>) {
        for (&consumer, data) in (&self.consumers)
            .iter()
            .zip(repeat(data, self.consumers.len()))
        {
            manager.add_task(Box::new(ConsumeTask::new(consumer, data)));
        }
    }
}
