use core::fmt;

use log::{log_enabled, trace, Level};

use crate::{
    inst::{decode_inst, AddressingMode, Inst},
    Device, Layout,
};

pub struct CPU {
    pc: u16,
    sp: u8,
    a: Register,
    x: Register,
    y: Register,
    status: Status,
    layout: Layout,

    debug_inst: Inst,
    debug_pc: u16,
    debug_operand: DebugOp,
    debug_desc: DebugDesc,
}
impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CPU")
            .field("pc", &self.pc)
            .field("sp", &self.sp)
            .field("a", &self.a)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("status", &self.status)
            // .field("layout", &self.layout)
            // .field("debug_inst", &self.debug_inst)
            // .field("debug_pc", &self.debug_pc)
            // .field("debug_operand", &self.debug_operand)
            // .field("debug_desc", &self.debug_desc)
            .finish()
    }
}
impl Drop for CPU {
    fn drop(&mut self) {
        self.layout.detach();
    }
}
impl CPU {
    /// create a 6502 microprocessor emulator.
    /// _layout_ must have at least 65536 possible addresses ranging from 0x0000 to 0xFFFF.
    pub fn new(mut layout: Layout) -> Option<Self> {
        if layout.get_byte_count() < u16::MAX as usize {
            return None;
        }
        layout.attach();

        Some(Self {
            pc: 0,
            sp: 0,
            a: Default::default(),
            x: Default::default(),
            y: Default::default(),
            status: Status::default(),
            layout,
            debug_inst: Inst::LDA,
            debug_pc: 0,
            debug_operand: DebugOp::Implied,
            debug_desc: DebugDesc::ChangeVal(0),
        })
    }

    pub fn reset(&mut self) {
        self.layout.reset();

        self.status = Status::default();
        self.a = Default::default();
        self.x = Default::default();
        self.y = Default::default();

        self.sp = 0xFF;
        self.push_byte((self.pc >> 8) as u8);
        self.push_byte((self.pc & 0xFF) as u8);
        self.push_byte(self.status.into());
        self.pc = self.read_word(0xFFFC);
    }

    pub fn is_irq_enabled(&self) -> bool {
        !self.status.int_disable
    }

    pub fn irq(&mut self) {
        if self.status.int_disable {
            if log_enabled!(Level::Trace) {
                trace!("IRQ IGNORED\r");
            }
            return;
        }
        self.push_byte((self.pc >> 8) as u8);
        self.push_byte((self.pc & 0xFF) as u8);
        let mut status = self.status;
        status.break_ = false;
        self.push_byte(status.into());
        self.status.int_disable = true;
        self.pc = self.read_word(0xFFFE);
    }

    pub fn nmi(&mut self) {
        self.push_byte((self.pc >> 8) as u8);
        self.push_byte((self.pc & 0xFF) as u8);
        let mut status = self.status;
        status.break_ = false;
        self.push_byte(status.into());
        self.pc = self.read_word(0xFFFA);
    }

    pub fn step(&mut self) -> Result<(), ExecutionError> {
        self.debug_pc = self.pc;
        self.debug_desc = DebugDesc::Unset;
        let inst_byte = self.next_byte();

        let Some((inst, addr_mode)) = decode_inst(inst_byte) else {
            return Err(ExecutionError::UnknownInst(inst_byte));
        };
        self.debug_inst = inst;

        match inst {
            Inst::LDA => {
                self.a.data = self.read_byte_addressed(addr_mode).1;
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
                self.check_nz(self.a);
            }
            Inst::LDX => {
                self.x.data = self.read_byte_addressed(addr_mode).1;
                self.debug_desc = DebugDesc::ChangeVal(self.x.data);
                self.check_nz(self.x);
            }
            Inst::LDY => {
                self.y.data = self.read_byte_addressed(addr_mode).1;
                self.debug_desc = DebugDesc::ChangeVal(self.y.data);
                self.check_nz(self.y);
            }

            Inst::STA => self.write_byte_addressed(self.a.data, addr_mode),
            Inst::STX => self.write_byte_addressed(self.x.data, addr_mode),
            Inst::STY => self.write_byte_addressed(self.y.data, addr_mode),

            Inst::TAX => {
                self.x = self.a;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.x.data);
                self.check_nz(self.x);
            }
            Inst::TAY => {
                self.y = self.a;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.y.data);
                self.check_nz(self.y);
            }
            Inst::TSX => {
                self.x.data = self.sp;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.x.data);
                self.check_nz(self.x);
            }
            Inst::TXA => {
                self.a = self.x;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
                self.check_nz(self.a);
            }
            Inst::TXS => {
                self.sp = self.x.data;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.sp);
            }
            Inst::TYA => {
                self.a = self.y;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
                self.check_nz(self.a);
            }

            Inst::PHA => {
                self.push_byte(self.a.data);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.a.data, self.sp);
            }
            Inst::PHP => {
                let mut status = self.status;
                status.break_ = true;
                self.push_byte(status.into());
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.status.into(), self.sp);
            }
            Inst::PHX => {
                self.push_byte(self.x.data);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.x.data, self.sp);
            }
            Inst::PHY => {
                self.push_byte(self.y.data);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.y.data, self.sp);
            }
            Inst::PLA => {
                self.a.data = self.pull_byte();
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.a.data, self.sp);
                self.check_nz(self.a);
            }
            Inst::PLP => {
                self.status = Status::from(self.pull_byte());
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.status.into(), self.sp);
            }
            Inst::PLX => {
                self.x.data = self.pull_byte();
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.x.data, self.sp);
                self.check_nz(self.x);
            }
            Inst::PLY => {
                self.y.data = self.pull_byte();
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeStack(self.x.data, self.sp);
                self.check_nz(self.y);
            }

            Inst::DEC => {
                if addr_mode == AddressingMode::Implied {
                    self.a.data = self.a.data.wrapping_sub(1);
                    self.check_nz(self.a);
                    self.debug_operand = DebugOp::Implied;
                    self.debug_desc = DebugDesc::ChangeVal(self.a.data);
                } else {
                    let (addr, mut data) = self.read_byte_addressed(addr_mode);
                    data = data.wrapping_sub(1);
                    self.write_byte(addr, data);
                    self.check_nz(Register { data });
                    self.debug_desc = DebugDesc::ChangeVal(data);
                }
            }
            Inst::DEX => {
                self.x.data = self.x.data.wrapping_sub(1);
                self.check_nz(self.x);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.x.data);
            }
            Inst::DEY => {
                self.y.data = self.y.data.wrapping_sub(1);
                self.check_nz(self.y);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.y.data);
            }
            Inst::INC => {
                if addr_mode == AddressingMode::Implied {
                    self.a.data = self.a.data.wrapping_add(1);
                    self.check_nz(self.a);
                    self.debug_operand = DebugOp::Implied;
                    self.debug_desc = DebugDesc::ChangeVal(self.a.data);
                } else {
                    let (addr, mut data) = self.read_byte_addressed(addr_mode);
                    data = data.wrapping_add(1);
                    self.write_byte(addr, data);
                    self.check_nz(Register { data });
                    self.debug_desc = DebugDesc::ChangeVal(data);
                }
            }
            Inst::INX => {
                self.x.data = self.x.data.wrapping_add(1);
                self.check_nz(self.x);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.x.data);
            }
            Inst::INY => {
                self.y.data = self.y.data.wrapping_add(1);
                self.check_nz(self.y);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::ChangeVal(self.y.data);
            }

            Inst::ADC => {
                let operand = self.read_byte_addressed(addr_mode).1 as u16;
                let result = (self.a.data as u16)
                    .wrapping_add(operand)
                    .wrapping_add(self.status.carry as u16);

                self.status.carry = result > 0xFF;
                self.status.overflow =
                    ((result ^ self.a.data as u16) & (result ^ operand) & 0x80) > 0;
                self.a.data = result as u8;
                self.check_nz(self.a);
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
            }
            Inst::SBC => {
                let operand = self.read_byte_addressed(addr_mode).1 ^ 0xFF;
                let result = (self.a.data as u16)
                    .wrapping_add(operand as u16) // invert operand to get -operand - 1
                    .wrapping_add(self.status.carry as u16);

                self.status.carry = result > 0xFF;
                self.status.overflow =
                    ((result ^ self.a.data as u16) & (result ^ (operand as u16)) & 0x80) > 0;
                self.a.data = result as u8;
                self.check_nz(self.a);
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
            }

            Inst::AND => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data &= data;
                self.check_nz(self.a);
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
            }
            Inst::EOR => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data ^= data;
                self.check_nz(self.a);
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
            }
            Inst::ORA => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.a.data |= data;
                self.check_nz(self.a);
                self.debug_desc = DebugDesc::ChangeVal(self.a.data);
            }

            Inst::ASL => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    self.a.data = data;
                    self.debug_operand = DebugOp::Implied;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b10000000) > 0;
                    data <<= 1;
                    self.write_byte(read.0, data);
                }
                self.check_nz(Register { data });
                self.status.carry = send_carry;
                self.debug_desc = DebugDesc::ChangeVal(data);
            }
            Inst::LSR => {
                let mut data;
                let send_carry;
                if addr_mode == AddressingMode::Implied {
                    data = self.a.data;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    self.a.data = data;
                    self.debug_operand = DebugOp::Implied;
                } else {
                    let read = self.read_byte_addressed(addr_mode);
                    data = read.1;
                    send_carry = (data & 0b1) > 0;
                    data >>= 1;
                    self.write_byte(read.0, data);
                };
                self.check_nz(Register { data });
                self.status.carry = send_carry;
                self.debug_desc = DebugDesc::ChangeVal(data);
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
                    self.debug_operand = DebugOp::Implied;
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
                self.debug_desc = DebugDesc::ChangeVal(data);
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
                    self.debug_operand = DebugOp::Implied;
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
                self.debug_desc = DebugDesc::ChangeVal(data);
            }

            Inst::CLC => {
                self.status.carry = false;
                self.debug_desc = DebugDesc::ChangeVal(self.status.carry as u8);
            }
            Inst::CLD => {
                self.status.decimal = false;
                self.debug_desc = DebugDesc::ChangeVal(self.status.decimal as u8);
            }
            Inst::CLI => {
                self.status.int_disable = false;
                self.debug_desc = DebugDesc::ChangeVal(self.status.int_disable as u8);
            }
            Inst::CLV => {
                self.status.overflow = false;
                self.debug_desc = DebugDesc::ChangeVal(self.status.overflow as u8);
            }
            Inst::SEC => {
                self.status.carry = true;
                self.debug_desc = DebugDesc::ChangeVal(self.status.carry as u8);
            }
            Inst::SED => {
                self.status.decimal = true;
                self.debug_desc = DebugDesc::ChangeVal(self.status.decimal as u8);
            }
            Inst::SEI => {
                self.status.int_disable = true;
                self.debug_desc = DebugDesc::ChangeVal(self.status.int_disable as u8);
            }

            Inst::CMP => {
                let operand = self.read_byte_addressed(addr_mode).1;
                let result = self.a.data.wrapping_sub(operand);
                self.check_nz(Register { data: result });
                self.status.carry = self.a.data >= operand;
                self.debug_desc = DebugDesc::Compare(self.a.data, operand);
            }
            Inst::CPX => {
                let operand = self.read_byte_addressed(addr_mode).1;
                let result = self.x.data.wrapping_sub(operand);
                self.check_nz(Register { data: result });
                self.status.carry = self.x.data >= operand;
                self.debug_desc = DebugDesc::Compare(self.x.data, operand);
            }
            Inst::CPY => {
                let operand = self.read_byte_addressed(addr_mode).1;
                let result = self.y.data.wrapping_sub(operand);
                self.check_nz(Register { data: result });
                self.status.carry = self.y.data >= operand;
                self.debug_desc = DebugDesc::Compare(self.y.data, operand);
            }

            Inst::BRA => {
                let offset = self.read_byte_relative();
                self.pc = (self.pc as i32 + offset as i32) as u16;
            }

            Inst::BCC => {
                let offset = self.read_byte_relative();
                if !self.status.carry {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.carry as u8);
            }
            Inst::BCS => {
                let offset = self.read_byte_relative();
                if self.status.carry {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.carry as u8);
            }

            Inst::BNE => {
                let offset = self.read_byte_relative();
                if !self.status.zero {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.zero as u8);
            }
            Inst::BEQ => {
                let offset = self.read_byte_relative();
                if self.status.zero {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.zero as u8);
            }

            Inst::BPL => {
                let offset = self.read_byte_relative();
                if !self.status.negative {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.negative as u8);
            }
            Inst::BMI => {
                let offset = self.read_byte_relative();
                if self.status.negative {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.negative as u8);
            }

            Inst::BVC => {
                let offset = self.read_byte_relative();
                if !self.status.overflow {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.overflow as u8);
            }
            Inst::BVS => {
                let offset = self.read_byte_relative();
                if self.status.overflow {
                    self.pc = (self.pc as i32 + offset as i32) as u16;
                }
                self.debug_desc = DebugDesc::Cond(self.status.overflow as u8);
            }

            Inst::JMP => match addr_mode {
                AddressingMode::Indirect => {
                    let indirect_addr = self.next_word();
                    let addr = self.read_word(indirect_addr);
                    self.pc = addr;
                    self.debug_operand = DebugOp::Indirect(indirect_addr);
                    self.debug_desc = DebugDesc::Jmp(self.pc);
                }
                AddressingMode::Absolute => {
                    let addr = self.next_word();
                    self.pc = addr;
                    self.debug_operand = DebugOp::Absolute(addr);
                    self.debug_desc = DebugDesc::Jmp(self.pc);
                }
                _ => unimplemented!("JMP {:?}", addr_mode),
            },
            Inst::JSR => {
                let to_addr = self.next_word();
                let ret_addr = self.pc.wrapping_sub(1);
                self.push_byte((ret_addr >> 8) as u8);
                self.push_byte((ret_addr & 0xFF) as u8);
                self.pc = to_addr;
                self.debug_operand = DebugOp::Absolute(self.pc);
                self.debug_desc = DebugDesc::Jmp(self.pc);
            }
            Inst::RTS => {
                let lo_pc = self.pull_byte() as u16;
                let hi_pc = self.pull_byte() as u16;
                self.pc = (hi_pc << 8) | lo_pc;
                self.pc = self.pc.wrapping_add(1); // simulate prefetching
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::Jmp(self.pc);
            }

            Inst::BRK => {
                let pc_next = self.pc + 1;
                self.push_byte((pc_next >> 8) as u8);
                self.push_byte((pc_next & 0xFF) as u8);
                let mut status = self.status;
                status.break_ = true;
                self.push_byte(status.into());
                self.status.int_disable = true;
                self.pc = self.read_word(0xFFFE);
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::Jmp(self.pc);
            }
            Inst::RTI => {
                self.status = Status::from(self.pull_byte());
                let lo_pc = self.pull_byte() as u16;
                let hi_pc = self.pull_byte() as u16;
                self.pc = (hi_pc << 8) | lo_pc;
                self.debug_operand = DebugOp::Implied;
                self.debug_desc = DebugDesc::Restore(self.pc);
            }

            Inst::BIT => {
                let data = self.read_byte_addressed(addr_mode).1;
                self.status.zero = (self.a.data & data) == 0;
                self.status.negative = (data & 0b10000000) > 0;
                self.status.overflow = (data & 0b1000000) > 0;
            }

            Inst::NOP => {
                self.debug_operand = DebugOp::Implied;
            }
        };

        if log_enabled!(log::Level::Trace) {
            trace!("{}", self.trace_exec());
        }

        Ok(())
    }

    pub fn trace_exec(&self) -> String {
        format!(
            "{:#06x} {} {:?} {: <15} ; {}\r",
            self.debug_pc,
            self.status,
            self.debug_inst,
            match self.debug_operand {
                DebugOp::Implied => String::new(),
                DebugOp::Immediate(v) => format!("#${:02x}", v),
                DebugOp::ZeroPage(v) => format!("${:02x}", v),
                DebugOp::ZeroPageX(v, x) => format!("${:02x}, X({:#04x})", v, x),
                DebugOp::ZeroPageY(v, y) => format!("${:02x}, Y({:#04x})", v, y),
                DebugOp::Absolute(v) => format!("${:04x}", v),
                DebugOp::AbsoluteX(v, x) => format!("${:04x}, X({:#04x})", v, x),
                DebugOp::AbsoluteY(v, y) => format!("${:04x}, Y({:#04x})", v, y),
                DebugOp::Relative(v) => format!("${:04x}", (self.pc as i32 + v as i32) as u16),
                DebugOp::Indirect(v) => format!("(${:04x})", v),
                DebugOp::XIndirect(v, x) => format!("(${:02x}, X({:#04x}))", v, x),
                DebugOp::IndirectY(v, y) => format!("(${:02x}), Y({:#04x})", v, y),
            },
            match self.debug_desc {
                DebugDesc::Unset => String::new(),
                DebugDesc::ChangeVal(v) => format!("result = {:#04x}", v),
                DebugDesc::ChangeStack(v, sp) => format!("value = {:#04x}, sp = {:#04x}", v, sp),
                DebugDesc::Compare(reg, operand) =>
                    format!("reg = {:#04x}, operand = {:#04x}", reg, operand),
                DebugDesc::Cond(v) => format!(
                    "flag is {}",
                    match v {
                        0 => "cleared",
                        1 => "set",
                        _ => unimplemented!("DebugDesc::Cond {}", v),
                    }
                ),
                DebugDesc::Jmp(v) => format!("addr = {:#06x}", v),
                DebugDesc::Restore(pc) => format!("pc = {:#06x}", pc),
            }
        )
    }

    fn check_nz(&mut self, reg: Register) {
        self.status.negative = reg.is_negative();
        self.status.zero = reg.is_zero();
    }

    fn push_byte(&mut self, data: u8) {
        self.write_byte(self.get_sp(), data);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pull_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.read_byte(self.get_sp())
    }

    fn get_sp(&self) -> u16 {
        self.sp as u16 + 0x100
    }

    fn read_byte_relative(&mut self) -> i8 {
        let rel_addr = self.next_byte() as i8;
        self.debug_operand = DebugOp::Relative(rel_addr);
        rel_addr
    }

    fn read_byte_addressed(&mut self, addr_mode: AddressingMode) -> (u16, u8) {
        match addr_mode {
            AddressingMode::Implied => unimplemented!("Implied addressing mode"),
            AddressingMode::Immediate => {
                let data = self.next_byte();
                self.debug_operand = DebugOp::Immediate(data);
                (self.pc, data)
            }
            AddressingMode::Absolute => {
                let addr = self.next_word();
                self.debug_operand = DebugOp::Absolute(addr);
                (addr, self.read_byte(addr))
            }
            AddressingMode::AbsoluteX => {
                let abs_addr = self.next_word();
                let addr = abs_addr.wrapping_add(self.x.data as u16);
                self.debug_operand = DebugOp::AbsoluteX(abs_addr, self.x.data);
                (addr, self.read_byte(addr))
            }
            AddressingMode::AbsoluteY => {
                let abs_addr = self.next_word();
                let addr = abs_addr.wrapping_add(self.y.data as u16);
                self.debug_operand = DebugOp::AbsoluteY(abs_addr, self.y.data);
                (addr, self.read_byte(addr))
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let zp_addr = self.next_byte();
                let indexed = zp_addr.wrapping_add(self.x.data);
                let addr = self.read_word(indexed as u16);
                self.debug_operand = DebugOp::XIndirect(zp_addr, self.x.data);
                (addr, self.read_byte(addr))
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte();
                let addr = self.read_word(zp_addr as u16) + self.y.data as u16;
                self.debug_operand = DebugOp::IndirectY(zp_addr, self.y.data);
                (addr, self.read_byte(addr))
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let addr = self.next_byte();
                self.debug_operand = DebugOp::ZeroPage(addr);
                (addr as u16, self.read_byte(addr as u16))
            }
            AddressingMode::ZeroPageX => {
                let zp_addr = self.next_byte();
                let addr = (zp_addr.wrapping_add(self.x.data)) as u16;
                self.debug_operand = DebugOp::ZeroPageX(zp_addr, self.x.data);
                (addr, self.read_byte(addr))
            }
            AddressingMode::ZeroPageY => {
                let zp_addr = self.next_byte();
                let addr = (zp_addr.wrapping_add(self.y.data)) as u16;
                self.debug_operand = DebugOp::ZeroPageY(zp_addr, self.y.data);
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
                self.debug_operand = DebugOp::Absolute(addr);
                self.write_byte(addr, data);
            }
            AddressingMode::AbsoluteX => {
                let abs_addr = self.next_word();
                let addr = abs_addr.wrapping_add(self.x.data as u16);
                self.debug_operand = DebugOp::AbsoluteX(abs_addr, self.x.data);
                self.write_byte(addr, data);
            }
            AddressingMode::AbsoluteY => {
                let abs_addr = self.next_word();
                let addr = abs_addr.wrapping_add(self.y.data as u16);
                self.debug_operand = DebugOp::AbsoluteY(abs_addr, self.y.data);
                self.write_byte(addr, data);
            }
            AddressingMode::Indirect => unimplemented!("Indirect addressing mode"),
            AddressingMode::XIndirect => {
                let zp_addr = self.next_byte();
                let addr = self.read_word(zp_addr.wrapping_add(self.x.data) as u16);
                self.debug_operand = DebugOp::XIndirect(zp_addr, self.x.data);
                self.write_byte(addr, data);
            }
            AddressingMode::IndirectY => {
                let zp_addr = self.next_byte();
                let addr = self.read_word(zp_addr as u16) + self.y.data as u16;
                self.debug_operand = DebugOp::IndirectY(zp_addr, self.y.data);
                self.write_byte(addr, data);
            }
            AddressingMode::Relative => unimplemented!("Relative addressing mode"),
            AddressingMode::ZeroPage => {
                let zp_addr = self.next_byte();
                self.debug_operand = DebugOp::ZeroPage(zp_addr);
                self.write_byte(zp_addr as u16, data);
            }
            AddressingMode::ZeroPageX => {
                let zp_addr = self.next_byte();
                let addr = zp_addr.wrapping_add(self.x.data) as u16;
                self.debug_operand = DebugOp::ZeroPageX(zp_addr, self.x.data);
                self.write_byte(addr, data);
            }
            AddressingMode::ZeroPageY => {
                let zp_addr = self.next_byte();
                let addr = zp_addr.wrapping_add(self.y.data) as u16;
                self.debug_operand = DebugOp::ZeroPageY(zp_addr, self.y.data);
                self.write_byte(addr, data);
            }
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn next_word(&mut self) -> u16 {
        let word = self.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match self.layout.read(addr as usize) {
            Some(v) => v,
            None => {
                if log_enabled!(Level::Trace) {
                    trace!("read byte at {:#06x} failed", addr);
                }
                0
            }
        }
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        // not going to verify write result
        self.layout.write(addr as usize, data);
    }

    pub fn set_pc(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
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
    int_disable: bool,
    zero: bool,
    carry: bool,
}
impl From<Status> for u8 {
    fn from(val: Status) -> Self {
        (val.negative as u8) << 7
            | (val.overflow as u8) << 6
            | (1 << 5)
            | (val.break_ as u8) << 4
            | (val.decimal as u8) << 3
            | (val.int_disable as u8) << 2
            | (val.zero as u8) << 1
            | (val.carry as u8)
    }
}
impl From<u8> for Status {
    fn from(value: u8) -> Self {
        Self {
            negative: (value & 0b10000000) > 0,
            overflow: (value & 0b1000000) > 0,
            break_: (value & 0b10000) > 0,
            decimal: (value & 0b1000) > 0,
            int_disable: (value & 0b100) > 0,
            zero: (value & 0b10) > 0,
            carry: (value & 0b1) > 0,
        }
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "N{},Z{},C{},I{},D{},V{},B{}",
            self.negative as u8,
            self.zero as u8,
            self.carry as u8,
            self.int_disable as u8,
            self.decimal as u8,
            self.overflow as u8,
            self.break_ as u8
        )
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

#[derive(Debug)]
enum DebugOp {
    Implied,
    Immediate(u8),
    ZeroPage(u8),
    ZeroPageX(u8, u8),
    ZeroPageY(u8, u8),
    Absolute(u16),
    AbsoluteX(u16, u8),
    AbsoluteY(u16, u8),
    Indirect(u16),
    Relative(i8),
    XIndirect(u8, u8),
    IndirectY(u8, u8),
}

#[derive(Debug)]
enum DebugDesc {
    Unset,
    ChangeVal(u8),       // result
    ChangeStack(u8, u8), // value, sp
    Compare(u8, u8),     // Reg, Mem
    Cond(u8),            // flag status
    Jmp(u16),            // addr
    Restore(u16),        // pc
}
