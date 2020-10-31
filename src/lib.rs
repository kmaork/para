use crossbeam_channel::{Sender, Receiver, unbounded};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let print = Box::new(|x| println!("{}", x));
        let mut prod = IteratorProducer { iter: vec!(1, 2, 3).into_iter(), consumer: &print };
        {
            let s = Scheduler::new();
            s.add_task(Box::new(ProduceTask { producer: &mut prod }));
            s.run();
        }
    }
}

trait Task<'a> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>);
}

struct ProduceTask<'a, D, C: Consumer<'a, D>, P: Producer<'a, Data=D, Consumer=C>> {
    producer: &'a mut P
}

impl<'a, D, C: Consumer<'a, D>, P: Producer<'a, Data=D, Consumer=C>> Task<'a> for ProduceTask<'a, D, C, P> {
    fn run(self: Box<Self<>>, scheduler: &Scheduler<'a>) {
        self.producer.produce(scheduler);
    }
}

struct ConsumeTask<'a, D, C> {
    consumer: &'a C,
    data: D,
}

impl<'a, D, C: Consumer<'a, D>> Task<'a> for ConsumeTask<'a, D, C> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>) {
        self.consumer.consume(self.data, scheduler);
    }
}

trait Producer<'a>: Sized {
    // TODO: remove
    type Data;
    type Consumer: Consumer<'a, Self::Data>;

    fn get_next_product(&mut self) -> Option<Self::Data>;
    fn consumer(&self) -> &'a Self::Consumer;
    fn produce(&'a mut self, scheduler: &Scheduler<'a>) {
        if let Some(data) = self.get_next_product() {
            scheduler.add_task(Box::new(ConsumeTask { consumer: self.consumer(), data }));
            scheduler.add_task(Box::new(ProduceTask { producer: self }));
        };
    }
}

trait Consumer<'a, D>: Sized {
    fn consume(&self, data: D, scheduler: &Scheduler<'a>);
}

impl<'a, D, F: Fn(D)> Consumer<'a, D> for F {
    fn consume(&self, data: D, _scheduler: &Scheduler<'a>) {
        self(data);
    }
}

struct IteratorProducer<'a, D, I: Iterator<Item=D>, C: Consumer<'a, D>> {
    iter: I,
    consumer: &'a C,
}

impl<'a, D, I: Iterator<Item=D>, C: Consumer<'a, D>> Producer<'a> for IteratorProducer<'a, D, I, C> {
    type Data = D;
    type Consumer = C;

    fn get_next_product(&mut self) -> Option<Self::Data> {
        self.iter.next()
    }

    fn consumer(&self) -> &'a Self::Consumer {
        self.consumer
    }
}


struct Scheduler<'a> {
    task_sender: Sender<Box<dyn Task<'a> + 'a>>,
    task_receiver: Receiver<Box<dyn Task<'a> + 'a>>,
}

impl<'a> Scheduler<'a> {
    fn new() -> Self {
        let (task_sender, task_receiver) = unbounded();
        Self { task_sender, task_receiver }
    }

    fn add_task(&self, task: Box<dyn Task<'a> + 'a>) {
        // TODO: add support for clonable producer
        // TODO: add suppoer for IO
        self.task_sender.send(task).unwrap();
    }

    fn run(self) {
        for task in self.task_receiver.try_iter() {
            task.run(&self);
        }
    }
}

// first priority is unparallelizable nodes