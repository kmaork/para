use crossbeam::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};

pub trait Task<'a> {
    fn run(self: Box<Self>, scheduler: &Scheduler<'a>);
}

pub struct Scheduler<'a> {
    task_sender: Sender<Box<dyn Task<'a> + 'a + Send>>,
    task_receiver: Receiver<Box<dyn Task<'a> + 'a + Send>>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = unbounded();
        Self {
            task_sender,
            task_receiver,
        }
    }

    pub fn add_task(&self, task: Box<dyn Task<'a> + 'a + Send>) {
        self.task_sender.send(task).unwrap();
    }

    pub fn run(self, num_threads: usize) {
        let sync = Arc::new((Mutex::new(0), Condvar::new()));
        thread::scope(|s| {
            for _ in 0..num_threads {
                let sync_clone = sync.clone();
                let receiver = self.task_receiver.clone();
                let sched_ref = &self;
                s.spawn(move |_| {
                    let (lock, cvar) = &*sync_clone;
                    loop {
                        for task in receiver.try_iter() {
                            task.run(sched_ref);
                            // We might have created new work, so we wanna wake up some workers
                            cvar.notify_all();
                        }
                        let mut waiting_amount = lock.lock().unwrap();
                        *waiting_amount += 1;
                        if *waiting_amount == num_threads {
                            cvar.notify_all();
                            return;
                        }
                        waiting_amount = cvar.wait(waiting_amount).unwrap();
                        if *waiting_amount == num_threads {
                            return;
                        }
                        *waiting_amount -= 1;
                    }
                });
            }
        })
        .unwrap();
    }
}
