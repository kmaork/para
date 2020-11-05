use para::*;

#[test]
fn test() {
    // Define pipeline
    let print = |x| println!("{}", x);
    let mult = (|x| x * 2).pipe(&print);
    let mut prod = (1..10).pipe(&mult);

    // Run pipeline
    pipeline!(4, prod);
}
