#!/bin/zsh
# Regression run of the UI scripts in qa/scripts (real-input `raw`/`rawp` commands + `expect` checks).
# usage: qa/run_all.sh [name ...]   — debug build; output in target/qa/<name>.log, screenshots in target/qa/shots
root=${0:A:h:h}
out=$root/target/qa
mkdir -p $out/shots $out/home $out/models
cargo build -q -p varstotas_app --manifest-path $root/Cargo.toml || exit 1
names=($@)
(( $#names )) || names=($root/qa/scripts/*.txt(:t:r))
fail=0
for n in $names; do
  cd $out
  HOME=$out/home RUST_LOG=info VARSTOTAS_SCRIPT=$root/qa/scripts/$n.txt timeout 300 $root/target/debug/varstotas > $out/$n.log 2>&1
  code=$?
  bad=$(grep -cE 'expect FAIL|panicked' $out/$n.log)
  ok=$(grep -c 'expect ok' $out/$n.log)
  if (( code != 0 || bad > 0 )); then
    echo "FAIL $n (exit $code, $bad failures, $ok ok)"; grep -E 'expect FAIL|panicked' $out/$n.log | cut -c1-300; fail=1
  else
    echo "ok   $n ($ok checks)"
  fi
done
exit $fail
