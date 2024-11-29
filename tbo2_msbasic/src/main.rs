use std::{
    fs,
    io::{stdout, Write},
};

use tbo2::{
    cpu::CPU,
    mem::{RAM, ROM},
};
use termion::{async_stdin, event::Key, input::TermRead, raw::IntoRawMode};

fn main() {
    let mut stdin = async_stdin().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();
    stdout.flush().unwrap();

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

    loop {
        const CHR_DATA: u16 = 0x5000;
        const CHR_MODE: u16 = 0x5001;
        const CHR_REQ: u16 = 0x5002;

        if cpu.read_byte(CHR_REQ) == 1 {
            let mode = cpu.read_byte(CHR_MODE);
            match mode {
                0 => {
                    let mut c = (cpu.read_byte(CHR_DATA) as char).to_string();
                    if c == "\r" {
                        c += "\n";
                    }
                    write!(stdout, "{}", c).unwrap();
                    stdout.lock().flush().unwrap();
                    cpu.write_byte(CHR_REQ, 0);
                }
                1 => {
                    if let Some(Ok(key)) = stdin.next() {
                        match key {
                            Key::Char(mut c) => {
                                if c == '\n' {
                                    c = '\r';
                                }
                                cpu.write_byte(CHR_DATA, c as u8);
                                cpu.write_byte(CHR_REQ, 0);
                            }
                            Key::Backspace => {
                                cpu.write_byte(CHR_DATA, 0x8);
                                cpu.write_byte(CHR_REQ, 0);
                            }
                            Key::Ctrl('d' | 'c') => break,
                            _ => (),
                        };
                    }
                }
                _ => panic!("invalid CHR_MODE"),
            };
        }

        if let Err(e) = cpu.step() {
            eprintln!("Error: {:#04x?}", e);
            break;
        }
    }
}
