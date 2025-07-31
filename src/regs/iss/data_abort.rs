use crate::regs::{SyndromeAccessSize, VRegister};
use ahv::*;
use bitfield::bitfield;

bitfield! {
    /// ISS - Instruction Specific Syndrome
    ///
    /// This field provides additional information about the exception.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DataAbortISS(u32);

    // Bits [31:24] - Not used

    /// Bits [24] - Is Valid
    isv, set_isv: 24;

    /// Bits [23:22] - Access Size
    sas, set_sas: 23, 22;

    /// Bits [21] - Sign Extend
    sse, set_sse: 21;

    /// Bits [20:16] - Register Transfer
    srt, set_srt: 20, 16;

    /// Bits [15] - 32 or 64 register
    sf, set_sf: 15;

    // Bits [14:7] - We don't care

    /// Bits [6] - Write not Read
    wnr, set_wnr: 6;

    // Bits [5:0] - DFSC, we don't care
}

impl DataAbortISS {
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

    /// Get the access size
    pub fn access_size(&self) -> SyndromeAccessSize {
        SyndromeAccessSize::from(self.sas() as u8)
    }

    pub fn is_write(&self) -> bool {
        self.wnr()
    }

    pub fn access_register(&self) -> VRegister {
        match self.srt() {
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
}

impl Default for DataAbortISS {
    fn default() -> Self {
        Self::new()
    }
}
