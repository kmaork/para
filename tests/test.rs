use para::*;
use std::collections::HashSet;
use std::sync::Mutex;

#[test]
fn test() {
    let mut results = HashSet::new();
    // Define pipeline
    let collect = Mutex::new(|x| { results.insert(x); });
    let mult = (|x| x * 2).pipe(&collect);
    let mut prod = (1..=3).pipe(&mult);
    let length = (|s: &str| s.len() as i32).pipe(&collect);
    let mut prod2 = vec!("o", "yay", "ouwee").pipe(&length);
    // Run pipeline
    pipeline!(4, prod, prod2);
    // Check results
    assert_eq!(results, vec!(1, 2, 3, 4, 5, 6).into_iter().collect());
}
