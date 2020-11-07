#![feature(in_band_lifetimes)]

mod consumer;
mod producer;
mod scheduler;

pub use consumer::Mapper;
pub use producer::{IntoIteratorProducer, Producer};
pub use scheduler::Scheduler;

#[macro_export]
macro_rules! pipeline {
    ($threads:expr, $($producer:expr),+) => {
        let s = Scheduler::new();
        $($producer.add_to_scheduler(&s);)*
        s.run($threads);
    }
}
