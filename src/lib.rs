#![allow(incomplete_features)]

mod consumer;
mod producer;
mod scheduler;
mod util;

pub use consumer::{Fanout, Mapper};
pub use producer::{IntoIteratorProducer, Producer};

use crossbeam::thread;
use scheduler::{Scheduler, TaskGenerator};

pub fn schedule<'a>(producers: &'a mut [&'a mut (dyn TaskGenerator<'a> + 'a)], num_threads: usize) {
    let scheduler_group = Scheduler::group(num_threads);
    for (producer, scheduler) in producers.iter_mut().zip(scheduler_group.iter().cycle()) {
        scheduler.add_task(producer.first_task());
    }
    thread::scope(|s| {
        for (_thread_num, mut scheduler) in scheduler_group.into_iter().enumerate() {
            s.spawn(move |_| {
                let mut _n = 0;
                while let Some(task) = scheduler.next() {
                    task.run(&mut scheduler);
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

#[macro_export]
macro_rules! run_pipeline_reversed {
    ($producer:expr) => {
        schedule(&mut [&mut $producer], 2);
    };
    ($node1:expr=>$node2:expr$(=>$node:expr)*) => {
        let local_node = $node1;
        let mut new_consumer = $node2.pipe(&local_node);
        run_pipeline_reversed!(new_consumer$(=>$node)*);
    };
}

#[macro_export]
macro_rules! run_pipeline {
    (;$($reversed:expr)=>+) => {run_pipeline_reversed!($($reversed)=>*)};
    ($first:expr$(=>$original:expr)*$(;$($reversed:expr)=>*)?) => {run_pipeline!($($original)=>*;$first$($(=>$reversed)*)*)};
}
