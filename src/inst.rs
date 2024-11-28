#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inst {
    LDA,
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

        _ => return None,
    })
}
