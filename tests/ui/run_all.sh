#!/bin/zsh
# Regression run of the UI scripts in tests/ui/scripts (real-input `raw`/`rawp` commands + `expect` checks).
# usage: tests/ui/run_all.sh [name ...]   — debug build; output in target/ui-tests/<name>.log, screenshots in target/ui-tests/shots
# tests/ui/fixtures/<name>/components, when present, is the components library the script starts with.
root=${0:A:h:h:h}
out=$root/target/ui-tests
mkdir -p $out/shots $out/home $out/models
cargo build -q -p varstotas_app --manifest-path $root/Cargo.toml || exit 1
names=($@)
(( $#names )) || names=($root/tests/ui/scripts/*.txt(:t:r))
fail=0
for n in $names; do
  cd $out
  rm -rf $out/home/.varstotas/cache/thumbs $out/home/.varstotas/components $out/home/.varstotas/materials $out/home/.varstotas/templates $out/home/.varstotas/library_state.json $out/home/.varstotas/downloads/manual   # the libraries (favourites, history) start empty in every script
  [[ -d $root/tests/ui/fixtures/$n/components ]] && cp -R $root/tests/ui/fixtures/$n/components $out/home/.varstotas/   # or with the script's fixture
  # Explicit repository fixtures for regression tests, never installed by the application.
  [[ $n == templates ]] && rm -rf $out/models/custom_templates
  # printing_all hands its export to this stub slicer, which records the file it received.
  [[ $n == printing_all ]] && printf '#!/bin/sh\n[ "$#" -eq 1 ] || exit 21\nprintf "%%s\\n" "$1" >> %s/slicer-received.txt\n' $out > $out/fake-print-slicer && chmod 700 $out/fake-print-slicer
  qa_home=$out/home
  qa_examples=$root/examples
  if [[ $n == no_examples ]]; then
    qa_home=$out/no_examples_home
    qa_examples=
    mkdir -p $qa_home
  else
    mkdir -p $qa_home/.varstotas/plugins/examples
    cp $root/examples/plugins/* $qa_home/.varstotas/plugins/examples/
  fi
  HOME=$qa_home VARSTOTAS_LANG=en RUST_LOG=info VARSTOTAS_QA_EXAMPLES=$qa_examples VARSTOTAS_SCRIPT=$root/tests/ui/scripts/$n.txt timeout 300 $root/target/debug/varstotas > $out/$n.log 2>&1
  code=$?
  bad=$(grep -cE 'expect FAIL|panicked' $out/$n.log)
  ok=$(grep -c 'expect ok' $out/$n.log)
  if [[ $n == no_examples && ( -e $qa_home/.varstotas/examples || -e $qa_home/.varstotas/plugins/examples ) ]]; then
    echo "FAIL no_examples: application installed example files"
    bad=$((bad + 1))
  fi
  if (( code != 0 || bad > 0 )); then
    echo "FAIL $n (exit $code, $bad failures, $ok ok)"; grep -E 'expect FAIL|panicked' $out/$n.log | cut -c1-300; fail=1
    cp $out/$n.log $out/$n.fail.log; echo "     log: $out/$n.fail.log (last: $(grep 'script:' $out/$n.log | tail -1 | sed -E 's/.*script: //' | cut -c1-80))"
  else
    echo "ok   $n ($ok checks)"
  fi
done
exit $fail
