#![allow(unused_must_use)]

use para::*;
use std::sync::Mutex;

fn main() {
    // let mut v = vec![];
    // run_pipeline!(0..10000000 => |x| x + 1 => Mutex::new(|x| {v.push(x * 99);}));
    loop {
        let f = Fanout::new(vec![&|x| {}, &|x| {}]);
        run_pipeline!(0..1000000 => f);
    }
}
