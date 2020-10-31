use crossbeam_channel::{Sender, Receiver, unbounded};

pub trait Task<'a> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>);
}

pub struct Scheduler<'a> {
    task_sender: Sender<Box<dyn Task<'a> + 'a>>,
    task_receiver: Receiver<Box<dyn Task<'a> + 'a>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = unbounded();
        Self { task_sender, task_receiver }
    }

    pub fn add_task(&self, task: Box<dyn Task<'a> + 'a>) {
        // TODO: add support for clonable producer
        // TODO: add suppoer for IO
        self.task_sender.send(task).unwrap();
    }

    pub fn run(self) {
        for task in self.task_receiver.try_iter() {
            task.run(&self);
        }
    }
}