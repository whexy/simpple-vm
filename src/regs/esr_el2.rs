use ahv::Register;
/// ESR_EL2 - Exception Syndrome Register (Exception Level 2)
use bitfield::bitfield;

/// Exception Class values for ESR_EL2
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ExceptionClass {
    /// Unknown reason
    Unknown = 0b000000,
    /// Trapped WF* instruction execution
    TrappedWfInstruction = 0b000001,
    /// Trapped MCR or MRC access with (coproc==0b1111)
    TrappedMcrMrcCp15 = 0b000011,
    /// Trapped MCRR or MRRC access with (coproc==0b1111)
    TrappedMcrrMrrcCp15 = 0b000100,
    /// Trapped MCR or MRC access with (coproc==0b1110)
    TrappedMcrMrcCp14 = 0b000101,
    /// Trapped LDC or STC access
    TrappedLdcStc = 0b000110,
    /// Access to SME, SVE, Advanced SIMD or floating-point functionality trapped
    TrappedSimdFp = 0b000111,
    /// Trapped VMRS access, from ID group trap
    TrappedVmrs = 0b001000,
    /// Trapped use of a Pointer authentication instruction
    TrappedPointerAuth = 0b001001,
    /// Trapped execution of any instruction not covered by other EC values
    TrappedOtherInstruction = 0b001010,
    /// Trapped MRRC access with (coproc==0b1110)
    TrappedMrrcCp14 = 0b001100,
    /// Branch Target Exception
    BranchTargetException = 0b001101,
    /// Illegal Execution state
    IllegalExecutionState = 0b001110,
    /// SVC instruction execution in AArch32 state
    SvcAArch32 = 0b010001,
    /// HVC instruction execution in AArch32 state
    HvcAArch32 = 0b010010,
    /// SMC instruction execution in AArch32 state
    SmcAArch32 = 0b010011,
    /// Trapped MSRR, MRRS or System instruction execution in AArch64 state
    TrappedSysregAArch64_128bit = 0b010100,
    /// SVC instruction execution in AArch64 state
    SvcAArch64 = 0b010101,
    /// HVC instruction execution in AArch64 state
    HvcAArch64 = 0b010110,
    /// SMC instruction execution in AArch64 state
    SmcAArch64 = 0b010111,
    /// Trapped MSR, MRS or System instruction execution in AArch64 state
    TrappedSysregAArch64 = 0b011000,
    /// Access to SVE functionality trapped
    TrappedSve = 0b011001,
    /// Trapped ERET, ERETAA, or ERETAB instruction execution
    TrappedEret = 0b011010,
    /// Exception from an access to a TSTART instruction
    TrappedTstart = 0b011011,
    /// Exception from a PAC Fail
    PacFail = 0b011100,
    /// Access to SME functionality trapped
    TrappedSme = 0b011101,
    /// Instruction Abort from a lower Exception level
    InstructionAbortLowerEl = 0b100000,
    /// Instruction Abort taken without a change in Exception level
    InstructionAbortSameEl = 0b100001,
    /// PC alignment fault exception
    PcAlignmentFault = 0b100010,
    /// Data Abort exception from a lower Exception level
    DataAbortLowerEl = 0b100100,
    /// Data Abort exception without a change in Exception level
    DataAbortSameEl = 0b100101,
    /// SP alignment fault exception
    SpAlignmentFault = 0b100110,
    /// Memory Operation Exception
    MemoryOperation = 0b100111,
    /// Trapped floating-point exception taken from AArch32 state
    TrappedFpAArch32 = 0b101000,
    /// Trapped floating-point exception taken from AArch64 state
    TrappedFpAArch64 = 0b101100,
    /// GCS exception
    GcsException = 0b101101,
    /// SError exception
    SError = 0b101111,
    /// Breakpoint exception from a lower Exception level
    BreakpointLowerEl = 0b110000,
    /// Breakpoint exception taken without a change in Exception level
    BreakpointSameEl = 0b110001,
    /// Software Step exception from a lower Exception level
    SoftwareStepLowerEl = 0b110010,
    /// Software Step exception taken without a change in Exception level
    SoftwareStepSameEl = 0b110011,
    /// Watchpoint from a lower Exception level
    WatchpointLowerEl = 0b110100,
    /// Watchpoint exceptions without a change in Exception level
    WatchpointSameEl = 0b110101,
    /// BKPT instruction execution in AArch32 state
    BkptAArch32 = 0b111000,
    /// Vector Catch exception from AArch32 state
    VectorCatchAArch32 = 0b111010,
    /// BRK instruction execution in AArch64 state
    BrkAArch64 = 0b111100,
    /// Profiling exception
    ProfilingException = 0b111101,
    /// Unrecognized exception class
    Unrecognized(u8),
}

impl From<u8> for ExceptionClass {
    fn from(value: u8) -> Self {
        match value {
            0b000000 => ExceptionClass::Unknown,
            0b000001 => ExceptionClass::TrappedWfInstruction,
            0b000011 => ExceptionClass::TrappedMcrMrcCp15,
            0b000100 => ExceptionClass::TrappedMcrrMrrcCp15,
            0b000101 => ExceptionClass::TrappedMcrMrcCp14,
            0b000110 => ExceptionClass::TrappedLdcStc,
            0b000111 => ExceptionClass::TrappedSimdFp,
            0b001000 => ExceptionClass::TrappedVmrs,
            0b001001 => ExceptionClass::TrappedPointerAuth,
            0b001010 => ExceptionClass::TrappedOtherInstruction,
            0b001100 => ExceptionClass::TrappedMrrcCp14,
            0b001101 => ExceptionClass::BranchTargetException,
            0b001110 => ExceptionClass::IllegalExecutionState,
            0b010001 => ExceptionClass::SvcAArch32,
            0b010010 => ExceptionClass::HvcAArch32,
            0b010011 => ExceptionClass::SmcAArch32,
            0b010100 => ExceptionClass::TrappedSysregAArch64_128bit,
            0b010101 => ExceptionClass::SvcAArch64,
            0b010110 => ExceptionClass::HvcAArch64,
            0b010111 => ExceptionClass::SmcAArch64,
            0b011000 => ExceptionClass::TrappedSysregAArch64,
            0b011001 => ExceptionClass::TrappedSve,
            0b011010 => ExceptionClass::TrappedEret,
            0b011011 => ExceptionClass::TrappedTstart,
            0b011100 => ExceptionClass::PacFail,
            0b011101 => ExceptionClass::TrappedSme,
            0b100000 => ExceptionClass::InstructionAbortLowerEl,
            0b100001 => ExceptionClass::InstructionAbortSameEl,
            0b100010 => ExceptionClass::PcAlignmentFault,
            0b100100 => ExceptionClass::DataAbortLowerEl,
            0b100101 => ExceptionClass::DataAbortSameEl,
            0b100110 => ExceptionClass::SpAlignmentFault,
            0b100111 => ExceptionClass::MemoryOperation,
            0b101000 => ExceptionClass::TrappedFpAArch32,
            0b101100 => ExceptionClass::TrappedFpAArch64,
            0b101101 => ExceptionClass::GcsException,
            0b101111 => ExceptionClass::SError,
            0b110000 => ExceptionClass::BreakpointLowerEl,
            0b110001 => ExceptionClass::BreakpointSameEl,
            0b110010 => ExceptionClass::SoftwareStepLowerEl,
            0b110011 => ExceptionClass::SoftwareStepSameEl,
            0b110100 => ExceptionClass::WatchpointLowerEl,
            0b110101 => ExceptionClass::WatchpointSameEl,
            0b111000 => ExceptionClass::BkptAArch32,
            0b111010 => ExceptionClass::VectorCatchAArch32,
            0b111100 => ExceptionClass::BrkAArch64,
            0b111101 => ExceptionClass::ProfilingException,
            _ => ExceptionClass::Unrecognized(value),
        }
    }
}

bitfield! {
    /// ESR_EL2 - Exception Syndrome Register (Exception Level 2)
    ///
    /// This Register holds syndrome information for an exception taken to EL2.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EsrEl2(u64);

    // Bits [63:56] - Reserved, RES0

    /// Bit [55:32] - ISS2
    pub iss2, set_iss2: 55, 32;

    /// Bit [31:26] - EC
    pub ec, set_ec: 31, 26;

    /// Bit [25] - IL
    pub il, set_il: 25;

    /// Bit [24:0] - ISS
    pub iss, set_iss: 24, 0;
}

impl EsrEl2 {
    /// Create a new SPSR_EL3 with all fields cleared
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create SPSR_EL3 from raw u64 value
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Get raw u64 value
    pub const fn raw(&self) -> u64 {
        self.0
    }

    /// Get the Exception Class
    pub fn exception_class(&self) -> ExceptionClass {
        ExceptionClass::from(self.ec() as u8)
    }
}

impl Default for EsrEl2 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SyndromeAccessSize {
    Byte = 0b00,
    Halfword = 0b01,
    Word = 0b10,
    DoubleWord = 0b11,
}

impl From<u8> for SyndromeAccessSize {
    fn from(value: u8) -> Self {
        match value {
            0b00 => SyndromeAccessSize::Byte,
            0b01 => SyndromeAccessSize::Halfword,
            0b10 => SyndromeAccessSize::Word,
            0b11 => SyndromeAccessSize::DoubleWord,
            _ => panic!("Invalid access size value"),
        }
    }
}

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
        !self.wnr()
    }

    pub fn access_register(&self) -> Register {
        match self.srt() {
            0b00000 => Register::X0,
            0b00001 => Register::X1,
            0b00010 => Register::X2,
            0b00011 => Register::X3,
            0b00100 => Register::X4,
            0b00101 => Register::X5,
            0b00110 => Register::X6,
            0b00111 => Register::X7,
            0b01000 => Register::X8,
            0b01001 => Register::X9,
            0b01010 => Register::X10,
            0b01011 => Register::X11,
            0b01100 => Register::X12,
            0b01101 => Register::X13,
            0b01110 => Register::X14,
            0b01111 => Register::X15,
            0b10000 => Register::X16,
            0b10001 => Register::X17,
            0b10010 => Register::X18,
            0b10011 => Register::X19,
            0b10100 => Register::X20,
            0b10101 => Register::X21,
            0b10110 => Register::X22,
            0b10111 => Register::X23,
            0b11000 => Register::X24,
            0b11001 => Register::X25,
            0b11010 => Register::X26,
            0b11011 => Register::X27,
            0b11100 => Register::X28,
            0b11101 => Register::X29,
            0b11110 => Register::X30,
            _ => panic!("Invalid register transfer value"),
        }
    }
}

impl Default for DataAbortISS {
    fn default() -> Self {
        Self::new()
    }
}
