use cpu::TbO2;
use mem::{RAM, ROM};

#[allow(dead_code)]
pub mod cpu;
pub mod inst;
pub mod layout;
pub mod mem;

fn main() {
    let mut cpu = TbO2::new();

    let ram = RAM::<0x8000>::new();
    cpu.set_region(0x0000, 0x7FFF, Box::new(ram));

    let rom = ROM::<0x8000>::new();
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));

    cpu.reset();

    loop {
        if let Err(e) = cpu.step() {
            eprintln!("Error: {:?}", e);
            break;
        }
    }
}
