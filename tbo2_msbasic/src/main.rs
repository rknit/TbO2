use std::fs;

use tbo2::{
    cpu::CPU,
    mem::{RAM, ROM},
};

fn main() {
    let mut cpu = CPU::new();

    cpu.set_region(0x0000, 0x7fff, Box::new(RAM::<0x8000>::new()));

    let image = fs::read("asm/a.out").expect("temporary binary file");
    assert!(
        image.len() == 0x8000,
        "image's size is not the exact size of ROM"
    );

    let mut rom = ROM::<0x8000>::new();
    rom.load_bytes(0, &image);
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));

    cpu.reset();
    loop {
        if let Err(e) = cpu.step() {
            eprintln!("Error: {:#04x?}", e);
            break;
        }
    }
}
