use tbo2::{
    cpu::CPU,
    mem::{RAM, ROM},
};

fn main() {
    let mut cpu = CPU::new();

    cpu.set_region(0x0000, 0x7fff, Box::new(RAM::<0x8000>::new()));

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
