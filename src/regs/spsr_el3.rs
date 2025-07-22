/// SPSR_EL3 - Saved Program Status Register (Exception Level 3)
use bitfield::bitfield;

bitfield! {
    /// SPSR_EL3 - Saved Program Status Register (Exception Level 3)
    ///
    /// This register stores the processor state when an exception is taken to EL3.
    /// Fields are restored to PSTATE on exception return from EL3.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SpsrEl3(u64);

    // Bits [63:37] - Reserved, RES0

    /// Bit [36] - Inject Undefined Instruction exception (FEAT_UINJ)
    pub uinj, set_uinj: 36;

    /// Bit [35] - PACM (FEAT_PAuth_LR)
    pub pacm, set_pacm: 35;

    /// Bit [34] - Exception return state lock (FEAT_GCS)
    pub exlock, set_exlock: 34;

    /// Bit [33] - PMU Profiling exception pending bit (FEAT_SEBEP)
    pub ppend, set_ppend: 33;

    /// Bit [32] - Profiling exception mask bit (FEAT_EBEP)
    pub pm, set_pm: 32;

    /// Bit [31] - Negative Condition flag
    pub n, set_n: 31;

    /// Bit [30] - Zero Condition flag
    pub z, set_z: 30;

    /// Bit [29] - Carry Condition flag
    pub c, set_c: 29;

    /// Bit [28] - Overflow Condition flag
    pub v, set_v: 28;

    // Bits [27:26] - Reserved, RES0

    /// Bit [25] - Tag Check Override (FEAT_MTE)
    pub tco, set_tco: 25;

    /// Bit [24] - Data Independent Timing (FEAT_DIT)
    pub dit, set_dit: 24;

    /// Bit [23] - User Access Override (FEAT_UAO)
    pub uao, set_uao: 23;

    /// Bit [22] - Privileged Access Never (FEAT_PAN)
    pub pan, set_pan: 22;

    /// Bit [21] - Software Step
    pub ss, set_ss: 21;

    /// Bit [20] - Illegal Execution state
    pub il, set_il: 20;

    // Bits [19:14] - Reserved, RES0

    /// Bit [13] - All IRQ or FIQ interrupts mask (FEAT_NMI)
    pub allint, set_allint: 13;

    /// Bit [12] - Speculative Store Bypass (FEAT_SSBS)
    pub ssbs, set_ssbs: 12;

    /// Bits [11:10] - Branch Type Indicator (FEAT_BTI)
    pub btype, set_btype: 11, 10;

    /// Bit [9] - Debug exception mask
    pub d, set_d: 9;

    /// Bit [8] - SError exception mask
    pub a, set_a: 8;

    /// Bit [7] - IRQ interrupt mask
    pub i, set_i: 7;

    /// Bit [6] - FIQ interrupt mask
    pub f, set_f: 6;

    // Bit [5] - Reserved, RES0

    /// Bit [4] - Execution state (M[4])
    /// 0 = AArch64, 1 = AArch32
    pub m4, set_m4: 4;

    /// Bits [3:0] - Exception level and Stack Pointer selection (M[3:0])
    pub m3_0, set_m3_0: 3, 0;
}

impl SpsrEl3 {
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

    /// Set exception level (bits [3:2] of M field)
    pub fn set_exception_level(&mut self, el: u8) {
        let current_m = self.m3_0();
        let new_m = (current_m & 0b0011) | (((el & 0b11) as u64) << 2);
        self.set_m3_0(new_m);
    }

    /// Get exception level (bits [3:2] of M field)
    pub fn exception_level(&self) -> u8 {
        ((self.m3_0() >> 2) & 0b11) as u8
    }

    /// Set stack pointer selection (bit [0] of M field)
    /// false = SP_ELx (dedicated stack pointer)
    /// true = SP_EL0 (shared stack pointer)
    pub fn set_stack_pointer(&mut self, use_el0_sp: bool) {
        let current_m = self.m3_0();
        let new_m = (current_m & 0b1110) | (use_el0_sp as u64);
        self.set_m3_0(new_m);
    }

    /// Get stack pointer selection (bit [0] of M field)
    pub fn stack_pointer_is_el0(&self) -> bool {
        (self.m3_0() & 1) == 0
    }

    /// Set condition flags (NZCV)
    pub fn set_condition_flags(&mut self, n: bool, z: bool, c: bool, v: bool) {
        self.set_n(n);
        self.set_z(z);
        self.set_c(c);
        self.set_v(v);
    }

    /// Set interrupt masks (DAIF)
    pub fn set_interrupt_masks(&mut self, d: bool, a: bool, i: bool, f: bool) {
        self.set_d(d);
        self.set_a(a);
        self.set_i(i);
        self.set_f(f);
    }
}

impl Default for SpsrEl3 {
    fn default() -> Self {
        Self::new()
    }
}

// Common exception level and stack pointer combinations
impl SpsrEl3 {
    /// EL0 (User mode)
    pub const EL0: u64 = 0b0000;

    /// EL1 with SP_EL0 (EL1t)
    pub const EL1T: u64 = 0b0100;

    /// EL1 with SP_EL1 (EL1h)
    pub const EL1H: u64 = 0b0101;

    /// EL2 with SP_EL0 (EL2t)
    pub const EL2T: u64 = 0b1000;

    /// EL2 with SP_EL2 (EL2h)
    pub const EL2H: u64 = 0b1001;

    /// EL3 with SP_EL0 (EL3t)
    pub const EL3T: u64 = 0b1100;

    /// EL3 with SP_EL3 (EL3h)
    pub const EL3H: u64 = 0b1101;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spsr_el3_basic() {
        let mut spsr = SpsrEl3::new();
        assert_eq!(spsr.raw(), 0);

        spsr.set_m3_0(SpsrEl3::EL1H);
        assert_eq!(spsr.exception_level(), 1);
        assert!(!spsr.stack_pointer_is_el0());

        spsr.set_condition_flags(true, false, true, false);
        assert!(spsr.n());
        assert!(!spsr.z());
        assert!(spsr.c());
        assert!(!spsr.v());
    }

    #[test]
    fn test_exception_levels() {
        let mut spsr = SpsrEl3::new();

        spsr.set_m3_0(SpsrEl3::EL0);
        assert_eq!(spsr.exception_level(), 0);

        spsr.set_m3_0(SpsrEl3::EL2H);
        assert_eq!(spsr.exception_level(), 2);
        assert!(!spsr.stack_pointer_is_el0());
    }
}
