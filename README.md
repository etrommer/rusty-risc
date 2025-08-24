# RISC-V Emulator

Purely educational implementation of a RISC-V CPU in Rust. The purpose was for me to learn about the RISC-V instruction set and Embedded Linux.

The core implements:
- **RV32I** base ISA
- **M** Extension
- **A** Extension
- **Zicsr** Extension
- **Machine/User** modes
- **CLINT** Interrupt Controller
- **8205 UART**

The aim for the core was to do two things:
1. Run a simple embedded C baremetal program
2. Boot an embedded `nommu` Linux

For the second part especially, the implementation borrows heavily from cnlohr's [mini-rv32ima](https://github.com/cnlohr/mini-rv32ima), which I used as a reference for the necessary components, and which was unbelievably helpful during debugging.

## Baremetal Demo
```bash
make -C baremetal
cargo run -- --elf baremetal/kernel.elf -d 50 --log-level DEBUG
```

## RISC-V Test Suite
The relevant test cases are pre-compiled in the `tests` folder and can be run with:
```bash
./run_tests.sh
```
The core supports the `-t/--test` flag to exit the program when an `ECALL` instruction from the riscv-tests test suite is detected with the test result as the exit code.

To compile the riscv-tests yourself:
- TODO

## Embedded Linux
```bash
cargo run -r -- -k mini-rv32ima/mini-rv32ima/DownloadedImage --dtb mini-rv32ima/mini-rv32ima/sixtyfourmb.dtb --log-level WARN
```