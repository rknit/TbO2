use std::{
    collections::VecDeque,
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

    env_logger::builder().format_timestamp(None).init();

    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut keys = termion::async_stdin().keys();

    let mut cpu = CPU::new();
    setup_mem(&mut cpu);
    cpu.reset();

    const CHR_IN: u16 = 0x5000;
    const CHR_CTS: u16 = 0x5001;
    const CHR_OUT: u16 = 0x5002;
    const CHR_CTR: u16 = 0x5003;

    let mut inputs = VecDeque::new();

    loop {
        let timer_start = Instant::now();

        if let Some(c) = get_char(&mut keys) {
            if c == 0x4 as char {
                break;
            }
            if c == 'p' {
                println!("{}\r", cpu.trace_exec());
            }
            inputs.push_back(c);
        }

        if !inputs.is_empty() && cpu.is_irq_enabled() && cpu.read_byte(CHR_CTS) > 0 {
            let c = inputs.pop_front().unwrap();
            cpu.write_byte(CHR_IN, c as u8);
            cpu.irq();
        }

        if cpu.read_byte(CHR_CTR) == 1 {
            let c = cpu.read_byte(CHR_OUT);
            print_char(&mut stdout, c as char);
            cpu.write_byte(CHR_CTR, 0);
        }

        if let Err(e) = cpu.step() {
            write!(stdout, "\r\nError: {:0x?} at {:#04x}\r\n", e, cpu.get_pc()).unwrap();
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
    write!(stdout, "{}", c).unwrap();
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
    let image = fs::read("tbo2.bin").expect("\r\ntemporary binary file\r\n");
    assert!(
        image.len() == 0x8000,
        "\r\nimage's size is not the exact size of ROM\r\n"
    );
    //let image = [0; 0x8000];
    rom.load_bytes(0, &image);

    cpu.set_region(0x0000, 0x7FFF, Box::new(RAM::<0x8000>::new()));
    cpu.set_region(0x8000, 0xFFFF, Box::new(rom));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn klaus_dormann_test_suite() {
        if let Err(e) = env_logger::builder().format_timestamp(None).try_init() {
            panic!("{}", e);
        }

        const CLOCK_PERIOD_NANOS: u64 = 71;

        let mut cpu = CPU::new();

        let image = fs::read("6502_65C02_functional_tests/ca65/6502_functional_test.bin")
            .expect("test binary file");

        let (ram_part, rom_part) = image.split_at(0x8000);

        let mut ram = RAM::<0x8000>::new();
        ram.load_bytes(0, ram_part);

        let mut rom = ROM::<0x8000>::new();
        rom.load_bytes(0, rom_part);

        cpu.set_region(0x0000, 0x7FFF, Box::new(ram));
        cpu.set_region(0x8000, 0xFFFF, Box::new(rom));

        cpu.reset();
        cpu.set_pc(0x400);

        let mut prev_pc: i32 = -1;
        loop {
            let timer_start = Instant::now();

            if let Err(e) = cpu.step() {
                panic!("Error: {:0x?} at {:#04x}", e, cpu.get_pc());
            }

            if cpu.get_pc() as i32 == prev_pc {
                if cpu.get_pc() == 0x3699 {
                    break;
                }
                panic!("trapped");
            }
            prev_pc = cpu.get_pc() as i32;

            while Instant::now().duration_since(timer_start)
                < Duration::from_nanos(CLOCK_PERIOD_NANOS)
            {
                continue;
            }
        }
    }
}
