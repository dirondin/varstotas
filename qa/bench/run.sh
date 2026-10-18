#!/bin/zsh
# Path tracer benchmark: renders the demo house scenes and prints the timings.
root=${0:A:h:h:h}
out=$root/target/qa
mkdir -p $out/home
cargo build -q -p varstotas_app --manifest-path $root/Cargo.toml || exit 1
cd $out
HOME=$out/home RUST_LOG=info VARSTOTAS_SCRIPT=$root/qa/bench/pt_bench.txt timeout 900 $root/target/debug/varstotas > $out/pt_bench.log 2>&1; grep -E "path tracer|panicked|ERROR" $out/pt_bench.log | sed -E "s/^.*\] //"
