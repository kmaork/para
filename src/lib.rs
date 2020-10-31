
mod producer;
use producer::*;
mod consumer;
use consumer::*;
mod scheduler;
use scheduler::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test() {
        let print = |x| println!("{}", x);
        // let sleep = |x| {
        //     thread::sleep(Duration::from_secs(3));
        //     x
        // };
        let mut prod = vec!(1, 2, 3).pipe(&print);

        let s = Scheduler::new();
        s.add_task(Box::new(ProduceTask { producer: &mut prod }));
        s.run();
    }
}











// first priority is unparallelizable nodes