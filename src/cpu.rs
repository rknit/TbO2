use crate::{
    inst::{decode_inst, AddressingMode, Inst},
    layout::Layout,
    mem::Memory,
};

#[derive(Debug)]
pub struct TbO2 {
    pc: u16,
    sp: u8,
    a: Register,
    x: Register,
    y: Register,
    status: Status,
    layout: Layout,
}
impl TbO2 {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: Default::default(),
            x: Default::default(),
            y: Default::default(),
            status: Status::default(),
            layout: Layout::new(u16::max_value() as usize + 1),
        }
    }

    pub fn set_region(&mut self, addr_start: usize, addr_end: usize, mem: Box<dyn Memory>) {
        self.layout.set_region(addr_start, addr_end, mem);
    }

    pub fn reset(&mut self) {
        self.layout.validate();

        self.status = Status::default();
        self.a = Default::default();
        self.x = Default::default();
        self.y = Default::default();

        self.sp = 0;
        self.pc = self.read_word(0xFFFC);
        println!("starting execution at {:#04x}...", self.pc);
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        let inst_byte = self.next_byte();

        let Some((inst, addr_mode)) = decode_inst(inst_byte) else {
            return Err(ExecutionError::UnknownInst(inst_byte));
        };

        match inst {
            Inst::LDA => {
                self.a.data = self.read_byte_addressed(addr_mode);
                self.check_nz(self.a);
            }
            Inst::LDX => {
                self.x.data = self.read_byte_addressed(addr_mode);
                self.check_nz(self.x);
            }
            Inst::LDY => {
                self.y.data = self.read_byte_addressed(addr_mode);
                self.check_nz(self.y);
            }

            Inst::STA => self.write_byte_addressed(self.a.data, addr_mode),
            Inst::STX => self.write_byte_addressed(self.x.data, addr_mode),
            Inst::STY => self.write_byte_addressed(self.y.data, addr_mode),

            Inst::TAX => {
                self.x = self.a;
                self.check_nz(self.x);
            }
            Inst::TAY => {
                self.y = self.a;
                self.check_nz(self.y);
            }
            Inst::TSX => {
                self.x.data = self.sp;
                self.check_nz(self.x);
            }
            Inst::TXA => {
                self.a = self.x;
                self.check_nz(self.a);
            }
            Inst::TXS => {
                self.sp = self.x.data;
            }
            Inst::TYA => {
                self.a = self.y;
                self.check_nz(self.a);
            }
        };

        Ok(())
    }

    fn check_nz(&mut self, reg: Register) {
        self.status.negative = reg.is_negative();
        self.status.zero = reg.is_zero();
    }

    fn read_byte_addressed(&mut self, addr_mode: AddressingMode) -> u8 {
        match addr_mode {
            AddressingMode::Implied => unimplemented!("Implied addressing mode"),
            AddressingMode::Immediate => self.next_byte(),
            AddressingMode::Absolute => {
                let addr = self.next_word();
                self.read_byte(addr)
            }
            AddressingMode::AbsoluteX => {
                let addr = self.next_word() + self.x.data as u16;
                self.read_byte(addr)
            }
            AddressingMode::AbsoluteY => {
                let addr = self.next_word() + self.y.data as u16;
                self.read_byte(addr)
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let indexed = self.next_byte() + self.x.data;
                let addr = self.read_word(indexed as u16);
                self.read_byte(addr)
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte() as u16;
                let indexed = self.read_word(zp_addr) + self.y.data as u16;
                self.read_byte(indexed)
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let zp_addr = self.next_byte() as u16;
                self.read_byte(zp_addr)
            }
            AddressingMode::ZeroPageX => {
                let indexed = self.next_byte() + self.x.data;
                self.read_byte(indexed as u16)
            }
            AddressingMode::ZeroPageY => {
                let indexed = self.next_byte() + self.y.data;
                self.read_byte(indexed as u16)
            }
        }
    }

    fn write_byte_addressed(&mut self, byte: u8, addr_mode: AddressingMode) {
        match addr_mode {
            AddressingMode::Implied => unimplemented!("Implied addressing mode"),
            AddressingMode::Immediate => unimplemented!("Immediate addressing mode"),
            AddressingMode::Absolute => {
                let addr = self.next_word();
                self.write_byte(addr, byte);
            }
            AddressingMode::AbsoluteX => {
                let addr = self.next_word() + self.x.data as u16;
                self.write_byte(addr, byte);
            }
            AddressingMode::AbsoluteY => {
                let addr = self.next_word() + self.y.data as u16;
                self.write_byte(addr, byte);
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let indexed = self.next_byte() + self.x.data;
                let addr = self.read_word(indexed as u16);
                self.write_byte(addr, byte);
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte() as u16;
                let indexed = self.read_word(zp_addr) + self.y.data as u16;
                self.write_byte(indexed, byte);
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let zp_addr = self.next_byte() as u16;
                self.write_byte(zp_addr, byte);
            }
            AddressingMode::ZeroPageX => {
                let indexed = self.next_byte() + self.x.data;
                self.write_byte(indexed as u16, byte);
            }
            AddressingMode::ZeroPageY => {
                let indexed = self.next_byte() + self.y.data;
                self.write_byte(indexed as u16, byte);
            }
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn next_word(&mut self) -> u16 {
        let word = self.read_word(self.pc);
        self.pc += 2;
        word
    }

    fn read_byte(&self, addr: u16) -> u8 {
        self.layout.read_byte(addr as usize)
    }

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.layout.read_byte(addr as usize) as u16;
        let hi = self.layout.read_byte(addr as usize + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        self.layout.write_byte(addr as usize, data);
    }

    fn write_word(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = ((data >> 8) & 0xFF) as u8;
        self.layout.write_byte(addr as usize, lo);
        self.layout.write_byte(addr as usize + 1, hi);
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    UnknownInst(u8),
}

#[derive(Debug, Default)]
struct Status {
    negative: bool,
    overflow: bool,
    break_: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct Register {
    data: u8,
}
impl Register {
    pub fn is_negative(&self) -> bool {
        (self.data & 0b10000000) > 0
    }

    pub fn is_zero(&self) -> bool {
        self.data == 0
    }
}
impl From<u8> for Register {
    fn from(value: u8) -> Self {
        Self { data: value }
    }
}
