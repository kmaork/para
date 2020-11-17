#![feature(in_band_lifetimes)]

mod consumer;
mod producer;
mod scheduler;

pub use consumer::Mapper;
pub use producer::{IntoIteratorProducer, Producer};
pub use scheduler::Scheduler;

#[macro_export]
macro_rules! pipeline_reversed {
    ($producer:expr) => {
        let scheduler = Scheduler::new();
        $producer.add_to_scheduler(&scheduler);
        scheduler.run(4);
    };
    ($node1:expr=>$node2:expr$(=>$node:expr)*) => {
        let local_node = $node1;
        let mut new_consumer = $node2.pipe(&local_node);
        pipeline_reversed!(new_consumer$(=>$node)*);
    };
}

#[macro_export]
macro_rules! pipeline {
    (;$($reversed:expr)=>+) => {pipeline_reversed!($($reversed)=>*)};
    ($first:expr$(=>$original:expr)*$(;$($reversed:expr)=>*)?) => {pipeline!($($original)=>*;$first$($(=>$reversed)*)*)};
}
