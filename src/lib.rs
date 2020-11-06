mod consumer;
mod producer;
mod scheduler;

pub use consumer::IntoMap;
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

// TODO: automatic buffering of work might decrease synchronization overhead,
// but increase memory usage and can cause jitters if some nodes require smoothness
// TODO: first priority is unparallelizable nodes
// TODO: add support for clonable producer
// TODO: add support for IO
// TODO: allow stateless producers (makes sense?)
// TODO: optimize stateful consumers, don't wait for them while there is other work
// TODO: per thread queue with work stealing.
// TODO: it'd be ideal if the scheduler could own the nodes, as it wouldn't require the user to keep them on his stack.
// same goes for the producer. Other option is a recursive macro that defines the consumers and then attaches to a producer and runs the pipeline.
