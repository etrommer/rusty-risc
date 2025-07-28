use clap::Parser;
use std::{fs, vec};

use bus::Bus;
use cpu::Cpu;

mod bus;
mod cpu;
mod trap;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    kernel: String,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}
fn main() {
    let args = Args::parse();
    let mut ram = fs::read(args.kernel).unwrap();
    ram.extend(vec![0u8; 256 * 1024 - ram.len()]);

    let bus = Bus::new(ram);
    let mut cpu = Cpu::new(bus);

    loop {
        let _ = cpu.step();
    }
}
