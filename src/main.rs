use clap::Parser;
use std::{fs, vec};

use bus::Bus;
use cpu::Cpu;
use goblin::elf::Elf;
use std::path::Path;

mod bus;
mod cpu;
mod trap;

const RAM_SIZE: usize = 256 * 1024; // 256 KiB

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    kernel: Option<String>,

    #[arg(short, long)]
    elf: Option<String>,

    #[arg(short, long, default_value_t = 0)]
    delay: u64,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn load_from_bin(bin_path: &String) -> Vec<u8> {
    let mut ram = fs::read(bin_path).unwrap();
    if ram.len() > RAM_SIZE {
        panic!("Binary size exceeds RAM size");
    }
    ram.extend(vec![0u8; RAM_SIZE - ram.len()]);
    ram
}

fn load_from_elf(elf_path: &String) -> Vec<u8> {
    let elf_bytes = fs::read(elf_path).unwrap();
    let elf = Elf::parse(&elf_bytes).unwrap();
    let mut ram = vec![0u8; RAM_SIZE];
    // Find all sections starting with .text
    for section in elf.section_headers.iter() {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            if name.starts_with(".text") {
                let offset = section.sh_offset as usize;
                let size = section.sh_size as usize;
                let addr = section.sh_addr as usize;
                let text_bytes = &elf_bytes[offset..offset + size];
                println!(
                    "Loading {} section at {:#08x} with size {}",
                    name, addr, size
                );
                ram[addr - 0x80000000..addr - 0x80000000 + size].copy_from_slice(text_bytes);
            }
        }
    }
    ram
}

fn main() {
    let args = Args::parse();
    let ram = if let Some(elf_path) = args.elf {
        load_from_elf(&elf_path)
    } else if let Some(bin_path) = args.kernel {
        load_from_bin(&bin_path)
    } else {
        panic!("Either --kernel or --elf must be specified");
    };

    let mut cpu = Cpu::new(ram);
    cpu.delay = args.delay;

    loop {
        let _ = cpu.step();
    }
}
