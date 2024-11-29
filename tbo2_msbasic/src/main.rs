use std::{
    fs,
    io::{self, Stdout, Write},
    time::{Duration, Instant},
};

use tbo2::{
    cpu::CPU,
    mem::{RAM, ROM},
};
use termion::{
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    AsyncReader,
};

fn main() {
    const CLOCK_PERIOD_NANOS: u64 = 71; // 14 Mhz

    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut keys = termion::async_stdin().keys();

    let mut cpu = CPU::new();
    setup_mem(&mut cpu);
    cpu.reset();

    const CHR_DATA: u16 = 0x5000;
    const CHR_STATUS: u16 = 0x5001;

    loop {
        let timer_start = Instant::now();

        if let Some(c) = get_char(&mut keys) {
            if c == 0x4 as char {
                break;
            }

            cpu.write_byte(CHR_DATA, c as u8);
            cpu.irq();
        }

        if cpu.read_byte(CHR_STATUS) == 1 {
            let c = cpu.read_byte(CHR_DATA);
            print_char(&mut stdout, c as char);
            cpu.write_byte(CHR_STATUS, 0);
        }

        if let Err(e) = cpu.step() {
            write!(stdout, "Error: {:#04x?}\r\n", e).unwrap();
            stdout.flush().unwrap();
            break;
        }

        while Instant::now().duration_since(timer_start) < Duration::from_nanos(CLOCK_PERIOD_NANOS)
        {
            continue;
        }
    }
}

fn print_char(stdout: &mut RawTerminal<Stdout>, c: char) {
    if c == '\n' {
        return;
    }
    write!(stdout, "{}", c).unwrap();
    if c == '\r' {
        write!(stdout, "\n").unwrap();
    }
    stdout.flush().unwrap();
}

fn get_char(keys: &mut Keys<AsyncReader>) -> Option<char> {
    let Some(Ok(key)) = keys.next() else {
        return None;
    };
    use termion::event::Key::*;
    Some(match key {
        Backspace => 0x8 as char,
        Delete => 0x7F as char,
        Char(c) => match c {
            '\n' => '\r',
            _ => c,
        },
        Ctrl(c) => match c {
            'd' => 0x4 as char,
            'c' => 0x3 as char,
            _ => return None,
        },
        Esc => 0x1B as char,
        _ => return None,
    })
}

fn setup_mem(cpu: &mut CPU) {
    let mut rom = ROM::<0x8000>::new();
    let image = fs::read("asm/bios.bin").expect("temporary binary file");
    assert!(
        image.len() == 0x8000,
        "image's size is not the exact size of ROM"
    );
    rom.load_bytes(0, &image);

    cpu.set_region(0x0000, 0x7FFF, Box::new(RAM::<0x8000>::new()));
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));
}
