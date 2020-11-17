use crate::consumer::{ConsumeTask, Consumer};
use crate::Scheduler;

pub struct Fanout<'a, D: Clone + Send> {
    consumers: Vec<&'a dyn Consumer<'a, D>>,
}

impl<'a, D: Clone + Send> Fanout<'a, D> {
    pub fn new(consumers: Vec<&'a dyn Consumer<'a, D>>) -> Self {
        Self { consumers }
    }
}

impl<'a, D: Clone + Send> Consumer<'a, D> for Fanout<'a, D> {
    fn consume(&'a self, data: D, scheduler: &Scheduler<'a>) {
        for consumer in &self.consumers {
            let cref: &'a dyn Consumer<'a, D> = *consumer;
            let task = ConsumeTask::new(cref, data.clone()); // TODO: save one clone
            let b = Box::new(task);
            scheduler.add_task(b);
        }
    }
}
