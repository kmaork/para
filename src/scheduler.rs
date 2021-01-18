use crate::util::Circus;
use crossbeam::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};
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

#[derive(Clone)]
struct GlobalTaskQueue<'a> {
    sender: Sender<DynTask<'a>>,
    receiver: Receiver<DynTask<'a>>,
}

impl<'a> GlobalTaskQueue<'a> {
    fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    fn push(&self, task: DynTask<'a>) {
        self.sender.send(task).unwrap();
    }

    fn pop(&self) -> DynTask<'a> {
        self.receiver.recv().unwrap()
    }
}

pub struct TaskManager<'a> {
    thread_queue: Circus<DynTask<'a>, THREAD_QUEUE_SIZE>,
    global_queue: GlobalTaskQueue<'a>,
    scheduler: Arc<Scheduler>,
}

impl<'a> TaskManager<'a> {
    fn new(global_queue: GlobalTaskQueue<'a>, scheduler: Arc<Scheduler>) -> Self {
        Self {
            thread_queue: Circus::new(),
            global_queue,
            scheduler,
        }
    }

    pub fn add_task(&mut self, task: DynTask<'a>) {
        if self.thread_queue.can_push() {
            self.thread_queue.push(task).unwrap();
        } else {
            self.global_queue.push(task);
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
                let task = self.global_queue.pop();
                self.scheduler.done_blocking();
                Some(task)
            } else {
                self.global_queue.push(Box::new(()));
                None
            }
        }
    }
}

pub trait TaskGenerator<'a> {
    fn first_task(&'a mut self) -> DynTask<'a>;
}

struct Scheduler {
    count: AtomicU32,
}

impl Scheduler {
    fn new(num_threads: u32) -> Self {
        Self {
            count: AtomicU32::new(num_threads),
        }
    }

    fn start_blocking_if_someone_else_is_not(&self) -> bool {
        self.count.fetch_sub(1, Ordering::AcqRel) > 1
    }

    fn done_blocking(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);
    }
}

pub fn schedule<'a>(producers: &'a mut [&'a mut (dyn TaskGenerator<'a> + 'a)], num_threads: u32) {
    let global_queue = GlobalTaskQueue::new();
    for producer in producers {
        global_queue.push(producer.first_task());
    }
    let scheduler = Arc::new(Scheduler::new(num_threads));
    thread::scope(|s| {
        for _thread_num in 0..num_threads {
            let global_queue_clone = global_queue.clone();
            let scheduler_clone = Arc::clone(&scheduler);
            s.spawn(move |_| {
                let mut manager = TaskManager::new(global_queue_clone, scheduler_clone);
                let mut _n = 0;
                while let Some(task) = manager.next() {
                    task.run(&mut manager);
                    #[cfg(debug_assertions)]
                    {
                        _n += 1;
                    }
                }
                #[cfg(debug_assertions)]
                {
                    println!("Thread {}: {} tasks", _thread_num, _n);
                }
            });
        }
    })
    .unwrap();
}
