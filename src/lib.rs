
mod producer;
use producer::*;
mod consumer;
use consumer::*;
mod scheduler;
use scheduler::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test() {
        let start = Instant::now();
        let print = |x| { x - 1; };
        let mult = (|x| x * 2).pipe(&print);
        let mut prod = (1..2000000).pipe(&mult);

        let s = Scheduler::new();
        s.add_task(Box::new(ProduceTask { producer: &mut prod }));
        s.run(1);
        println!("{:?}", Instant::now() - start);
    }
}

// first priority is unparallelizable nodes