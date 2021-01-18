#![allow(incomplete_features)]
#![feature(in_band_lifetimes)]
#![feature(const_generics)]
#![feature(maybe_uninit_uninit_array)]

mod consumer;
mod producer;
mod scheduler;
mod util;

pub use consumer::{Fanout, Mapper};
pub use producer::{IntoIteratorProducer, Producer};
pub use scheduler::schedule;

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
