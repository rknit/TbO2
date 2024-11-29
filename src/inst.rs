#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inst {
    LDA,
    LDX,
    LDY,

    STA,
    STX,
    STY,

    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,

    PHA,
    PHP,
    PLA,
    PLP,

    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,

    ADC,
    SBC,

    AND,
    EOR,
    ORA,

    ASL,
    LSR,
    ROL,
    ROR,
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

        0x85 => (STA, ZeroPage),
        0x95 => (STA, ZeroPageX),
        0x8D => (STA, Absolute),
        0x9D => (STA, AbsoluteX),
        0x99 => (STA, AbsoluteY),
        0x81 => (STA, XIndirect),
        0x91 => (STA, IndirectY),

        0x86 => (STX, ZeroPage),
        0x96 => (STX, ZeroPageY),
        0x8E => (STX, Absolute),

        0x84 => (STY, ZeroPage),
        0x94 => (STY, ZeroPageX),
        0x8C => (STY, Absolute),

        0xAA => (TAX, Implied),
        0xA8 => (TAY, Implied),
        0xBA => (TSX, Implied),
        0x8A => (TXA, Implied),
        0x9A => (TXS, Implied),
        0x98 => (TYA, Implied),

        0x48 => (PHA, Implied),
        0x08 => (PHP, Implied),
        0x68 => (PLA, Implied),
        0x28 => (PLP, Implied),

        0xC6 => (DEC, ZeroPage),
        0xD6 => (DEC, ZeroPageX),
        0xCE => (DEC, Absolute),
        0xDE => (DEC, AbsoluteX),
        0xCA => (DEX, Implied),
        0x88 => (DEY, Implied),
        0xE6 => (INC, ZeroPage),
        0xF6 => (INC, ZeroPageX),
        0xEE => (INC, Absolute),
        0xFE => (INC, AbsoluteX),
        0xE8 => (INX, Implied),
        0xC8 => (INY, Implied),

        0x69 => (ADC, Immediate),
        0x65 => (ADC, ZeroPage),
        0x75 => (ADC, ZeroPageX),
        0x6D => (ADC, Absolute),
        0x7D => (ADC, AbsoluteX),
        0x79 => (ADC, AbsoluteY),
        0x61 => (ADC, XIndirect),
        0x71 => (ADC, IndirectY),

        0xE9 => (SBC, Immediate),
        0xE5 => (SBC, ZeroPage),
        0xF5 => (SBC, ZeroPageX),
        0xED => (SBC, Absolute),
        0xFD => (SBC, AbsoluteX),
        0xF9 => (SBC, AbsoluteY),
        0xE1 => (SBC, XIndirect),
        0xF1 => (SBC, IndirectY),

        0x29 => (AND, Immediate),
        0x25 => (AND, ZeroPage),
        0x35 => (AND, ZeroPageX),
        0x2D => (AND, Absolute),
        0x3D => (AND, AbsoluteX),
        0x39 => (AND, AbsoluteY),
        0x21 => (AND, XIndirect),
        0x31 => (AND, IndirectY),

        0x49 => (EOR, Immediate),
        0x45 => (EOR, ZeroPage),
        0x55 => (EOR, ZeroPageX),
        0x4D => (EOR, Absolute),
        0x5D => (EOR, AbsoluteX),
        0x59 => (EOR, AbsoluteY),
        0x41 => (EOR, XIndirect),
        0x51 => (EOR, IndirectY),

        0x09 => (ORA, Immediate),
        0x05 => (ORA, ZeroPage),
        0x15 => (ORA, ZeroPageX),
        0x0D => (ORA, Absolute),
        0x1D => (ORA, AbsoluteX),
        0x19 => (ORA, AbsoluteY),
        0x01 => (ORA, XIndirect),
        0x11 => (ORA, IndirectY),

        0x0A => (ASL, Implied),
        0x06 => (ASL, ZeroPage),
        0x16 => (ASL, ZeroPageX),
        0x0E => (ASL, Absolute),
        0x1E => (ASL, AbsoluteX),

        0x4A => (LSR, Implied),
        0x46 => (LSR, ZeroPage),
        0x56 => (LSR, ZeroPageX),
        0x4E => (LSR, Absolute),
        0x5E => (LSR, AbsoluteX),

        0x2A => (ROL, Implied),
        0x26 => (ROL, ZeroPage),
        0x36 => (ROL, ZeroPageX),
        0x2E => (ROL, Absolute),
        0x3E => (ROL, AbsoluteX),

        0x6A => (ROR, Implied),
        0x66 => (ROR, ZeroPage),
        0x76 => (ROR, ZeroPageX),
        0x6E => (ROR, Absolute),
        0x7E => (ROR, AbsoluteX),

        _ => return None,
    })
}
