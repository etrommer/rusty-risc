use clap::Parser;
use std::{fs, vec};

use cpu::Cpu;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod bus;
mod cpu;
mod trap;

fn parse_level(s: &str) -> Result<Level, String> {
    s.parse::<Level>().map_err(|_| {
        format!(
            "'{}' is not a valid log level. Possible values are: error, warn, info, debug, trace.",
            s
        )
    })
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    kernel: Option<String>,

    #[arg(short, long)]
    elf: Option<String>,

    #[arg(long)]
    dtb: Option<String>,

    #[arg(short, long, default_value_t = 0)]
    delay: u64,

    #[arg(short, long, default_value_t = 0)]
    instructions: u64,

    #[arg(short, long, default_value_t = false)]
    test: bool,

    #[arg(long, default_value_t = Level::INFO, value_parser = parse_level)]
    log_level: Level,
}

const RAM_SIZE: usize = 64 * 1024 * 1024; // 256 KiB

fn load_from_bin(bin_path: &String) -> Vec<u8> {
    fs::read(bin_path).unwrap()
}

fn main() {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("msg: Failed to set global subscriber");

    let mut kernel = vec![];

    if let Some(kernel_path) = args.kernel {
        kernel = load_from_bin(&kernel_path);
    }

    let mut cpu = Cpu::new(kernel, RAM_SIZE);
    cpu.delay = args.delay;
    cpu.instruction_count = args.instructions;
    cpu.test = args.test;

    if let Some(elf_path) = args.elf {
        cpu.load_elf(load_from_bin(&elf_path));
    }
    if let Some(dtb_path) = args.dtb {
        cpu.load_dtb(load_from_bin(&dtb_path));
    }

    loop {
        let _ = cpu.step();
    }
}
