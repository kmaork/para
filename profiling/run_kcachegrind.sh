#!/bin/bash
cargo build --bin profile --profile bench -Z unstable-options
bin_path=$(dirname `cargo locate-project --message-format plain`)
out_path=`date +%y_%m_%d-%H_%M_%S.callgrind`
valgrind --tool=callgrind --callgrind-out-file=$out_path $bin_path/target/debug/profile
kcachegrind $out_path
# http://www.codeofview.com/fix-rs/2017/01/24/how-to-optimize-rust-programs-on-linux/
