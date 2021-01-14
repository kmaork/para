use para::*;

fn main() {
    run_pipeline!(0..1000000 => |x| {x * 99;});
}