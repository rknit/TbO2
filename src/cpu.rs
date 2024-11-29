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

        self.sp = 0xFF;
        println!("Setting stack pointer to {:#06x}...", self.get_sp());
        self.pc = self.read_word(0xFFFC);
        println!("Starting execution at {:#06x}...", self.pc);
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        let inst_byte = self.next_byte();

        let Some((inst, addr_mode)) = decode_inst(inst_byte) else {
            return Err(ExecutionError::UnknownInst(inst_byte));
        };

        match inst {
            Inst::LDA => {
                self.a.data = self.read_byte_addressed(addr_mode).1;
                self.check_nz(self.a);
            }
            Inst::LDX => {
                self.x.data = self.read_byte_addressed(addr_mode).1;
                self.check_nz(self.x);
            }
            Inst::LDY => {
                self.y.data = self.read_byte_addressed(addr_mode).1;
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

            Inst::PHA => self.push_byte(self.a.data),
            Inst::PHP => self.push_byte(self.status.into()),
            Inst::PLA => {
                self.a.data = self.pull_byte();
                self.check_nz(self.a);
            }
            Inst::PLP => {
                self.status = Status::from(self.pull_byte());
            }

            Inst::DEC => {
                let (addr, mut data) = self.read_byte_addressed(addr_mode);
                data -= 1;
                self.write_byte(addr, data);
                self.check_nz(Register { data });
            }
            Inst::DEX => {
                self.x.data -= 1;
                self.check_nz(self.x);
            }
            Inst::DEY => {
                self.y.data -= 1;
                self.check_nz(self.y);
            }
            Inst::INC => {
                let (addr, mut data) = self.read_byte_addressed(addr_mode);
                data += 1;
                self.write_byte(addr, data);
                self.check_nz(Register { data });
            }
            Inst::INX => {
                self.x.data += 1;
                self.check_nz(self.x);
            }
            Inst::INY => {
                self.y.data += 1;
                self.check_nz(self.y);
            }

            Inst::ADC => {
                let operand = self.read_byte_addressed(addr_mode).1 as u16;
                let carry = self.status.carry as u16;
                let result = self.a.data as u16 + operand + carry;

                self.status.carry = result > 0xFF;
                self.status.overflow =
                    (self.a.data & operand as u8 & (self.a.data ^ result as u8) & 0x80) > 0;
                self.a.data = result as u8;
                self.check_nz(self.a);
            }
            Inst::SBC => {
                let operand = self.read_byte_addressed(addr_mode).1 as u16;
                let operand = !operand; // invert operand to get -operand - 1, then we can use adc
                let carry = self.status.carry as u16;
                let result = (self.a.data as u16)
                    .wrapping_add(operand)
                    .wrapping_add(carry);

                self.status.carry = result > 0xFF;
                self.status.overflow =
                    (self.a.data & operand as u8 & (self.a.data ^ result as u8) & 0x80) > 0;
                self.a.data = result as u8;
                self.check_nz(self.a);
            }

            Inst::AND => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data &= data;
                self.check_nz(self.a);
            }
            Inst::EOR => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data ^= data;
                self.check_nz(self.a);
            }
            Inst::ORA => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data |= data;
                self.check_nz(self.a);
            }

            Inst::ASL => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    self.a.data = data;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    self.write_byte(read.0, data);
                }
                self.check_nz(Register { data });
                self.status.carry = send_carry;
            }
            Inst::LSR => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    self.a.data = data;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    self.write_byte(read.0, data);
                };
                self.check_nz(Register { data });
                self.status.carry = send_carry;
            }
            Inst::ROL => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    data |= self.status.carry as u8;
                    self.a.data = data;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    data |= self.status.carry as u8;
                    self.write_byte(read.0, data);
                };
                self.check_nz(Register { data });
                self.status.carry = send_carry;
            }
            Inst::ROR => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    data |= (self.status.carry as u8) << 7;
                    self.a.data = data;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    data |= (self.status.carry as u8) << 7;
                    self.write_byte(read.0, data);
                };
                self.check_nz(Register { data });
                self.status.carry = send_carry;
            }

            Inst::CLC => self.status.carry = false,
            Inst::CLD => self.status.decimal = false,
            Inst::CLI => self.status.interrupt = false,
            Inst::CLV => self.status.overflow = false,
            Inst::SEC => self.status.carry = true,
            Inst::SED => self.status.decimal = true,
            Inst::SEI => self.status.interrupt = true,
        };

        Ok(())
    }

    fn check_nz(&mut self, reg: Register) {
        self.status.negative = reg.is_negative();
        self.status.zero = reg.is_zero();
    }

    fn push_byte(&mut self, data: u8) {
        self.write_byte(self.get_sp(), data);
        self.sp -= 1;
    }

    fn pull_byte(&mut self) -> u8 {
        self.sp += 1;
        self.read_byte(self.get_sp())
    }

    fn get_sp(&self) -> u16 {
        self.sp as u16 + 0x100
    }

    fn read_byte_addressed(&mut self, addr_mode: AddressingMode) -> (u16, u8) {
        match addr_mode {
            AddressingMode::Implied => unimplemented!("Implied addressing mode"),
            AddressingMode::Immediate => (self.pc, self.next_byte()),
            AddressingMode::Absolute => {
                let addr = self.next_word();
                (addr, self.read_byte(addr))
            }
            AddressingMode::AbsoluteX => {
                let addr = self.next_word() + self.x.data as u16;
                (addr, self.read_byte(addr))
            }
            AddressingMode::AbsoluteY => {
                let addr = self.next_word() + self.y.data as u16;
                (addr, self.read_byte(addr))
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let indexed = self.next_byte() + self.x.data;
                let addr = self.read_word(indexed as u16);
                (addr, self.read_byte(addr))
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte() as u16;
                let addr = self.read_word(zp_addr) + self.y.data as u16;
                (addr, self.read_byte(addr))
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let addr = self.next_byte() as u16;
                (addr, self.read_byte(addr))
            }
            AddressingMode::ZeroPageX => {
                let addr = (self.next_byte() + self.x.data) as u16;
                (addr, self.read_byte(addr))
            }
            AddressingMode::ZeroPageY => {
                let addr = (self.next_byte() + self.y.data) as u16;
                (addr, self.read_byte(addr))
            }
        }
    }

    fn write_byte_addressed(&mut self, data: u8, addr_mode: AddressingMode) {
        match addr_mode {
            AddressingMode::Implied => unimplemented!("Implied addressing mode"),
            AddressingMode::Immediate => unimplemented!("Immediate addressing mode"),
            AddressingMode::Absolute => {
                let addr = self.next_word();
                self.write_byte(addr, data);
            }
            AddressingMode::AbsoluteX => {
                let addr = self.next_word() + self.x.data as u16;
                self.write_byte(addr, data);
            }
            AddressingMode::AbsoluteY => {
                let addr = self.next_word() + self.y.data as u16;
                self.write_byte(addr, data);
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let indexed = self.next_byte() + self.x.data;
                let addr = self.read_word(indexed as u16);
                self.write_byte(addr, data);
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte() as u16;
                let indexed = self.read_word(zp_addr) + self.y.data as u16;
                self.write_byte(indexed, data);
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let zp_addr = self.next_byte() as u16;
                self.write_byte(zp_addr, data);
            }
            AddressingMode::ZeroPageX => {
                let indexed = self.next_byte() + self.x.data;
                self.write_byte(indexed as u16, data);
            }
            AddressingMode::ZeroPageY => {
                let indexed = self.next_byte() + self.y.data;
                self.write_byte(indexed as u16, data);
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

#[derive(Debug, Default, Clone, Copy)]
struct Status {
    negative: bool,
    overflow: bool,
    break_: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}
impl Into<u8> for Status {
    fn into(self) -> u8 {
        (self.negative as u8) << 7
            | (self.overflow as u8) << 6
            | (1 << 5)
            | (1 << 4)
            | (self.decimal as u8) << 3
            | (self.interrupt as u8) << 2
            | (self.zero as u8) << 1
            | (self.carry as u8)
    }
}
impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self {
            negative: (value & 0b10000000) > 0,
            overflow: (value & 0b1000000) > 0,
            break_: false,
            decimal: (value & 0b1000) > 0,
            interrupt: (value & 0b100) > 0,
            zero: (value & 0b10) > 0,
            carry: (value & 0b1) > 0,
        }
    }
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
