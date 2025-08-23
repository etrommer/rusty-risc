#!/bin/bash

cargo build -r

failures=0

for test_binary in tests/rv32*; do
  printf "%-45s" "Running test: $test_binary..."
  if target/release/riscv_emu --elf "$test_binary" > /dev/null 2>&1; then
    echo "[ OK ]"
  else
    echo "[FAIL]"
    failures=$((failures + 1))
  fi
done

echo
if [ $failures -eq 0 ]; then
  echo "All tests passed."
  exit 0
else
  echo "$failures tests failed."
  exit 1
fi
