use crossbeam::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::util::Circus;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

const THREAD_QUEUE_SIZE: usize = 100;

pub trait Task<'a> {
    fn run(self: Box<Self>, manager: &mut TaskManager<'a>);
}

impl<'a> Task<'a> for () {
    fn run(self: Box<Self>, _manager: &mut TaskManager<'_>) {}
}

pub type DynTask<'a> = Box<dyn Task<'a> + 'a + Send>;

pub struct TaskManager<'a> {
    thread_queue: Circus<DynTask<'a>, THREAD_QUEUE_SIZE>,
    global_sender: Sender<DynTask<'a>>,
    global_receiver: Receiver<DynTask<'a>>,
    scheduler: Arc<Scheduler>,
}

impl<'a> TaskManager<'a> {
    fn new(global_sender: Sender<DynTask<'a>>, global_receiver: Receiver<DynTask<'a>>, scheduler: Arc<Scheduler>) -> Self {
        Self { thread_queue: Circus::new(), global_sender, global_receiver, scheduler }
    }

    pub fn add_task(&mut self, task: DynTask<'a>) {
        if self.thread_queue.can_push() {
            self.thread_queue.push(task).unwrap();
        } else {
            self.global_sender.send(task).unwrap();
        }
    }
}

impl<'a> Iterator for TaskManager<'a> {
    type Item = DynTask<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(task) = self.thread_queue.pop() {
            Some(task)
        } else {
            if self.scheduler.start_blocking_if_someone_else_is_not() {
                let task = self.global_receiver.recv().unwrap();
                self.scheduler.done_blocking();
                Some(task)
            } else {
                self.global_sender.send(Box::new(())).unwrap();
                None
            }
        }
    }
}

pub trait TaskGenerator<'a> {
    fn first_task(&'a mut self) -> DynTask<'a>;
}

struct Scheduler {
    count: AtomicU32
}

impl Scheduler {
    fn new(num_threads: u32) -> Self {
        Self { count: AtomicU32::new(num_threads) }
    }

    fn start_blocking_if_someone_else_is_not(&self) -> bool {
        self.count.fetch_sub(1, Ordering::AcqRel) > 1// TODO: what?
    }

    fn done_blocking(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);// TODO: what?
    }
}

pub fn schedule<'a>(producers: &'a mut [&'a mut (dyn TaskGenerator<'a> + 'a)], num_threads: u32) {
    //TODO: global_sender and receiver should go inside scheduler
    let (global_sender, global_receiver) = unbounded();
    for producer in producers {
        global_sender.send(producer.first_task()).unwrap();
    }
    let scheduler = Arc::new(Scheduler::new(num_threads));
    thread::scope(|s| {
        for _ in 0..num_threads {
            let global_sender_clone = global_sender.clone();
            let global_receiver_clone = global_receiver.clone();
            let scheduler_clone = Arc::clone(&scheduler);
            s.spawn(|_| {
                let mut manager = TaskManager::new(global_sender_clone, global_receiver_clone, scheduler_clone);
                while let Some(task) = manager.next() {
                    task.run(&mut manager);
                }
            });
        }
        drop(global_sender);
        drop(global_receiver);
    }).unwrap();
}
