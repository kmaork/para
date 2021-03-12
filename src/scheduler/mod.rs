mod notifier;

use crossbeam::deque::{Injector, Worker};
use notifier::{Notification, Notifier};
use std::sync::Arc;

const THREAD_QUEUE_SIZE: usize = 100;

pub trait Task<'a> {
    fn run(self: Box<Self>, scheduler: &mut Scheduler<'a>);
}

pub type DynTask<'a> = Box<dyn Task<'a> + 'a + Send>;

pub trait TaskGenerator<'a> {
    fn first_task(&'a mut self) -> DynTask<'a>;
}

pub struct Scheduler<'a> {
    thread_queue: Worker<DynTask<'a>>,
    global_queue: Arc<Injector<DynTask<'a>>>,
    //TODO: maybe use lifetimes instead of arc? like in the rest of the code?
    notifier: Notifier,
}

impl<'a> Scheduler<'a> {
    pub fn group(size: usize) -> Vec<Self> {
        // TODO: instead of cloning many arcs, maybe have one shared state struct like in
        // https://github.com/tokio-rs/tokio/blob/master/tokio/src/runtime/thread_pool/worker.rs#L70
        let global_queue = Arc::new(Injector::new());
        let thread_queues: Vec<_> = (0..size).map(|_| Worker::new_fifo()).collect();
        let notifier = Notifier::new();
        thread_queues
            .into_iter()
            .map(|thread_queue| Self {
                thread_queue,
                global_queue: Arc::clone(&global_queue),
                notifier: notifier.clone(),
            })
            .collect()
    }

    pub fn add_task(&self, task: DynTask<'a>) {
        if self.thread_queue.len() < THREAD_QUEUE_SIZE {
            self.thread_queue.push(task);
        } else {
            self.global_queue.push(task);
            self.notifier.added_work();
            // TODO: ultimately steal a batch into the global queue
        }
    }
}

impl<'a> Iterator for Scheduler<'a> {
    type Item = DynTask<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(task) = self.thread_queue.pop() {
                return Some(task);
            } else if let Some(task) = self.thread_queue.pop() {
                return Some(task);
            } else {
                if let Notification::Done = self.notifier.wait_for_work() {
                    return None;
                }
            }
        }
    }
}
