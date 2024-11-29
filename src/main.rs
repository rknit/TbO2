use cpu::TbO2;
use mem::{RAM, ROM};

#[allow(dead_code)]
pub mod cpu;
pub mod inst;
pub mod layout;
pub mod mem;

fn main() {
    let mut cpu = TbO2::new();

    let mut ram = RAM::<0x8000>::new();
    ram.load_bytes(0x0, &[0xA9, 12, 0xE9, 9, 0x85, 0x06]);
    cpu.set_region(0x0000, 0x7FFF, Box::new(ram));

    let mut rom = ROM::<0x8000>::new();
    rom.load_bytes(0xFFFC - 0x8000, &[0x00, 0x00]);
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));

    cpu.reset();

    loop {
        if let Err(e) = cpu.step() {
            eprintln!("Error: {:?}", e);
            break;
        }
    }
}
