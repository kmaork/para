use para::Scheduler;
use para::*;
use std::collections::HashSet;
use std::sync::Mutex;

#[test]
fn test_with_macro() {
    run_pipeline!(vec!(1, 2, 3) => |x| x + 1 => |x| println!("{}", x));
}

#[test]
fn test_without_macro() {
    // State
    let mut results = HashSet::new();
    let mut sum = 0;
    // Define pipeline
    let collect = Mutex::new(|x| {
        results.insert(x);
    });
    let mult = (|x| x * 2).pipe(&collect);
    let mut prod = (1..=3).pipe(&mult);
    let sum_and_pass = Mutex::new(|x| {
        sum += x;
        x
    })
    .pipe(&collect);
    let length = (|s: &str| s.len() as i32).pipe(&sum_and_pass);
    let mut prod2 = vec!["o", "yay", "ouwee"].pipe(&length);
    // Run pipeline
    let s = Scheduler::new();
    prod.add_to_scheduler(&s);
    prod2.add_to_scheduler(&s);
    s.run(4);
    // Check results
    assert_eq!(results, vec!(1, 2, 3, 4, 5, 6).into_iter().collect());
    assert_eq!(sum, 9);
}
