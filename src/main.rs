use cpu::TbO2;
use mem::ROM;

#[allow(dead_code)]
pub mod cpu;
pub mod mem;

fn main() {
    let mut cpu = TbO2::new();
    cpu.set_region(0x8000, 0xFFFF, Box::new(ROM::<0x8000>::new()));

    cpu.reset();
}
