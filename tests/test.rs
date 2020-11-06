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
    // Run pipeline
    pipeline!(4, prod);
    // Check results
    assert_eq!(results, vec!(2, 4, 6).into_iter().collect());
}
