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
    schedule(&mut [&mut prod, &mut prod2], 4);
    // Check results
    assert_eq!(results, vec!(1, 2, 3, 4, 5, 6).into_iter().collect());
    assert_eq!(sum, 9);
}

#[test]
fn test_with_fanout() {
    // State
    let mut sum = 0;
    // Define pipeline
    let add = Mutex::new(|x| {
        sum += x;
    });
    let plus = (|x| x + 2).pipe(&add);
    let minus = (|x| x - 1).pipe(&add);
    let fanout = Fanout::new(vec![&plus, &minus]);
    let nums = vec![1, 2, 3];
    let mut prod = nums.iter().pipe(&fanout);
    // Run pipeline
    schedule(&mut [&mut prod], 4);
    // Check results
    let numsum = nums.iter().sum::<i32>();
    assert_eq!(sum, numsum * 2 + nums.len() as i32);
}

#[test]
fn test_fanout_and_fanin_with_macro() {
    // State
    let mut sum = 0;
    // Define pipeline
    let add = Mutex::new(|x| {
        sum += x;
    });
    let plus = (|x| x + 2).pipe(&add);
    let minus = (|x| x - 1).pipe(&add);
    let nums = vec![1, 2, 3];
    // Run pipeline
    run_pipeline!(&nums => Fanout::new(vec![&plus, &minus]));
    // Check results
    let numsum = nums.iter().sum::<i32>();
    assert_eq!(sum, numsum * 2 + nums.len() as i32);
}
