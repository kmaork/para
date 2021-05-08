mod notifier;

use crossbeam::deque::{Injector, Steal, Stealer, Worker};
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
    //TODO: maybe use lifetimes instead of arc? like in the rest of the code?
    notifier: Notifier,
    stealers: Vec<Stealer<DynTask<'a>>>,
}

impl<'a> Scheduler<'a> {
    pub fn group(size: usize) -> Vec<Self> {
        // TODO: instead of cloning many arcs, maybe have one shared state struct like in
        // https://github.com/tokio-rs/tokio/blob/master/tokio/src/runtime/thread_pool/worker.rs#L70
        let notifier = Notifier::new();
        let thread_queues: Vec<_> = (0..size).map(|_| Worker::new_lifo()).collect();
        let stealer_lists: Vec<Vec<Stealer<DynTask<'a>>>> = (0..size)
            .map(|queue_idx| {
                (0..size)
                    .filter(|&i| i != queue_idx)
                    .map(|i| thread_queues[i].stealer())
                    .collect()
            })
            .collect();
        thread_queues
            .into_iter()
            .zip(stealer_lists.into_iter())
            .map(|(thread_queue, stealers)| Self {
                thread_queue,
                notifier: notifier.clone(),
                stealers,
            })
            .collect()
    }

    pub fn add_task(&self, task: DynTask<'a>) {
        if self.thread_queue.len() < THREAD_QUEUE_SIZE {
            self.thread_queue.push(task);
        } else {
            // self.notifier.added_work();
        }
    }

    fn steal_from_peer(&self) -> Option<DynTask<'a>> {
        for stealer in self.stealers.iter() {
            // let size = self.thread_queue.len();
            //
            if let Steal::Success(_) = stealer.steal_batch(&self.thread_queue) {
                return self.thread_queue.pop();
            }
        }
        None
    }
}

impl<'a> Iterator for Scheduler<'a> {
    type Item = DynTask<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(task) = self.thread_queue.pop() {
                return Some(task);
            } else if let Some(task) = self.steal_from_peer() {
                return Some(task);
            } else {
                if let Notification::Done = self.notifier.wait_for_work() {
                    return None;
                }
            }
        }
    }
}
