#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inst {
    LDA,
    LDX,
    LDY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    XIndirect,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

pub fn decode_inst(byte: u8) -> Option<(Inst, AddressingMode)> {
    use AddressingMode::*;
    use Inst::*;
    Some(match byte {
        0xA9 => (LDA, Immediate),
        0xA5 => (LDA, ZeroPage),
        0xB5 => (LDA, ZeroPageX),
        0xAD => (LDA, Absolute),
        0xBD => (LDA, AbsoluteX),
        0xB9 => (LDA, AbsoluteY),
        0xA1 => (LDA, XIndirect),
        0xB1 => (LDA, IndirectY),

        0xA2 => (LDX, Immediate),
        0xA6 => (LDX, ZeroPage),
        0xB6 => (LDX, ZeroPageY),
        0xAE => (LDX, Absolute),
        0xBE => (LDX, AbsoluteY),

        0xA0 => (LDY, Immediate),
        0xA4 => (LDY, ZeroPage),
        0xB4 => (LDY, ZeroPageX),
        0xAC => (LDY, Absolute),
        0xBC => (LDY, AbsoluteX),

        _ => return None,
    })
}
