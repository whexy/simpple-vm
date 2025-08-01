use crate::regs::{EmulatedSystemRegister, VRegister};
use ahvf::*;
use bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SysRegAbortISS(u32);

    // Bits [31:22] - Not used

    /// Bits [21:20] - Op0
    op0, set_op0: 21, 20;

    /// Bits [19:17] - Op2
    op2, set_op2: 19, 17;

    /// Bits [16:14] - Op1
    op1, set_op1: 16, 14;

    /// Bits [13:10] - CRn
    crn, set_crn: 13, 10;

    /// Bits [9:5] - Rt
    rt, set_rt: 9, 5;

    /// Bits [4:1] - CRm
    crm, set_crm: 4, 1;

    /// Bits [0] - Direction
    direction, set_direction: 0;
}

impl SysRegAbortISS {
    /// Create a new ISS with all fields cleared
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create ISS from raw u32 value
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    /// Get raw u32 value
    pub const fn raw(&self) -> u32 {
        self.0
    }

    /// Get the access direction (read/write)
    pub fn is_write(&self) -> bool {
        // 0b0: write (MSR)
        // 0b1: read (MRS)
        !self.direction()
    }

    /// Reconstruct the system register access instruction (MSR or MRS)
    pub fn reconstruct(&self) -> u32 {
        let mut insn: u32 = 0xD5000000; // Base instruction for MSR
        insn |= 1 << 20;
        if !self.is_write() {
            insn |= 1 << 21;
        }
        insn |= 1 << 19;

        insn |= (self.op1() << 16)
            | (self.crn() << 12)
            | (self.crm() << 8)
            | (self.op2() << 5)
            | self.rt();

        insn
    }

    pub fn access_register(&self) -> VRegister {
        match self.rt() {
            0b00000 => VRegister::Register(Register::X0),
            0b00001 => VRegister::Register(Register::X1),
            0b00010 => VRegister::Register(Register::X2),
            0b00011 => VRegister::Register(Register::X3),
            0b00100 => VRegister::Register(Register::X4),
            0b00101 => VRegister::Register(Register::X5),
            0b00110 => VRegister::Register(Register::X6),
            0b00111 => VRegister::Register(Register::X7),
            0b01000 => VRegister::Register(Register::X8),
            0b01001 => VRegister::Register(Register::X9),
            0b01010 => VRegister::Register(Register::X10),
            0b01011 => VRegister::Register(Register::X11),
            0b01100 => VRegister::Register(Register::X12),
            0b01101 => VRegister::Register(Register::X13),
            0b01110 => VRegister::Register(Register::X14),
            0b01111 => VRegister::Register(Register::X15),
            0b10000 => VRegister::Register(Register::X16),
            0b10001 => VRegister::Register(Register::X17),
            0b10010 => VRegister::Register(Register::X18),
            0b10011 => VRegister::Register(Register::X19),
            0b10100 => VRegister::Register(Register::X20),
            0b10101 => VRegister::Register(Register::X21),
            0b10110 => VRegister::Register(Register::X22),
            0b10111 => VRegister::Register(Register::X23),
            0b11000 => VRegister::Register(Register::X24),
            0b11001 => VRegister::Register(Register::X25),
            0b11010 => VRegister::Register(Register::X26),
            0b11011 => VRegister::Register(Register::X27),
            0b11100 => VRegister::Register(Register::X28),
            0b11101 => VRegister::Register(Register::X29),
            0b11110 => VRegister::Register(Register::X30),
            0b11111 => VRegister::ZeroRegister, // access to XZR (zero register)
            srt => panic!("Invalid register transfer value {srt}"),
        }
    }

    pub fn system_register(&self) -> EmulatedSystemRegister {
        match (self.op0(), self.op1(), self.crn(), self.crm(), self.op2()) {
            (3, 7, 7, 12, 1) => EmulatedSystemRegister::CntpCtEl0,
            (3, 3, 14, 0, 1) => EmulatedSystemRegister::CntpCtEl0,
            (op0, op1, crn, crm, op2) => panic!(
                "Unsupported system register access: op0={op0}, op1={op1}, crn={crn}, crm={crm}, op2={op2}"
            ),
        }
    }
}

impl Default for SysRegAbortISS {
    fn default() -> Self {
        Self::new()
    }
}
