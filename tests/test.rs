use para::scheduler::*;
use para::consumer::*;
use para::producer::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test() {
        let start = Instant::now();

        // define pipeline
        let print = |x| println!("{}", x);
        let mult = (|x| x * 2).pipe(&print);
        let mut prod = (1..10).pipe(&mult);

        let s = Scheduler::new();
        prod.add_to_scheduler(&s);
        s.run(4);
        println!("{:?}", Instant::now() - start);
    }
}