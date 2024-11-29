use std::{collections::VecDeque, fs, io::stdin, time::Duration};

use tbo2::{
    cpu::CPU,
    mem::{RAM, ROM},
};

fn main() {
    let stdin = stdin();

    let mut rom = ROM::<0x8000>::new();
    let image = fs::read("asm/a.out").expect("temporary binary file");
    assert!(
        image.len() == 0x8000,
        "image's size is not the exact size of ROM"
    );
    rom.load_bytes(0, &image);

    let mut cpu = CPU::new();
    cpu.set_region(0x0000, 0x7fff, Box::new(RAM::<0x8000>::new()));
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));

    cpu.reset();

    const CHR_DATA: u16 = 0x5000;
    const CHR_MODE: u16 = 0x5001;
    const CHR_REQ: u16 = 0x5002;

    let mut buffer = VecDeque::<char>::new();

    loop {
        if cpu.read_byte(CHR_REQ) == 1 {
            match cpu.read_byte(CHR_MODE) {
                0 => {
                    let c = cpu.read_byte(CHR_DATA) as char;
                    print!("{}", c);
                }
                1 => {
                    if buffer.is_empty() {
                        let mut s = String::new();
                        stdin.read_line(&mut s).unwrap();
                        s.chars().for_each(|mut c| {
                            if c == '\n' {
                                c = '\r';
                            }
                            buffer.push_back(c)
                        });
                    }

                    cpu.write_byte(CHR_DATA, buffer.pop_front().unwrap() as u8);
                }
                _ => unimplemented!(),
            }
            cpu.write_byte(CHR_REQ, 0);
        }

        if let Err(e) = cpu.step() {
            eprintln!("Error: {:#04x?}", e);
            break;
        }

        std::thread::sleep(Duration::from_micros(100));
    }
}
